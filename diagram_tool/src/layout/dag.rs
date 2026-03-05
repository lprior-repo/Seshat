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
    [true, false, true, false]
        .into_iter()
        .fold(layers, |acc, is_forward| {
            let n = acc.len();
            if is_forward {
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
            let next = apply_position(id, node, &new_positions, &deltas, &doc.document.nodes);
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
    all_nodes: &HashMap<NodeId, Node>,
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

    let inherited_delta = std::iter::successors(Some(pid.clone()), |parent_id| {
        all_nodes
            .get(parent_id)
            .and_then(|parent| parent.parent.clone())
    })
    .take(all_nodes.len())
    .fold(None, |acc: Option<(f64, f64)>, parent_id| {
        deltas.get(&parent_id).map_or(acc, |&(dx, dy)| {
            Some(match acc {
                Some((adx, ady)) => (adx + dx, ady + dy),
                None => (dx, dy),
            })
        })
    });

    let Some((dx, dy)) = inherited_delta else {
        return node.clone(); // parent chain not moved → unchanged
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
            tags: im::Vector::new(),
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
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
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
            vec![(a, make_node(0.0, 0.0)), (b, make_node(0.0, 0.0))],
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
                (free, make_node(0.0, 0.0)),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        assert!(result.document.nodes.contains_key(&locked));
        let Some(ln) = result.document.nodes.get(&locked) else {
            return;
        };
        assert!(
            (ln.x.0 - 999.0).abs() < f64::EPSILON,
            "locked x must not change"
        );
        assert!(
            (ln.y.0 - 888.0).abs() < f64::EPSILON,
            "locked y must not change"
        );
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

    #[test]
    fn nested_children_follow_ancestor_delta() {
        let root = NodeId::new("root".to_string());
        let child = NodeId::new("child".to_string());
        let grandchild = NodeId::new("grandchild".to_string());

        let mut child_node = make_node(10.0, 10.0);
        child_node.parent = Some(root.clone());

        let mut grandchild_node = make_node(20.0, 20.0);
        grandchild_node.parent = Some(child.clone());

        let doc = make_doc(
            vec![
                (root.clone(), make_node(0.0, 0.0)),
                (child.clone(), child_node),
                (grandchild.clone(), grandchild_node),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());

        let root_before = doc
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let root_after = result
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let delta = (root_after.0 - root_before.0, root_after.1 - root_before.1);

        let grandchild_before = doc
            .document
            .nodes
            .get(&grandchild)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let grandchild_after = result
            .document
            .nodes
            .get(&grandchild)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        assert!((grandchild_after.0 - (grandchild_before.0 + delta.0)).abs() < f64::EPSILON);
        assert!((grandchild_after.1 - (grandchild_before.1 + delta.1)).abs() < f64::EPSILON);
    }

    #[test]
    fn given_two_unconnected_nodes_when_dag_layout_then_nodes_are_centered_in_single_layer() {
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
        let a_pos = result
            .document
            .nodes
            .get(&a)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let b_pos = result
            .document
            .nodes
            .get(&b)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        let y_values = [a_pos.1, b_pos.1];

        assert_eq!(a_pos.0, 120.0);
        assert_eq!(b_pos.0, 120.0);
        assert!(y_values.contains(&80.0));
        assert!(y_values.contains(&208.0));
    }

    #[test]
    fn given_single_edge_when_dag_layout_then_layer_spacing_and_padding_are_applied() {
        let a = NodeId::new("A".to_string());
        let b = NodeId::new("B".to_string());
        let doc = make_doc(
            vec![
                (a.clone(), make_node(0.0, 0.0)),
                (b.clone(), make_node(0.0, 0.0)),
            ],
            vec![(EdgeId::new("e1".to_string()), make_edge(&a, &b))],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let a_pos = result
            .document
            .nodes
            .get(&a)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let b_pos = result
            .document
            .nodes
            .get(&b)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        assert_eq!(a_pos, (120.0, 80.0));
        assert_eq!(b_pos, (480.0, 80.0));
    }

    #[test]
    fn given_node_without_neighbors_when_barycentre_computed_then_result_is_max() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let isolated = graph.add_node(NodeId::new(String::from("isolated")));
        let ref_pos = std::collections::HashMap::new();

        let value = barycentre(isolated, &ref_pos, &graph, petgraph::Direction::Incoming);
        assert_eq!(value, f64::MAX);
    }

    #[test]
    fn given_neighbor_positions_when_barycentre_computed_then_mean_is_used() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let a = graph.add_node(NodeId::new(String::from("a")));
        let b = graph.add_node(NodeId::new(String::from("b")));
        let c = graph.add_node(NodeId::new(String::from("c")));
        graph.add_edge(a, c, ());
        graph.add_edge(b, c, ());

        let ref_pos = std::collections::HashMap::from([(a, 2.0), (b, 6.0)]);
        let value = barycentre(c, &ref_pos, &graph, petgraph::Direction::Incoming);
        assert_eq!(value, 4.0);
    }

    #[test]
    fn given_mixed_layer_sizes_when_assigning_coordinates_then_shorter_layers_are_centered() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let n0 = graph.add_node(NodeId::new(String::from("n0")));
        let n1 = graph.add_node(NodeId::new(String::from("n1")));
        let n2 = graph.add_node(NodeId::new(String::from("n2")));
        let layers = vec![vec![n0, n1], vec![n2]];

        let coords = assign_coordinates(&layers, &DagLayoutSettings::default());
        let y0 = coords.get(&n0).map_or(0.0, |(_, y)| *y);
        let y1 = coords.get(&n1).map_or(0.0, |(_, y)| *y);
        let y2 = coords.get(&n2).map_or(0.0, |(_, y)| *y);

        assert_eq!(y0, 80.0);
        assert_eq!(y1, 208.0);
        assert_eq!(y2, 144.0);
    }

    #[test]
    fn given_ancestor_deltas_when_applying_position_then_deltas_are_accumulated() {
        let root_id = NodeId::new(String::from("root"));
        let child_id = NodeId::new(String::from("child"));
        let grandchild_id = NodeId::new(String::from("grandchild"));

        let mut child = make_node(5.0, 10.0);
        child.parent = Some(root_id.clone());
        let mut grandchild = make_node(20.0, 30.0);
        grandchild.parent = Some(child_id.clone());

        let all_nodes = HashMap::new()
            .update(root_id.clone(), make_node(0.0, 0.0))
            .update(child_id.clone(), child)
            .update(grandchild_id.clone(), grandchild.clone());

        let deltas = HashMap::new()
            .update(root_id, (10.0, 20.0))
            .update(child_id, (5.0, 7.0));

        let moved = apply_position(
            &grandchild_id,
            &grandchild,
            &HashMap::new(),
            &deltas,
            &all_nodes,
        );
        assert_eq!(moved.x.0, 35.0);
        assert_eq!(moved.y.0, 57.0);
    }

    #[test]
    fn given_nonzero_root_origin_when_dag_layout_runs_then_child_follows_exact_root_delta() {
        let root = NodeId::new(String::from("root"));
        let child = NodeId::new(String::from("child"));

        let mut child_node = make_node(70.0, 90.0);
        child_node.parent = Some(root.clone());

        let doc = make_doc(
            vec![
                (root.clone(), make_node(50.0, 70.0)),
                (child.clone(), child_node),
            ],
            vec![],
        );

        let result = dag_layout(&doc, &DagLayoutSettings::default());
        let root_before = doc
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let root_after = result
            .document
            .nodes
            .get(&root)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let child_before = doc
            .document
            .nodes
            .get(&child)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let child_after = result
            .document
            .nodes
            .get(&child)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let delta = (root_after.0 - root_before.0, root_after.1 - root_before.1);

        assert_eq!(child_after.0, child_before.0 + delta.0);
        assert_eq!(child_after.1, child_before.1 + delta.1);
    }

    #[test]
    fn given_crossed_two_layer_order_when_swept_then_barycenter_reorders_layer() {
        let mut graph = DiGraph::<NodeId, ()>::new();
        let a = graph.add_node(NodeId::new(String::from("a")));
        let b = graph.add_node(NodeId::new(String::from("b")));
        let c = graph.add_node(NodeId::new(String::from("c")));
        let d = graph.add_node(NodeId::new(String::from("d")));

        graph.add_edge(a, d, ());
        graph.add_edge(b, c, ());

        let layers = vec![vec![a, b], vec![c, d]];
        let swept = barycenter_sweep(layers, &graph);

        assert_eq!(swept.len(), 2);
        assert_eq!(swept[0], vec![a, b]);
        assert_eq!(swept[1], vec![d, c]);
    }

    #[test]
    fn given_multi_layer_graph_when_swept_then_matches_reference_sweep_order() {
        fn barycentre_ref(
            node: NodeIndex,
            ref_pos: &std::collections::HashMap<NodeIndex, f64>,
            graph: &DiGraph<NodeId, ()>,
            dir: petgraph::Direction,
        ) -> f64 {
            let neighbors = graph
                .neighbors_directed(node, dir)
                .filter_map(|n| ref_pos.get(&n).copied())
                .collect::<Vec<_>>();

            if neighbors.is_empty() {
                f64::MAX
            } else {
                neighbors.iter().sum::<f64>() / neighbors.len() as f64
            }
        }

        fn reference_sweep(
            layers: Vec<Vec<NodeIndex>>,
            graph: &DiGraph<NodeId, ()>,
        ) -> Vec<Vec<NodeIndex>> {
            (0..4_u8).fold(layers, |acc, sweep| {
                let n = acc.len();
                if sweep % 2 == 0 {
                    (1..n).fold(acc, |mut ls, l| {
                        let ref_pos = ls[l - 1]
                            .iter()
                            .enumerate()
                            .map(|(i, &node)| (node, i as f64))
                            .collect::<std::collections::HashMap<_, _>>();
                        ls[l].sort_by(|&a, &b| {
                            barycentre_ref(a, &ref_pos, graph, petgraph::Direction::Incoming)
                                .partial_cmp(&barycentre_ref(
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
                    (0..n.saturating_sub(1)).rev().fold(acc, |mut ls, l| {
                        let ref_pos = ls[l + 1]
                            .iter()
                            .enumerate()
                            .map(|(i, &node)| (node, i as f64))
                            .collect::<std::collections::HashMap<_, _>>();
                        ls[l].sort_by(|&a, &b| {
                            barycentre_ref(a, &ref_pos, graph, petgraph::Direction::Outgoing)
                                .partial_cmp(&barycentre_ref(
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

        let mut graph = DiGraph::<NodeId, ()>::new();
        let a = graph.add_node(NodeId::new(String::from("a")));
        let b = graph.add_node(NodeId::new(String::from("b")));
        let c = graph.add_node(NodeId::new(String::from("c")));
        let d = graph.add_node(NodeId::new(String::from("d")));
        let e = graph.add_node(NodeId::new(String::from("e")));
        let f = graph.add_node(NodeId::new(String::from("f")));

        graph.add_edge(a, c, ());
        graph.add_edge(a, d, ());
        graph.add_edge(b, d, ());
        graph.add_edge(b, e, ());
        graph.add_edge(c, f, ());
        graph.add_edge(e, f, ());

        let layers = vec![vec![a, b], vec![e, d, c], vec![f]];
        let expected = reference_sweep(layers.clone(), &graph);
        let actual = barycenter_sweep(layers, &graph);

        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::models::document::{
        ArrowType, Edge, EdgeId, EditorState, NodeKind, NodeStyle, Revision,
    };
    use proptest::prelude::*;

    fn make_node_for_prop(x: f64, y: f64) -> Node {
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
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn make_edge_for_prop(src: &NodeId, tgt: &NodeId) -> Edge {
        Edge {
            source: src.clone(),
            target: tgt.clone(),
            label: String::new(),
            style: crate::models::document::EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: im::vector![],
            tags: im::vector![],
            metadata: im::HashMap::new(),
            font_size: None,
        }
    }

    fn make_doc_for_prop(
        nodes: Vec<(NodeId, Node)>,
        edges: Vec<(EdgeId, Edge)>,
    ) -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: nodes.into_iter().collect(),
                edges: edges.into_iter().collect(),
            },
            editor_state: EditorState::default(),
        }
    }

    prop_compose! {
        fn arb_dag_layout_settings()(
            layer_spacing in 1.0..1000.0f64,
            node_spacing in 1.0..500.0f64,
        ) -> DagLayoutSettings {
            DagLayoutSettings { layer_spacing, node_spacing }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_all_coordinates_finite(
            node_count in 1usize..20,
            settings in arb_dag_layout_settings(),
        ) {
            let nodes: Vec<(NodeId, Node)> = (0..node_count)
                .map(|i| (NodeId::new(format!("n{i}")), make_node_for_prop(0.0, 0.0)))
                .collect();

            let doc = make_doc_for_prop(nodes, vec![]);
            let result = dag_layout(&doc, &settings);

            for node in result.document.nodes.values() {
                prop_assert!(node.x.0.is_finite(), "x coordinate must be finite");
                prop_assert!(node.y.0.is_finite(), "y coordinate must be finite");
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_layer_ordering_respected(
            edge_count in 0usize..10,
            settings in arb_dag_layout_settings(),
        ) {
            let a = NodeId::new("a".to_string());
            let b = NodeId::new("b".to_string());
            let c = NodeId::new("c".to_string());

            let mut edges: Vec<(EdgeId, Edge)> = vec![];
            if edge_count % 3 == 0 {
                edges.push((EdgeId::new("e1".to_string()), make_edge_for_prop(&a, &b)));
            }
            if edge_count % 3 == 1 {
                edges.push((EdgeId::new("e2".to_string()), make_edge_for_prop(&b, &c)));
            }
            if edge_count % 3 == 2 {
                edges.push((EdgeId::new("e3".to_string()), make_edge_for_prop(&a, &c)));
            }

            let doc = make_doc_for_prop(
                vec![
                    (a.clone(), make_node_for_prop(0.0, 0.0)),
                    (b.clone(), make_node_for_prop(0.0, 0.0)),
                    (c.clone(), make_node_for_prop(0.0, 0.0)),
                ],
                edges,
            );

            let result = dag_layout(&doc, &settings);

            let get_x = |id: &NodeId| result.document.nodes.get(id).map_or(0.0, |n| n.x.0);

            if result.document.edges.contains_key(&EdgeId::new("e1".to_string())) {
                let (ax, bx) = (get_x(&a), get_x(&b));
                prop_assert!(ax <= bx, "a.x={ax} must be <= b.x={bx} when a→b");
            }
            if result.document.edges.contains_key(&EdgeId::new("e2".to_string())) {
                let (bx, cx) = (get_x(&b), get_x(&c));
                prop_assert!(bx <= cx, "b.x={bx} must be <= c.x={cx} when b→c");
            }
            if result.document.edges.contains_key(&EdgeId::new("e3".to_string())) {
                let (ax, cx) = (get_x(&a), get_x(&c));
                prop_assert!(ax <= cx, "a.x={ax} must be <= c.x={cx} when a→c");
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_no_node_overlap(
            node_count in 2usize..15,
            settings in arb_dag_layout_settings(),
        ) {
            let nodes: Vec<(NodeId, Node)> = (0..node_count)
                .map(|i| (NodeId::new(format!("n{i}")), make_node_for_prop(0.0, 0.0)))
                .collect();

            let doc = make_doc_for_prop(nodes, vec![]);
            let result = dag_layout(&doc, &settings);

            let positions: Vec<(&NodeId, f64, f64)> = result
                .document
                .nodes
                .iter()
                .filter(|(_, n)| !n.locked && n.parent.is_none())
                .map(|(id, n)| (id, n.x.0, n.y.0))
                .collect();

            for i in 0..positions.len() {
                for j in (i + 1)..positions.len() {
                    let (_, x1, y1) = positions[i];
                    let (_, x2, y2) = positions[j];
                    let dx = (x1 - x2).abs();
                    let dy = (y1 - y2).abs();
                    let overlapping = dx < NODE_WIDTH && dy < NODE_HEIGHT;
                    prop_assert!(!overlapping, "nodes at ({x1},{y1}) and ({x2},{y2}) overlap");
                }
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_edge_endpoints_valid(
            node_count in 2usize..10,
            edge_count in 0usize..9,
            settings in arb_dag_layout_settings(),
        ) {
            let node_ids: Vec<NodeId> = (0..node_count)
                .map(|i| NodeId::new(format!("n{i}")))
                .collect();

            let nodes: Vec<(NodeId, Node)> = node_ids
                .iter()
                .map(|id| (id.clone(), make_node_for_prop(0.0, 0.0)))
                .collect();

            let edges: Vec<(EdgeId, Edge)> = (0..edge_count.min(node_count - 1))
                .map(|i| {
                    let src = node_ids[i].clone();
                    let tgt = node_ids[i + 1].clone();
                    (EdgeId::new(format!("e{i}")), make_edge_for_prop(&src, &tgt))
                })
                .collect();

            let doc = make_doc_for_prop(nodes, edges.clone());
            let result = dag_layout(&doc, &settings);

            for (edge_id, edge) in &result.document.edges {
                prop_assert!(
                    result.document.nodes.contains_key(&edge.source),
                    "edge {edge_id:?} source must exist"
                );
                prop_assert!(
                    result.document.nodes.contains_key(&edge.target),
                    "edge {edge_id:?} target must exist"
                );
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_extreme_coordinates_no_panic(
            x in f64::MIN..f64::MAX,
            y in f64::MIN..f64::MAX,
            settings in arb_dag_layout_settings(),
        ) {
            let x = if x.is_nan() { 0.0 } else { x };
            let y = if y.is_nan() { 0.0 } else { y };

            let doc = make_doc_for_prop(
                vec![(NodeId::new("n".to_string()), make_node_for_prop(x, y))],
                vec![],
            );

            let result = dag_layout(&doc, &settings);

            for node in result.document.nodes.values() {
                if !node.locked {
                    prop_assert!(node.x.0.is_finite(), "result x must be finite");
                    prop_assert!(node.y.0.is_finite(), "result y must be finite");
                }
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_node_count_preserved(
            node_count in 0usize..50,
            settings in arb_dag_layout_settings(),
        ) {
            let nodes: Vec<(NodeId, Node)> = (0..node_count)
                .map(|i| (NodeId::new(format!("n{i}")), make_node_for_prop(0.0, 0.0)))
                .collect();

            let doc = make_doc_for_prop(nodes, vec![]);
            let result = dag_layout(&doc, &settings);

            prop_assert_eq!(result.document.nodes.len(), node_count);
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_edge_count_preserved(
            node_count in 2usize..20,
            edge_count in 0usize..19,
            settings in arb_dag_layout_settings(),
        ) {
            let node_ids: Vec<NodeId> = (0..node_count)
                .map(|i| NodeId::new(format!("n{i}")))
                .collect();

            let nodes: Vec<(NodeId, Node)> = node_ids
                .iter()
                .map(|id| (id.clone(), make_node_for_prop(0.0, 0.0)))
                .collect();

            let edges: Vec<(EdgeId, Edge)> = (0..edge_count.min(node_count - 1))
                .map(|i| {
                    let src = node_ids[i % node_ids.len()].clone();
                    let tgt = node_ids[(i + 1) % node_ids.len()].clone();
                    (EdgeId::new(format!("e{i}")), make_edge_for_prop(&src, &tgt))
                })
                .collect();

            let doc = make_doc_for_prop(nodes, edges.clone());
            let result = dag_layout(&doc, &settings);

            prop_assert_eq!(result.document.edges.len(), edges.len());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_empty_document_no_panic(settings in arb_dag_layout_settings()) {
            let doc = make_doc_for_prop(vec![], vec![]);
            let result = dag_layout(&doc, &settings);
            prop_assert!(result.document.nodes.is_empty());
            prop_assert!(result.document.edges.is_empty());
        }
    }
}
