use crate::export::svg::generate_svg_string;
use diagram_models::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, EditorState, LockState,
    Node, NodeId, NodeKind, OrderedFloat, Revision,
};
use im::HashMap;

fn create_test_document() -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        },
        editor_state: diagram_models::document::EditorState::default(),
    }
}

fn create_test_node(id: &str, x: f64, y: f64, width: f64, height: f64) -> Node {
    Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: "Node".to_string(),
        x: OrderedFloat(x),
        y: OrderedFloat(y),
        width: OrderedFloat(width),
        height: OrderedFloat(height),
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

#[test]
fn test_export_document_with_one_edge_produces_valid_svg_path() {
    // Given
    let mut doc = create_test_document();

    // Create source node (0, 0, 100, 100) -> Center is (50, 50)
    let src_id = NodeId::new("n1".to_string());
    doc.document.nodes.insert(
        src_id.clone(),
        create_test_node("n1", 0.0, 0.0, 100.0, 100.0),
    );

    // Create target node (200, 200, 100, 100) -> Center is (250, 250)
    let tgt_id = NodeId::new("n2".to_string());
    doc.document.nodes.insert(
        tgt_id.clone(),
        create_test_node("n2", 200.0, 200.0, 100.0, 100.0),
    );

    // Create edge connecting them
    let edge_id = EdgeId::new("e1".to_string());
    let edge = Edge {
        source: src_id,
        target: tgt_id,
        label: String::new(),
        style: EdgeStyle::Solid,
        arrow_type: ArrowType::Default,
        label_offset_t: OrderedFloat(0.5),
        color: Some("#ff0000".to_string()),
        thickness: OrderedFloat(2.0),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    };
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then
    assert!(svg.contains("<svg xmlns='http://www.w3.org/2000/svg'"));
    // The edge should be drawn from center to center: (50, 50) to (250, 250)
    assert!(svg
        .contains("<line x1='50' y1='50' x2='250' y2='250' stroke='#ff0000' stroke-width='2' />"));
}

#[test]
fn test_export_document_with_missing_nodes_does_not_panic() {
    // Given
    let mut doc = create_test_document();

    // Create an edge that references missing nodes
    let edge_id = EdgeId::new("e1".to_string());
    let edge = Edge {
        source: NodeId::new("missing1".to_string()),
        target: NodeId::new("missing2".to_string()),
        label: String::new(),
        style: EdgeStyle::Solid,
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
    };
    doc.document.edges.insert(edge_id, edge);

    // When
    let svg = generate_svg_string(&doc);

    // Then - should successfully generate SVG without the invalid edge and not panic
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("<line"));
}

#[test]
fn test_export_document_with_zero_edges_produces_empty_svg() {
    // Given
    let doc = create_test_document();

    // When
    let svg = generate_svg_string(&doc);

    // Then
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("<line"));
}
