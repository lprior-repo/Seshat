use crate::geometry::Point;
use crate::history::History;
use crate::models::document::{
    ArrowType, DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle,
    OrderedFloat,
};
use crate::ui::commands::{
    apply_delete_selected, apply_redo, apply_undo, apply_zoom_in, apply_zoom_out,
};
use dioxus::prelude::*;
use im::{HashMap, HashSet};

fn create_test_node(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Text,
        icon: String::new(),
        label: String::from("Test Node"),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(100.0),
        height: OrderedFloat(24.0),
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

fn create_test_edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: ArrowType::default(),
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
    }
}

#[test]
fn test_doc_003_deleting_node_removes_incident_edges() {
    let mut doc = DiagramDocument::default();

    // Add two nodes
    let node1_id = NodeId::new("node1".to_string());
    let node2_id = NodeId::new("node2".to_string());

    doc.document
        .nodes
        .insert(node1_id.clone(), create_test_node(0.0, 0.0));
    doc.document
        .nodes
        .insert(node2_id.clone(), create_test_node(100.0, 0.0));

    // Add edge between them
    let edge_id = EdgeId::new("edge1".to_string());
    doc.document.edges.insert(
        edge_id.clone(),
        create_test_edge(node1_id.clone(), node2_id.clone()),
    );

    // Select node1
    doc.editor_state
        .selected_items
        .insert(node1_id.as_str().to_string());

    // Call delete command
    // We mock the Dioxus signals inside a test closure
    let doc_signal = Signal::new(doc);
    let history_signal = Signal::new(History::new());

    let result = apply_delete_selected(doc_signal, history_signal, None);
    assert!(result);

    let next_doc = doc_signal.read();
    assert_eq!(next_doc.document.nodes.len(), 1);
    assert_eq!(
        next_doc.document.edges.len(),
        0,
        "Incident edge should be deleted"
    );
    assert!(next_doc.document.nodes.contains_key(&node2_id));
}

#[test]
fn test_doc_006_zoom_commands_remain_clamped() {
    let doc = DiagramDocument::default();

    let doc_signal = Signal::new(doc);
    let history_signal = Signal::new(History::new());
    let viewport = (1000.0, 1000.0);

    // Zoom in wildly
    for _ in 0..50 {
        apply_zoom_in(doc_signal, history_signal, viewport);
    }

    assert!(doc_signal.read().editor_state.zoom.0 <= 4.0);
    assert!(doc_signal.read().editor_state.zoom.0 > 1.0);

    // Zoom out wildly
    for _ in 0..100 {
        apply_zoom_out(doc_signal, history_signal, viewport);
    }

    assert!(doc_signal.read().editor_state.zoom.0 >= 0.1);
}

#[test]
fn test_doc_004_undo_redo_roundtrips_mutation_state() {
    let mut doc = DiagramDocument::default();

    let node1_id = NodeId::new("node1".to_string());
    doc.document
        .nodes
        .insert(node1_id.clone(), create_test_node(0.0, 0.0));
    doc.editor_state
        .selected_items
        .insert(node1_id.as_str().to_string());

    let doc_signal = Signal::new(doc.clone());
    let history_signal = Signal::new(History::new());

    // Perform delete
    let deleted = apply_delete_selected(doc_signal, history_signal, None);
    assert!(deleted);
    assert_eq!(doc_signal.read().document.nodes.len(), 0);

    // Undo
    apply_undo(doc_signal, history_signal, None);
    assert_eq!(
        doc_signal.read().document.nodes.len(),
        1,
        "Undo should restore node"
    );

    // Redo
    apply_redo(doc_signal, history_signal, None);
    assert_eq!(
        doc_signal.read().document.nodes.len(),
        0,
        "Redo should delete node again"
    );
}
