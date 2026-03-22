use crate::test_utils::builders::edge::EdgeBuilder;
use crate::test_utils::builders::node::NodeBuilder;
use crate::ui::dispatch::errors::DispatchError;
use crate::ui::dispatch::send::edge::*;
use diagram_models::document::{DiagramDocument, EdgeId, NodeId};
use im::{HashMap, HashSet};

fn create_test_doc() -> DiagramDocument {
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();

    // Add some test nodes
    nodes.insert(
        NodeId::new("n1".to_string()),
        NodeBuilder::new(0.0, 0.0, 100.0, 100.0)
            .with_label("Node 1")
            .build(),
    );
    nodes.insert(
        NodeId::new("n2".to_string()),
        NodeBuilder::new(200.0, 0.0, 100.0, 100.0)
            .with_label("Node 2")
            .build(),
    );
    nodes.insert(
        NodeId::new("n3".to_string()),
        NodeBuilder::new(400.0, 0.0, 100.0, 100.0)
            .with_label("Node 3")
            .build(),
    );

    // Add existing edge n2 -> n3
    edges.insert(
        EdgeId::new("e1".to_string()),
        EdgeBuilder::new(NodeId::new("n2".to_string()), NodeId::new("n3".to_string())).build(),
    );

    let mut doc = DiagramDocument::default();
    doc.document.nodes = nodes;
    doc.document.edges = edges;
    doc
}

#[test]
fn given_empty_source_id_when_validating_preconditions_then_returns_edge_not_found() {
    let doc = create_test_doc();
    let result = validate_edge_connect_preconditions(&doc, "", "n2");
    assert!(matches!(result, Err(DispatchError::EdgeNotFound)));
}

#[test]
fn given_missing_source_node_when_validating_preconditions_then_returns_edge_not_found() {
    let doc = create_test_doc();
    let result = validate_edge_connect_preconditions(&doc, "missing", "n2");
    assert!(matches!(result, Err(DispatchError::EdgeNotFound)));
}

#[test]
fn given_self_loop_when_dispatching_edge_connect_then_returns_self_loop_error() {
    let doc = create_test_doc();
    let db_tx = None;

    let result = dispatch_edge_connect(
        &db_tx,
        &doc,
        "new_edge".to_string(),
        "n1".to_string(),
        "n1".to_string(),
    );
    assert!(matches!(result, Err(DispatchError::SelfLoop)));
}

#[test]
fn given_cycle_when_dispatching_edge_connect_then_returns_cycle_detected_error() {
    let doc = create_test_doc();
    let db_tx = None;

    // e1 is n2 -> n3. Adding n3 -> n2 creates a cycle.
    let result = dispatch_edge_connect(
        &db_tx,
        &doc,
        "new_edge".to_string(),
        "n3".to_string(),
        "n2".to_string(),
    );
    assert!(matches!(result, Err(DispatchError::CycleDetected)));
}

#[test]
fn given_valid_edge_without_channel_when_dispatching_edge_connect_then_returns_channel_missing() {
    let doc = create_test_doc();
    let db_tx = None;

    // n1 -> n2 is valid
    let result = dispatch_edge_connect(
        &db_tx,
        &doc,
        "new_edge".to_string(),
        "n1".to_string(),
        "n2".to_string(),
    );
    assert!(matches!(result, Err(DispatchError::ChannelMissing)));
}

#[test]
fn given_unselected_edge_when_dispatching_edge_disconnect_then_returns_not_selected() {
    let doc = create_test_doc();
    let selected = HashSet::new();
    let db_tx = None;

    let result = dispatch_edge_disconnect(&db_tx, &doc, &selected, "e1");
    assert!(matches!(result, Err(DispatchError::NotSelected)));
}

#[test]
fn given_missing_edge_when_dispatching_edge_disconnect_then_returns_edge_not_found() {
    let doc = create_test_doc();
    let mut selected = HashSet::new();
    selected.insert("missing_edge".to_string());
    let db_tx = None;

    let result = dispatch_edge_disconnect(&db_tx, &doc, &selected, "missing_edge");
    assert!(matches!(result, Err(DispatchError::EdgeNotFound)));
}

#[test]
fn given_valid_edge_disconnect_without_channel_then_returns_no_tx() {
    let doc = create_test_doc();
    let mut selected = HashSet::new();
    selected.insert("e1".to_string());
    let db_tx = None;

    let result = dispatch_edge_disconnect(&db_tx, &doc, &selected, "e1");
    assert!(matches!(result, Err(DispatchError::NoTx)));
}
