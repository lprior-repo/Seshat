use crate::document::{DiagramDocument, Edge, EdgeId, Node, NodeId, OrderedFloat};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Empty clipboard")]
    EmptyClipboard,
    #[error("Corrupt clipboard")]
    CorruptClipboard,
    #[error("Duplicate ID created")]
    DuplicateIdCreated,
    #[error("Invalid edge reference")]
    InvalidEdgeReference,
    #[error("Invalid parent reference")]
    InvalidParentReference,
    #[error("Cyclic parent reference")]
    CyclicParentReference,
    #[error("Postcondition violated: {0}")]
    PostconditionViolated(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub nodes: Vec<NodeId>,
}

impl Selection {
    #[must_use]
    pub const fn empty() -> Self {
        Self { nodes: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    pub nodes: Vec<(NodeId, Node)>,
    pub edges: Vec<(EdgeId, Edge)>,
    pub paste_serial: u32,
}

impl ClipboardData {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            paste_serial: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct PasteResult {
    pub new_nodes: Vec<(NodeId, Node)>,
    pub new_edges: Vec<(EdgeId, Edge)>,
    pub new_selection: HashSet<String>,
}

pub fn copy(selection: &Selection, doc: &DiagramDocument) -> Result<ClipboardData, Error> {
    if selection.nodes.is_empty() {
        return Err(Error::EmptySelection);
    }

    let selected_set: HashSet<_> = selection.nodes.iter().collect();

    let nodes = selection
        .nodes
        .iter()
        .map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.clone()))
                .ok_or_else(|| {
                    Error::PostconditionViolated(format!(
                        "Selected node {id} not found in document"
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_set.contains(&edge.source) && selected_set.contains(&edge.target)
        })
        .map(|(id, edge)| (id.clone(), edge.clone()))
        .collect();

    Ok(ClipboardData {
        nodes,
        edges,
        paste_serial: 0,
    })
}

pub fn cut(selection: &Selection, doc: &mut DiagramDocument) -> Result<ClipboardData, Error> {
    let clipboard = copy(selection, doc)?;

    selection.nodes.iter().for_each(|id| {
        let _ = doc.remove_node(id);
        doc.editor_state.selected_items.remove(id.as_str());
    });

    Ok(clipboard)
}

pub fn calculate_paste(
    clipboard: &ClipboardData,
    doc: &DiagramDocument,
) -> Result<PasteResult, Error> {
    if clipboard.nodes.is_empty() {
        return Err(Error::EmptyClipboard);
    }

    let mut clipboard_node_ids = HashSet::new();
    for (id, _) in &clipboard.nodes {
        if !clipboard_node_ids.insert(id.clone()) {
            return Err(Error::CorruptClipboard);
        }
    }
    let mut clipboard_edge_ids = HashSet::new();
    for (id, _) in &clipboard.edges {
        if !clipboard_edge_ids.insert(id.clone()) {
            return Err(Error::CorruptClipboard);
        }
    }

    let offset_val = 20.0 * f64::from(clipboard.paste_serial + 1);

    let new_nodes_mapped = clipboard
        .nodes
        .iter()
        .map(|(old_id, node)| {
            let new_id = NodeId::new(Uuid::new_v4().to_string());
            if doc.document.nodes.contains_key(&new_id) {
                return Err(Error::DuplicateIdCreated);
            }
            Ok((
                old_id.clone(),
                new_id,
                Node {
                    x: OrderedFloat::new_unchecked(node.x.0 + offset_val),
                    y: OrderedFloat::new_unchecked(node.y.0 + offset_val),
                    ..node.clone()
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let id_map: HashMap<NodeId, NodeId> = new_nodes_mapped
        .iter()
        .map(|(old_id, new_id, _)| (old_id.clone(), new_id.clone()))
        .collect();

    let clipboard_nodes_map: HashMap<NodeId, &Node> = clipboard
        .nodes
        .iter()
        .map(|(id, n)| (id.clone(), n))
        .collect();

    let mut path = HashSet::new();
    for (start_id, _) in &clipboard.nodes {
        let mut current = start_id.clone();
        path.clear();
        while let Some(parent_id) = clipboard_nodes_map
            .get(&current)
            .and_then(|n| n.parent.as_ref())
        {
            if path.contains(parent_id) || parent_id == start_id {
                return Err(Error::CyclicParentReference);
            }
            path.insert(parent_id.clone());
            current = parent_id.clone();
        }
    }

    let mapped_nodes = new_nodes_mapped
        .into_iter()
        .map(|(_, new_id, node)| {
            let remapped_parent = match node.parent.as_ref() {
                Some(parent) => {
                    if let Some(new_parent) = id_map.get(parent) {
                        Some(new_parent.clone())
                    } else if doc.document.nodes.contains_key(parent) {
                        Some(parent.clone())
                    } else {
                        return Err(Error::InvalidParentReference);
                    }
                }
                None => None,
            };
            Ok((
                new_id,
                Node {
                    parent: remapped_parent,
                    ..node
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let new_edges_mapped = clipboard
        .edges
        .iter()
        .map(|(_, edge)| {
            let is_remapped_source = id_map.contains_key(&edge.source);
            let is_remapped_target = id_map.contains_key(&edge.target);

            let new_source = if is_remapped_source {
                id_map.get(&edge.source).unwrap_or(&edge.source).clone()
            } else {
                edge.source.clone()
            };

            let new_target = if is_remapped_target {
                id_map.get(&edge.target).unwrap_or(&edge.target).clone()
            } else {
                edge.target.clone()
            };

            if !is_remapped_source && !doc.document.nodes.contains_key(&new_source) {
                return Err(Error::InvalidEdgeReference);
            }
            if !is_remapped_target && !doc.document.nodes.contains_key(&new_target) {
                return Err(Error::InvalidEdgeReference);
            }

            let new_edge_id = EdgeId::new(Uuid::new_v4().to_string());
            if doc.document.edges.contains_key(&new_edge_id) {
                return Err(Error::DuplicateIdCreated);
            }

            Ok((
                new_edge_id,
                Edge {
                    source: new_source,
                    target: new_target,
                    ..edge.clone()
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let new_selection: HashSet<String> = mapped_nodes
        .iter()
        .map(|(id, _)| id.as_str().to_string())
        .collect();

    Ok(PasteResult {
        new_nodes: mapped_nodes,
        new_edges: new_edges_mapped,
        new_selection,
    })
}
