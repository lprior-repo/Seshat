#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::expect_used, clippy::panic, clippy::unwrap_used, clippy::similar_names, clippy::redundant_clone)]
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentError, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId,
    NodeKind, OrderedFloat, ValidRect,
};
use im::HashMap;

fn create_base_node(_id: &str) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "test".to_string(),
        x: OrderedFloat(0.0),
        y: OrderedFloat(0.0),
        width: OrderedFloat(100.0),
        height: OrderedFloat(100.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    }
}

fn create_test_edge(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
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

#[test]
fn attack_1_self_referential_parent_cycle() {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("node_1".to_string());

    let mut node = create_base_node("node_1");
    // Make node its own parent
    node.parent = Some(node_id.clone());

    doc.document.nodes.insert(node_id.clone(), node.clone());

    // Test infinite loop check. We'll run it in a thread and see if it loops.
    // If it doesn't loop forever, great.
    let _coords = node.get_world_coords(&doc.document.nodes.into_iter().collect());
    // In Rust, using `.into_iter().collect()` turns it into a standard HashMap which it expects
    // Actually the function is get_world_coords_im or get_world_coords
    // let result = node.get_world_coords_im(&doc.document.nodes);
}

#[test]
fn attack_2_deep_parent_chain_stack_overflow() {
    let mut doc = DiagramDocument::default();

    let mut last_id = None;
    // Precondition: ceiling is 1000
    for i in 0..1000 {
        let current_id = NodeId::new(format!("node_{}", i));
        let mut node = create_base_node(&format!("node_{}", i));
        node.parent = last_id.clone();
        doc.document.nodes.insert(current_id.clone(), node);
        last_id = Some(current_id);
    }

    let leaf_node = doc.document.nodes.get(last_id.as_ref().unwrap()).unwrap();
    let coords = leaf_node.get_world_coords_im(&doc.document.nodes);
    assert!(coords.is_ok());
}

#[test]
fn attack_3_marquee_invalid_mode() {
    let mut doc = DiagramDocument::default();
    let _rect = ValidRect::new(0.0, 0.0, 100.0, 100.0).unwrap();

    // Create an invalid struct by bypassing new()
    let evil_rect = ValidRect {
        x: 0.0,
        y: 0.0,
        width: -100.0,
        height: -100.0,
    };

    let _ = doc.select_marquee(
        evil_rect,
        diagram_models::spatial_index::MarqueeMode::Contain,
    );
}

#[test]
fn attack_4_nan_ordered_float() {
    let _res = std::panic::catch_unwind(|| {
        // new_unchecked allows bypassing NaN check
        let _ = OrderedFloat::new_unchecked(f64::NAN);
    });
}

#[test]
fn attack_5_massive_edge_routing() {
    let mut doc = DiagramDocument::default();
    let edge_id = EdgeId::new("edge_1".to_string());
    let edge = create_test_edge("missing_1", "missing_2");

    let result = doc.add_edge(edge_id, edge);
    assert!(matches!(result, Err(DocumentError::NodeNotFound(_))));
}
