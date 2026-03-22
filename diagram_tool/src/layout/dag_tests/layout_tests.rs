#![allow(clippy::unwrap_used, clippy::panic, clippy::module_inception, clippy::let_unit_value, clippy::redundant_pattern_matching, unused_variables, unused_imports)]
use crate::layout::dag::apply::apply_position;
use crate::layout::dag::crossing::{barycenter_sweep, barycentre};
use crate::layout::dag::positioning::{assign_coordinates, NODE_HEIGHT, NODE_WIDTH};
use crate::layout::dag::{dag_layout, DagLayoutSettings};
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EditorState, LockState, NodeKind,
    NodeStyle, Revision,
};
use diagram_models::document::{Node, NodeId, OrderedFloat};
use im::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};

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
        lock_state: LockState::Unlocked,
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
        lock_state: LockState::Locked,
        ..make_node(x, y)
    }
}

fn make_edge(src: &NodeId, tgt: &NodeId) -> Edge {
    Edge {
        source: src.clone(),
        target: tgt.clone(),
        label: String::new(),
        style: diagram_models::document::EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: None,
        thickness: OrderedFloat(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
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
#[cfg(kani)]
#[kani::proof]
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
    let get_x = |id| result.document.nodes.get(&id).map_or(0.0, |n| n.x.0);

    let (ax, bx, cx) = (get_x(&a), get_x(&b), get_x(&c));
    assert!(ax < bx, "A.x={ax} must be < B.x={bx}");
    assert!(bx < cx, "B.x={bx} must be < C.x={cx}");
}

// ── Test 2: No edges → no panic, all nodes present ──────────────────────
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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
#[cfg(kani)]
#[kani::proof]
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

    let get_xy = |r: &DiagramDocument, id| r.document.nodes.get(&id).map(|n| (n.x.0, n.y.0));

    assert_eq!(get_xy(&r1, &a), get_xy(&r2, &a), "A must be deterministic");
    assert_eq!(get_xy(&r1, &b), get_xy(&r2, &b), "B must be deterministic");
    assert_eq!(get_xy(&r1, &c), get_xy(&r2, &c), "C must be deterministic");
}
