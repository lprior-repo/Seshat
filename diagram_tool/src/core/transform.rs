use crate::geometry::operations::compute_subgraph_bounds;
use diagram_models::document::{DiagramDocument, Node, NodeId, NodeKind, OrderedFloat};
use im::HashMap;
use itertools::Itertools;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransformError {
    #[error("No items selected to align")]
    EmptySelection,
    #[error("Locked nodes cannot be transformed: {0}")]
    LockedNode(NodeId),
    #[error("Translation delta is not a finite number")]
    InvalidDelta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentMode {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy)]
struct NodeExtent {
    pos: f64,
    size: f64,
}

impl NodeExtent {
    const fn from_node(node: &Node, axis: AlignmentAxis) -> Self {
        match axis {
            AlignmentAxis::Horizontal => Self {
                pos: node.x.0,
                size: node.width.0,
            },
            AlignmentAxis::Vertical => Self {
                pos: node.y.0,
                size: node.height.0,
            },
        }
    }

    fn apply_to(&self, node: &Node, axis: AlignmentAxis) -> Node {
        match axis {
            AlignmentAxis::Horizontal => Node {
                x: OrderedFloat::new_unchecked(self.pos),
                ..node.clone()
            },
            AlignmentAxis::Vertical => Node {
                y: OrderedFloat::new_unchecked(self.pos),
                ..node.clone()
            },
        }
    }
}

fn recompute_container_bounds(doc: &mut DiagramDocument, moved_node_ids: &[NodeId]) {
    let containers: Vec<NodeId> = moved_node_ids
        .iter()
        .filter_map(|id| doc.document.nodes.get(id))
        .filter_map(|node| node.parent.as_ref())
        .filter(|pid| {
            doc.document
                .nodes
                .get(*pid)
                .is_some_and(|p| p.kind == NodeKind::Subgraph)
        })
        .unique()
        .cloned()
        .collect();

    doc.document.nodes = containers
        .into_iter()
        .fold(doc.document.nodes.clone(), |nodes, cid| {
            let children_bounds: Vec<_> = nodes
                .iter()
                .filter(|(_, n)| n.parent.as_ref() == Some(&cid))
                .map(|(_, n)| (n.x.0, n.y.0, n.width.0, n.height.0))
                .collect();

            compute_subgraph_bounds(children_bounds)
                .and_then(|(x, y, w, h)| {
                    nodes.get(&cid).map(|container| {
                        nodes.update(
                            cid.clone(),
                            Node {
                                x: OrderedFloat::new_unchecked(x - 24.0),
                                y: OrderedFloat::new_unchecked(y - 24.0),
                                width: OrderedFloat::new_unchecked(w + 48.0),
                                height: OrderedFloat::new_unchecked(h + 48.0),
                                ..container.clone()
                            },
                        )
                    })
                })
                .unwrap_or(nodes)
        });
}

fn apply_transform(
    doc: &mut DiagramDocument,
    lock_check: impl Fn(&Node) -> bool,
    transform_fn: impl Fn(&[NodeId], &HashMap<NodeId, Node>) -> HashMap<NodeId, Node>,
) -> Result<(), TransformError> {
    let selected_ids: Vec<NodeId> = doc
        .editor_state
        .selected_items
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .collect();

    if let Some(id) = selected_ids
        .iter()
        .find(|id| doc.document.nodes.get(*id).is_some_and(&lock_check))
    {
        return Err(TransformError::LockedNode(id.clone()));
    }

    doc.document.nodes = transform_fn(&selected_ids, &doc.document.nodes);
    recompute_container_bounds(doc, &selected_ids);
    Ok(())
}

/// Aligns selected nodes along the specified axis.
///
/// # Errors
/// Returns `TransformError::EmptySelection` if fewer than 2 nodes are selected.
/// Returns `TransformError::LockedNode` if any selected node is locked.
pub fn align_selection(
    doc: &mut DiagramDocument,
    axis: &AlignmentAxis,
    mode: &AlignmentMode,
) -> Result<(), TransformError> {
    if doc.editor_state.selected_items.len() < 2 {
        return Err(TransformError::EmptySelection);
    }

    apply_transform(
        doc,
        |n| !n.lock_state.is_movable(&n.kind),
        |selected, nodes| {
            let extents: Vec<_> = selected
                .iter()
                .filter_map(|id| nodes.get(id).map(|n| NodeExtent::from_node(n, *axis)))
                .collect();
            let (min_val, max_val) = extents
                .iter()
                .fold((f64::MAX, f64::MIN), |(min, max), ext| {
                    (min.min(ext.pos), max.max(ext.pos + ext.size))
                });
            let center_val = min_val + (max_val - min_val) / 2.0;

            selected.iter().fold(nodes.clone(), |acc, id| {
                acc.get(id).map_or(acc.clone(), |node| {
                    let ext = NodeExtent::from_node(node, *axis);
                    let pos = match mode {
                        AlignmentMode::Start => min_val,
                        AlignmentMode::Center => center_val - (ext.size / 2.0),
                        AlignmentMode::End => max_val - ext.size,
                    };
                    acc.update(
                        id.clone(),
                        NodeExtent {
                            pos,
                            size: ext.size,
                        }
                        .apply_to(node, *axis),
                    )
                })
            })
        },
    )
}

/// Distributes selected nodes evenly along the specified axis.
///
/// # Errors
/// Returns `TransformError::LockedNode` if any selected node is locked.
pub fn distribute_selection(
    doc: &mut DiagramDocument,
    axis: &AlignmentAxis,
) -> Result<(), TransformError> {
    if doc.editor_state.selected_items.len() < 3 {
        return Ok(());
    }

    apply_transform(
        doc,
        |n| !n.lock_state.is_movable(&n.kind),
        |selected, nodes| {
            let mut sorted: Vec<_> = selected
                .iter()
                .filter_map(|id| nodes.get(id).map(|n| (id, NodeExtent::from_node(n, *axis))))
                .collect();
            sorted.sort_by(|a, b| {
                a.1.pos
                    .partial_cmp(&b.1.pos)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let Some(first_elem) = sorted.first().copied() else {
                return nodes.clone();
            };
            let Some(last_elem) = sorted.last().copied() else {
                return nodes.clone();
            };

            let first_pos = first_elem.1.pos;
            let total_span = (last_elem.1.pos + last_elem.1.size) - first_pos;
            let sum_extents: f64 = sorted.iter().map(|n| n.1.size).sum();
            #[allow(clippy::cast_precision_loss)]
            let spacing = (total_span - sum_extents) / (sorted.len() as f64 - 1.0);

            let (_, updated) =
                sorted
                    .into_iter()
                    .fold((first_pos, nodes.clone()), |(pos, acc), (id, ext)| {
                        (
                            pos + ext.size + spacing,
                            acc.get(id).map_or(acc.clone(), |node| {
                                acc.update(
                                    id.clone(),
                                    NodeExtent {
                                        pos,
                                        size: ext.size,
                                    }
                                    .apply_to(node, *axis),
                                )
                            }),
                        )
                    });
            updated
        },
    )
}

/// Translates selected nodes by `dx` and `dy`.
///
/// # Errors
/// Returns `TransformError::InvalidDelta` if `dx` or `dy` is not finite.
/// Returns `TransformError::EmptySelection` if no nodes are selected.
/// Returns `TransformError::LockedNode` if any selected node is locked.
pub fn translate_selection(
    doc: &mut DiagramDocument,
    dx: f64,
    dy: f64,
) -> Result<(), TransformError> {
    if !dx.is_finite() || !dy.is_finite() {
        return Err(TransformError::InvalidDelta);
    }
    if doc.editor_state.selected_items.is_empty() {
        return Err(TransformError::EmptySelection);
    }

    apply_transform(
        doc,
        |n| n.lock_state.is_locked(),
        |selected, nodes| {
            selected.iter().fold(nodes.clone(), |acc, id| {
                acc.get(id).map_or(acc.clone(), |node| {
                    acc.update(
                        id.clone(),
                        Node {
                            x: OrderedFloat::new_unchecked(node.x.0 + dx),
                            y: OrderedFloat::new_unchecked(node.y.0 + dy),
                            ..node.clone()
                        },
                    )
                })
            })
        },
    )
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
