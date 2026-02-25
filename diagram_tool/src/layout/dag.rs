#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// usize→f64 precision loss is unavoidable for coordinate maths; allow it explicitly.
#![allow(clippy::cast_precision_loss)]

use crate::layout::grid::calculate_grid_layout;
use crate::models::document::{DiagramDocument, DocumentData, Node, NodeId, OrderedFloat};
use im::HashMap;
use itertools::Itertools;
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};

const NODE_WIDTH: f64 = 220.0;
const NODE_HEIGHT: f64 = 68.0;
const LEFT_PADDING: f64 = 120.0;
const TOP_PADDING: f64 = 80.0;

/// Settings for the Sugiyama-style DAG layout algorithm.
#[derive(Clone, Debug)]
pub struct DagLayoutSettings {
    pub layer_spacing: f64,
    pub node_spacing: f64,
}

impl Default for DagLayoutSettings {
    fn default() -> Self {
        Self {
            layer_spacing: 140.0,
            node_spacing: 60.0,
        }
    }
}

/// Build a `petgraph::DiGraph` from the document's unlocked root nodes and
/// edges.  Returns the graph plus two maps:
///  - `NodeId` → `NodeIndex` (for edge insertion)
///  - a sorted `Vec<NodeId>` of the nodes that participate in layout
fn build_graph(
    doc: &DiagramDocument,
) -> (DiGraph<NodeId, ()>, HashMap<NodeId, NodeIndex>, Vec<NodeId>) {
    let layout_ids: Vec<NodeId> = doc
        .document
        .nodes
        .iter()
        .filter(|(_, n)| !n.locked && n.parent.is_none())
        .map(|(id, _)| id.clone())
        .sorted()
        .collect();

    // pre-compute index map from sorted position
    let id_to_idx: HashMap<NodeId, NodeIndex> = layout_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), NodeIndex::new(i)))
        .collect();

    let graph = layout_ids
        .iter()
        .fold(DiGraph::<NodeId, ()>::new(), |mut g, id| {
            g.add_node(id.clone());
            g
        });

    let graph = doc.document.edges.values().fold(graph, |mut g, edge| {
        if let (Some(&src), Some(&tgt)) = (id_to_idx.get(&edge.source), id_to_idx.get(&edge.target))
        {
            g.add_edge(src, tgt, ());
        }
        g
    });

    (graph, id_to_idx, layout_ids)
}

/// Longest-path layer assignment.  Returns layers as `Vec<Vec<NodeIndex>>`.
fn assign_layers(topo_order: &[NodeIndex], graph: &DiGraph<NodeId, ()>) -> Vec<Vec<NodeIndex>> {
    let layer_map: std::collections::HashMap<NodeIndex, usize> =
        topo_order
            .iter()
            .fold(std::collections::HashMap::new(), |mut m, &idx| {
                let layer = graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .filter_map(|p| m.get(&p).copied())
                    .max()
                    .map_or(0, |max_pred| max_pred + 1);
                m.insert(idx, layer);
                m
            });

    let max_layer = layer_map.values().copied().max().unwrap_or(0);

    (0..=max_layer)
        .map(|layer| {
            topo_order
                .iter()
                .filter(|&&idx| layer_map.get(&idx).copied().unwrap_or(0) == layer)
                .copied()
                .collect()
        })
        .collect()
}

/// Barycentre of `node` neighbours found in `ref_pos`.
fn barycentre(
    node: NodeIndex,
    ref_pos: &std::collections::HashMap<NodeIndex, f64>,
    graph: &DiGraph<NodeId, ()>,
    dir: petgraph::Direction,
) -> f64 {
    let neighbours: Vec<f64> = graph
        .neighbors_directed(node, dir)
        .filter_map(|n| ref_pos.get(&n).copied())
        .collect();

    if neighbours.is_empty() {
        f64::MAX
    } else {
        neighbours.iter().sum::<f64>() / neighbours.len() as f64
    }
}

/// 4-sweep barycentric crossing minimisation.
fn barycenter_sweep(
    layers: Vec<Vec<NodeIndex>>,
    graph: &DiGraph<NodeId, ()>,
) -> Vec<Vec<NodeIndex>> {
    (0..4_u8).fold(layers, |acc, sweep| {
        let n = acc.len();
        if sweep % 2 == 0 {
            // forward: fix layer l-1, reorder l
            (1..n).fold(acc, |mut ls, l| {
                let ref_pos: std::collections::HashMap<NodeIndex, f64> = ls[l - 1]
                    .iter()
                    .enumerate()
                    .map(|(i, &node)| (node, i as f64))
                    .collect();
                ls[l].sort_by(|&a, &b| {
                    barycentre(a, &ref_pos, graph, petgraph::Direction::Incoming)
                        .partial_cmp(&barycentre(
                            b,
                            &ref_pos,
                            graph,
                            petgraph::Direction::Incoming,
                        ))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                ls
            })
        } else {
            // backward: fix layer l+1, reorder l
            (0..n.saturating_sub(1)).rev().fold(acc, |mut ls, l| {
                let ref_pos: std::collections::HashMap<NodeIndex, f64> = ls[l + 1]
                    .iter()
                    .enumerate()
                    .map(|(i, &node)| (node, i as f64))
                    .collect();
                ls[l].sort_by(|&a, &b| {
                    barycentre(a, &ref_pos, graph, petgraph::Direction::Outgoing)
                        .partial_cmp(&barycentre(
                            b,
                            &ref_pos,
                            graph,
                            petgraph::Direction::Outgoing,
                        ))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                ls
            })
        }
    })
}

/// Assign (x, y) coordinates from the layered, ordered structure.
fn assign_coordinates(
    layers: &[Vec<NodeIndex>],
    settings: &DagLayoutSettings,
) -> std::collections::HashMap<NodeIndex, (f64, f64)> {
    let max_layer_size = layers.iter().map(Vec::len).max().unwrap_or(1);
    let canvas_height = (max_layer_size as f64).mul_add(
        NODE_HEIGHT,
        max_layer_size.saturating_sub(1) as f64 * settings.node_spacing,
    );

    layers
        .iter()
        .enumerate()
        .flat_map(|(layer_idx, nodes)| {
            let x = (layer_idx as f64).mul_add(NODE_WIDTH + settings.layer_spacing, LEFT_PADDING);
            let layer_total_height = (nodes.len() as f64).mul_add(
                NODE_HEIGHT,
                nodes.len().saturating_sub(1) as f64 * settings.node_spacing,
            );
            let y_offset = TOP_PADDING + (canvas_height - layer_total_height) / 2.0;

            nodes
                .iter()
                .enumerate()
                .map(move |(pos, &idx)| {
                    let y = (pos as f64).mul_add(NODE_HEIGHT + settings.node_spacing, y_offset);
                    (idx, (x, y))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Pure function: returns a new `DiagramDocument` with node positions updated
/// according to the Sugiyama-style DAG layout.  Falls back to grid layout
/// silently if a cycle is detected.
///
/// Invariants:
/// - Locked nodes are **not** moved.
/// - Child nodes (`parent.is_some()`) move with their parent by the same delta.
/// - The input document is never mutated.
#[must_use]
pub fn dag_layout(doc: &DiagramDocument, settings: &DagLayoutSettings) -> DiagramDocument {
    let (graph, id_to_idx, layout_ids) = build_graph(doc);

    let Ok(topo_order) = toposort(&graph, None) else {
        return calculate_grid_layout(doc, 200.0);
    };

    if layout_ids.is_empty() {
        return doc.clone();
    }

    let layers = assign_layers(&topo_order, &graph);
    let ordered_layers = barycenter_sweep(layers, &graph);
    let coords = assign_coordinates(&ordered_layers, settings);

    // NodeIndex → NodeId reverse map
    let idx_to_id: std::collections::HashMap<NodeIndex, NodeId> = id_to_idx
        .iter()
        .map(|(id, &idx)| (idx, id.clone()))
        .collect();

    // NodeId → new (x, y)
    let new_positions: HashMap<NodeId, (f64, f64)> = coords
        .iter()
        .filter_map(|(&idx, &pos)| idx_to_id.get(&idx).map(|id| (id.clone(), pos)))
        .collect();

    // Deltas for child propagation: only for nodes that were moved
    let deltas: HashMap<NodeId, (f64, f64)> = new_positions
        .iter()
        .filter_map(|(id, &(nx, ny))| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), (nx - node.x.0, ny - node.y.0)))
        })
        .collect();

    let next_nodes: HashMap<NodeId, Node> = doc
        .document
        .nodes
        .iter()
        .map(|(id, node)| {
            let next = apply_position(id, node, &new_positions, &deltas);
            (id.clone(), next)
        })
        .collect();

    DiagramDocument {
        version: doc.version,
        revision: doc.revision.increment(),
        document: DocumentData {
            nodes: next_nodes,
            edges: doc.document.edges.clone(),
        },
        editor_state: doc.editor_state.clone(),
    }
}

/// Compute the new node position:
/// - If it has an entry in `new_positions` → use that (unlocked root node).
/// - Else if it is a child → apply parent delta if parent was moved.
/// - Else (locked root node) → leave unchanged.
fn apply_position(
    id: &NodeId,
    node: &Node,
    new_positions: &HashMap<NodeId, (f64, f64)>,
    deltas: &HashMap<NodeId, (f64, f64)>,
) -> Node {
    if let Some(&(nx, ny)) = new_positions.get(id) {
        return Node {
            x: OrderedFloat(nx),
            y: OrderedFloat(ny),
            ..node.clone()
        };
    }

    let Some(pid) = node.parent.as_ref() else {
        return node.clone(); // locked root → unchanged
    };

    let Some(&(dx, dy)) = deltas.get(pid) else {
        return node.clone(); // parent not moved → unchanged
    };

    Node {
        x: OrderedFloat(node.x.0 + dx),
        y: OrderedFloat(node.y.0 + dy),
        ..node.clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::{
        ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, NodeKind, NodeStyle,
        Revision,
    };
    use im::HashMap;

    fn make_node(x: f64, y: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(220.0),
            height: OrderedFloat(68.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: vec![],
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn make_locked_node(x: f64, y: f64) -> Node {
        Node {
            locked: true,
            ..make_node(x, y)
        }
    }

    fn make_edge(src: &NodeId, tgt: &NodeId) -> Edge {
        Edge {
            source: src.clone(),
            target: tgt.clone(),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: ArrowType::Arrow,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: vec![],
            tags: vec![],
            metadata: HashMap::new(),
            font_size: None,
        }
    }

    fn empty_editor() -> EditorState {
        EditorState {
            snap_to_grid: false,
            ..EditorState::default()
        }
    }

    fn make_doc(nodes: Vec<(NodeId, Node)>, edges: Vec<(EdgeId, Edge)>) -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: nodes.into_iter().collect(),
                edges: edges.into_iter().collect(),
            },
            editor_state: empty_editor(),
        }
    }

    // ── Test 1: A→B→C sequential: A.x < B.x < C.x ──────────────────────────
    #[test]
    fn sequential_dag_x_ordering() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let c = NodeId::new("C".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
                (c.clone(), make_node(0.0, 0.0)),
            ],
            vec![
                (EdgeId::new("e1".to_string()), make_edge(&a, &b)),
                (EdgeId::new("e2".to_string()), make_edge(&b, &c)),
            ],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let get_x = |id: &NodeId| result.document.nodes.get(id).map_or(0.0, |n| n.x.0);

        let (ax, bx, cx) = (get_x(&a), get_x(&b), get_x(&c));
        assert!(ax < bx, "A.x={ax} must be < B.x={bx}");
        assert!(bx < cx, "B.x={bx} must be < C.x={cx}");
    }

    // ── Test 2: No edges → no panic, all nodes present ──────────────────────
    #[test]
    fn no_edges_no_panic() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        assert_eq!(result.document.nodes.len(), 2);
    }

    // ── Test 3: Cycle A→B→A falls back without panic ────────────────────────
    #[test]
    fn cycle_fallback_no_panic() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
            ],
            vec![
                (EdgeId::new("e1".to_string()), make_edge(&a, &b)),
                (EdgeId::new("e2".to_string()), make_edge(&b, &a)),
            ],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        assert_eq!(result.document.nodes.len(), 2);
    }

    // ── Test 4: Locked nodes are not moved ──────────────────────────────────
    #[test]
    fn locked_nodes_unchanged() {
        let locked = NodeId::new("locked".to_string());
        let free = NodeId::new("free".to_string());

        let doc = make_doc(
            vec![
                (locked.clone(), make_locked_node(999.0, 888.0)),
                (free.clone(), make_node(0.0, 0.0)),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let ln = result
            .document
            .nodes
            .get(&locked)
            .expect("locked node must exist");
        assert_eq!(ln.x.0, 999.0, "locked x must not change");
        assert_eq!(ln.y.0, 888.0, "locked y must not change");
    }

    // ── Test 5: Deterministic — two calls on same input produce same result ─
    #[test]
    fn deterministic_output() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let c = NodeId::new("C".to_string());

        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
                (c.clone(), make_node(0.0, 0.0)),
            ],
            vec![
                (EdgeId::new("e1".to_string()), make_edge(&a, &b)),
                (EdgeId::new("e2".to_string()), make_edge(&b, &c)),
            ],
        );

        let r1 = dag_layout(&doc, &DagLayoutSettings::default());
        let r2 = dag_layout(&doc, &DagLayoutSettings::default());

        let get_xy =
            |r: &DiagramDocument, id: &NodeId| r.document.nodes.get(id).map(|n| (n.x.0, n.y.0));

        assert_eq!(get_xy(&r1, &a), get_xy(&r2, &a), "A must be deterministic");
        assert_eq!(get_xy(&r1, &b), get_xy(&r2, &b), "B must be deterministic");
        assert_eq!(get_xy(&r1, &c), get_xy(&r2, &c), "C must be deterministic");
    }
}
