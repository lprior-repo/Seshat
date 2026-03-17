//! DiagramDocument Serialization Tests (IO-001 to IO-003)
//!
//! This module contains tests for JSON string serialization of DiagramDocument.
//! Focus strictly on string serialization - no file system operations.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::document::{
    ArrowType, DiagramDocument, DocumentData, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind,
    OrderedFloat, Revision,
};
use im::HashMap;

fn create_minimal_document() -> DiagramDocument {
    DiagramDocument {
        version: 2,
        revision: Revision::INITIAL,
        document: DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        },
        editor_state: Default::default(),
    }
}

fn create_document_with_node() -> DiagramDocument {
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId::new("node-1".to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Test Node".to_string(),
            x: OrderedFloat(100.0),
            y: OrderedFloat(200.0),
            width: OrderedFloat(150.0),
            height: OrderedFloat(80.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        },
    );

    DiagramDocument {
        version: 2,
        revision: Revision::new(1),
        document: DocumentData { nodes, edges: HashMap::new() },
        editor_state: Default::default(),
    }
}

// ============================================================================
// IO-001: DiagramDocument JSON Deserialization (Import)
// ============================================================================

#[test]
fn io_001_empty_document_json_parse() {
    let json = r#"{
        "version": 2,
        "revision": {"value": 0},
        "document": {
            "nodes": {},
            "edges": {}
        },
        "editor_state": {}
    }"#;

    let result: Result<DiagramDocument, _> = serde_json::from_str(json);
    assert!(result.is_ok(), "Should parse valid empty document JSON");
    
    let doc = result.unwrap();
    assert_eq!(doc.version, 2);
    assert_eq!(doc.revision.value(), 0);
    assert!(doc.document.nodes.is_empty());
    assert!(doc.document.edges.is_empty());
}

#[test]
fn io_001_malformed_json_syntax_error() {
    let malformed_json = r#"{"version": 2, "broken": [}"#;
    let result: Result<DiagramDocument, _> = serde_json::from_str(malformed_json);
    assert!(result.is_err(), "Should fail on malformed JSON syntax");
}

#[test]
fn io_001_missing_required_field_version() {
    let json = r#"{
        "revision": {"value": 0},
        "document": {"nodes": {}, "edges": {}}
    }"#;
    let result: Result<DiagramDocument, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Should fail when version field is missing");
}

// ============================================================================
// IO-002: DiagramDocument JSON Serialization (Export)
// ============================================================================

#[test]
fn io_002_empty_document_serialize() {
    let doc = create_minimal_document();
    let result = serde_json::to_string(&doc);
    assert!(result.is_ok(), "Should serialize empty document");
    
    let json = result.unwrap();
    assert!(json.contains("\"version\":2"), "Should contain version");
    assert!(json.contains("\"nodes\":{}"), "Should contain empty nodes");
    assert!(json.contains("\"edges\":{}"), "Should contain empty edges");
}

#[test]
fn io_002_document_with_node_serialize() {
    let doc = create_document_with_node();
    let result = serde_json::to_string(&doc);
    assert!(result.is_ok(), "Should serialize document with node");
    
    let json = result.unwrap();
    assert!(json.contains("node-1"), "Should contain node ID");
    assert!(json.contains("Test Node"), "Should contain node label");
    assert!(json.contains("100"), "Should contain x coordinate");
}

#[test]
fn io_002_serialization_idempotent() {
    let doc = create_document_with_node();
    let json1 = serde_json::to_string(&doc).unwrap();
    let reparsed: DiagramDocument = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&reparsed).unwrap();
    assert_eq!(json1, json2, "Serialization should be idempotent");
}

// ============================================================================
// IO-003: DiagramDocument Round-Trip (Persistence)
// ============================================================================

#[test]
fn io_003_empty_document_round_trip() {
    let original = create_minimal_document();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: DiagramDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(original.version, parsed.version);
    assert_eq!(original.revision.value(), parsed.revision.value());
    assert_eq!(original.document.nodes.len(), parsed.document.nodes.len());
    assert_eq!(original.document.edges.len(), parsed.document.edges.len());
}

#[test]
fn io_003_document_with_node_round_trip() {
    let original = create_document_with_node();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: DiagramDocument = serde_json::from_str(&json).unwrap();
    
    assert_eq!(original.version, parsed.version);
    assert_eq!(original.document.nodes.len(), parsed.document.nodes.len());
    
    let original_node = original.document.nodes.get(&NodeId::new("node-1".to_string()));
    let parsed_node = parsed.document.nodes.get(&NodeId::new("node-1".to_string()));
    
    assert!(original_node.is_some());
    assert!(parsed_node.is_some());
    assert_eq!(original_node.unwrap().label, parsed_node.unwrap().label);
}

#[test]
fn io_003_document_with_edge_round_trip() {
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId::new("node-1".to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Source".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        },
    );
    nodes.insert(
        NodeId::new("node-2".to_string()),
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "Target".to_string(),
            x: OrderedFloat(200.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        },
    );

    let mut edges = HashMap::new();
    edges.insert(
        EdgeId::new("edge-1".to_string()),
        Edge {
            source: NodeId::new("node-1".to_string()),
            target: NodeId::new("node-2".to_string()),
            label: String::new(),
            style: EdgeStyle::Solid,
            arrow_type: ArrowType::Default,
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(2.0),
            directed: true,
            bend_points: im::Vector::new(),
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
        },
    );

    let original = DiagramDocument {
        version: 2,
        revision: Revision::new(2),
        document: DocumentData { nodes, edges },
        editor_state: Default::default(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: DiagramDocument = serde_json::from_str(&json).unwrap();

    assert_eq!(original.document.edges.len(), parsed.document.edges.len());
    let original_edge = original.document.edges.get(&EdgeId::new("edge-1".to_string()));
    let parsed_edge = parsed.document.edges.get(&EdgeId::new("edge-1".to_string()));
    assert!(original_edge.is_some());
    assert!(parsed_edge.is_some());
    assert_eq!(original_edge.unwrap().source, parsed_edge.unwrap().source);
    assert_eq!(original_edge.unwrap().target, parsed_edge.unwrap().target);
}
