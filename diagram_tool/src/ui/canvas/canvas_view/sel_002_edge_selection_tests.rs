use im::HashMap;

use super::find_edge_at;
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, LockState, Node, NodeId,
    NodeKind, NodeStyle, OrderedFloat,
};

fn select_single_edge(doc: &mut DiagramDocument, edge_id: EdgeId) {
    doc.editor_state.selected_items = std::iter::once(edge_id.to_string()).collect();
}

fn node_at(x: f64, y: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(10.0),
        height: OrderedFloat(10.0),
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

fn edge(source: NodeId, target: NodeId) -> Edge {
    Edge {
        source,
        target,
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Straight,
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

fn create_document_with_edge() -> DiagramDocument {
    let source_id = NodeId::new(String::from("node-a"));
    let target_id = NodeId::new(String::from("node-b"));
    let edge_id = EdgeId::new(String::from("edge-1"));

    DiagramDocument {
        document: DocumentData {
            nodes: HashMap::new()
                .update(source_id.clone(), node_at(0.0, 0.0))
                .update(target_id.clone(), node_at(100.0, 0.0)),
            edges: HashMap::new().update(edge_id, edge(source_id, target_id)),
        },
        ..DiagramDocument::default()
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_002_given_document_with_two_nodes_and_edge_when_clicking_edge_then_edge_is_selected() {
    let mut doc = create_document_with_edge();
    let hit = find_edge_at(&doc, 50.0, 0.0);

    if let Some(edge_id) = hit {
        assert_eq!(edge_id.as_str(), "edge-1");
        select_single_edge(&mut doc, edge_id.clone());
        assert_eq!(doc.editor_state.selected_items.len(), 1);
        assert!(doc
            .editor_state
            .selected_items
            .contains(&String::from("edge-1")));
    } else {
        panic!("Expected to find edge at click position");
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_002_given_document_with_edge_when_clicking_at_edge_center_then_edge_selected() {
    let doc = create_document_with_edge();
    if let Some(edge_id) = find_edge_at(&doc, 50.0, 0.0) {
        assert_eq!(edge_id.as_str(), "edge-1");
    } else {
        panic!("Expected to find edge at center");
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_002_given_empty_document_when_clicking_then_no_edge_selected() {
    assert!(DiagramDocument::default()
        .editor_state
        .selected_items
        .is_empty());
    assert!(find_edge_at(&DiagramDocument::default(), 50.0, 50.0).is_none());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_002_given_document_with_edge_when_clicking_far_from_edge_then_no_edge_selected() {
    assert!(find_edge_at(&create_document_with_edge(), 500.0, 500.0).is_none());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_002_given_document_when_clicking_with_nan_coordinates_then_no_edge_selected() {
    let doc = create_document_with_edge();
    assert!(find_edge_at(&doc, f64::NAN, 0.0).is_none());
    assert!(find_edge_at(&doc, f64::INFINITY, 0.0).is_none());
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn test_sel_002_given_horizontal_edge_when_clicking_at_endpoint_then_edge_selected() {
    if let Some(edge_id) = find_edge_at(&create_document_with_edge(), 0.0, 0.0) {
        assert_eq!(edge_id.as_str(), "edge-1");
    } else {
        panic!("Expected edge at endpoint");
    }
}
