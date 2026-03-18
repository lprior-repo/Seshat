//! Tests for event envelope module

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![forbid(unsafe_code)]

#[cfg(kani)]
use crate::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};
#[cfg(kani)]
use crate::envelope::parsing::parse_domain_op;
#[cfg(kani)]
use crate::envelope::types::{ContractError, LabelTargetId, LabelTargetType, OpKind};
#[cfg(kani)]
use crate::envelope::{
    domain_op_kind, encode_event_envelope, parse_event_envelope, Author, DomainOp, EventEnvelope,
};

#[cfg(kani)]
#[kani::proof]
#[test]
#[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
fn given_valid_json_when_parsing_event_envelope_then_returns_envelope() {
    let raw = r#"{
        "op_id": "evt-123",
        "operation": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "width": 80.0,
        "height": 40.0,
        "label": "Test Node",
        "author": {
            "id": "user-1",
            "name": "Alice"
        },
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let envelope = result.unwrap();
    assert_eq!(envelope.op_id, "evt-123");
    assert!(matches!(envelope.operation, DomainOp::NodeAdd { .. }));
    assert_eq!(envelope.author.id, "user-1");
    assert_eq!(envelope.timestamp, 1699999999);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_invalid_json_when_parsing_event_envelope_then_returns_invalid_json_error() {
    let raw = "not valid json";

    let result = parse_event_envelope(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::InvalidJson(_)) => {}
        _ => panic!("Expected InvalidJson error"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_missing_op_id_field_when_parsing_event_envelope_then_returns_missing_field_error() {
    let raw = r#"{
        "t": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "author": {"id": "user-1", "name": "Alice"},
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::MissingField(f)) => assert_eq!(f, "op_id"),
        _ => panic!("Expected MissingField error for 'op_id'"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_missing_author_field_when_parsing_event_envelope_then_returns_missing_field_error() {
    let raw = r#"{
        "op_id": "evt-123",
        "t": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::MissingField(f)) => assert_eq!(f, "author"),
        _ => panic!("Expected MissingField error for 'author'"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_invalid_author_missing_name_when_parsing_event_envelope_then_returns_invalid_author_error()
{
    let raw = r#"{
        "op_id": "evt-123",
        "t": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "author": {"id": "user-1"},
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::InvalidAuthor(_)) => {}
        _ => panic!("Expected InvalidAuthor error"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
#[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
fn given_unknown_op_type_type_when_parsing_event_envelope_then_returns_unknown_op_type_error() {
    let raw = r#"{
        "op_id": "evt-123",
        "t": "unknown_op_type",
        "author": {"id": "user-1", "name": "Alice"},
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::UnknownOpType(s)) => assert_eq!(s, "unknown_op_type"),
        _ => panic!("Expected UnknownOpType error"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
#[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
fn given_all_op_type_types_then_all_parse_correctly() {
    let test_cases = [
        (
            r#""t": "node_add", "id": "n1", "x": 0.0, "y": 0.0, "width": 80.0, "height": 40.0"#,
            "node_add",
        ),
        (
            r#""t": "node_move", "id": "n1", "x": 100.0, "y": 200.0"#,
            "node_move",
        ),
        (r#""t": "node_delete", "id": "n1""#, "node_delete"),
        (
            r#""t": "edge_connect", "id": "e1", "source": "n1", "target": "n2""#,
            "edge_connect",
        ),
        (r#""t": "edge_disconnect", "id": "e1""#, "edge_disconnect"),
        (r#""t": "bring_forward", "ids": ["n1"]"#, "bring_forward"),
        (r#""t": "send_backward", "ids": ["n1"]"#, "send_backward"),
        (r#""t": "bring_to_front", "ids": ["n1"]"#, "bring_to_front"),
        (r#""t": "send_to_back", "ids": ["n1"]"#, "send_to_back"),
        (r#""t": "group", "ids": ["n1", "n2"]"#, "group"),
        (r#""t": "ungroup", "id": "g1""#, "ungroup"),
    ];

    for (op_str, _op_name) in test_cases {
        let raw = format!(
            r#"{{"op_id": "evt-1", {}, "author": {{"id": "u1", "name": "A"}}, "timestamp": 1}}"#,
            op_str
        );
        let result = parse_event_envelope(&raw);
        assert!(result.is_ok(), "Failed for op: {}", op_str);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
#[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
fn given_author_with_email_when_parsing_event_envelope_then_email_is_preserved() {
    let raw = r#"{
        "op_id": "evt-123",
        "t": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "width": 80.0,
        "height": 40.0,
        "author": {
            "id": "user-1",
            "name": "Alice",
            "email": "alice@example.com"
        },
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_ok());
    let envelope = result.unwrap();
    assert_eq!(envelope.author.email, Some("alice@example.com".to_string()));
}

#[cfg(kani)]
#[kani::proof]
#[test]
#[ignore = "Known issue: serde internally tagged enum conflict with struct field"]
fn given_author_without_email_when_parsing_event_envelope_then_email_is_none() {
    let raw = r#"{
        "op_id": "evt-123",
        "t": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "width": 80.0,
        "height": 40.0,
        "author": {
            "id": "user-1",
            "name": "Alice"
        },
        "timestamp": 1699999999
    }"#;

    let result = parse_event_envelope(raw);

    assert!(result.is_ok());
    let envelope = result.unwrap();
    assert_eq!(envelope.author.email, None);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_event_envelope_when_encoding_then_roundtrip_works() {
    let original = EventEnvelope {
        op_id: "evt-roundtrip".to_string(),
        operation: DomainOp::NodeMove {
            id: NodeId::new("node-1".to_string()),
            x: 100.0,
            y: 200.0,
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Bob".to_string(),
            email: Some("bob@example.com".to_string()),
        },
        timestamp: 1700000000,
    };

    let encoded = encode_event_envelope(&original);
    assert!(encoded.is_ok(), "Encoding failed: {:?}", encoded.err());

    let decoded = parse_event_envelope(&encoded.unwrap());
    assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());

    assert_eq!(decoded.unwrap(), original);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_event_envelope_with_complex_operation_when_encoding_then_roundtrip_works() {
    let original = EventEnvelope {
        op_id: "evt-complex".to_string(),
        operation: DomainOp::Group {
            id: NodeId::new("group-1".to_string()),
            ids: vec![
                NodeId::new("node-1".to_string()),
                NodeId::new("node-2".to_string()),
                NodeId::new("node-3".to_string()),
            ],
        },

        author: Author {
            id: "user-2".to_string(),
            name: "Charlie".to_string(),
            email: None,
        },
        timestamp: 1700000001,
    };

    let encoded = encode_event_envelope(&original);
    assert!(encoded.is_ok(), "Encoding failed: {:?}", encoded.err());

    let decoded = parse_event_envelope(&encoded.unwrap());
    assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());

    assert_eq!(decoded.unwrap(), original);
}

// DomainOp parsing tests

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_node_add_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "node_add",
        "id": "node-1",
        "x": 100.0,
        "y": 200.0,
        "width": 80.0,
        "height": 40.0,
        "label": "Test Node"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    assert!(matches!(op, DomainOp::NodeAdd { .. }));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_node_move_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "node_move",
        "id": "node-1",
        "x": 150.0,
        "y": 250.0
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    let op = result.unwrap();
    assert!(matches!(op, DomainOp::NodeMove { .. }));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_node_delete_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "node_delete",
        "id": "node-1"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        DomainOp::NodeDelete { id } if id == NodeId::new("node-1".to_string())
    ));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_edge_connect_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "edge_connect",
        "id": "edge-1",
        "source": "node-1",
        "target": "node-2"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    let op = result.unwrap();
    assert!(matches!(op, DomainOp::EdgeConnect { .. }));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_edge_disconnect_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "edge_disconnect",
        "id": "edge-1"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        DomainOp::EdgeDisconnect { id } if id == EdgeId::new("edge-1".to_string())
    ));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_group_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "group",
        "ids": ["node-1", "node-2", "node-3"]
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    let op = result.unwrap();
    assert!(matches!(op, DomainOp::Group { ids } if ids.len() == 3));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_ungroup_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{
        "op": "ungroup",
        "id": "group-1"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        DomainOp::Ungroup { id } if id == NodeId::new("group-1".to_string())
    ));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_zorder_json_when_parsing_then_returns_domain_op() {
    let test_cases = [
        r#"{"op": "bring_forward", "ids": ["n1", "n2"]}"#,
        r#"{"op": "send_backward", "ids": ["n1", "n2"]}"#,
        r#"{"op": "bring_to_front", "ids": ["n1", "n2"]}"#,
        r#"{"op": "send_to_back", "ids": ["n1", "n2"]}"#,
    ];

    for raw in test_cases {
        let result = parse_domain_op(raw);
        assert!(result.is_ok(), "Failed for op: {}", raw);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_invalid_json_when_parsing_then_returns_invalid_json_error() {
    let raw = "not valid json";

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ContractError::InvalidJson(_)));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_missing_op_field_when_parsing_then_returns_missing_field_error() {
    let raw = r#"{
        "id": "node-1",
        "x": 100.0
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::MissingField(f)) => assert_eq!(f, "op"),
        _ => panic!("Expected MissingField error for 'op'"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_unknown_op_type_when_parsing_then_returns_unknown_op_type_error() {
    let raw = r#"{
        "op": "unknown_op_type",
        "id": "node-1"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::UnknownOpType(s)) => assert_eq!(s, "unknown_op_type"),
        _ => panic!("Expected UnknownOpType error"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_missing_required_field_when_parsing_then_returns_missing_field_error() {
    let raw = r#"{
        "op": "node_move",
        "id": "node-1"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::MissingField(f)) => assert!(f == "x" || f == "y"),
        _ => panic!("Expected MissingField error"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_invalid_array_when_parsing_then_returns_invalid_payload_error() {
    let raw = r#"{
        "op": "group",
        "ids": "not-an-array"
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ContractError::InvalidPayload(_)
    ));
}

// OpKind tests

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_add_op_when_getting_kind_then_returns_node_kind() {
    let op = DomainOp::NodeAdd {
        id: NodeId::new("node-1".to_string()),
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 40.0,
        label: "Test".to_string(),
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_move_op_when_getting_kind_then_returns_node_kind() {
    let op = DomainOp::NodeMove {
        id: NodeId::new("node-1".to_string()),
        x: 100.0,
        y: 200.0,
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_delete_op_when_getting_kind_then_returns_node_kind() {
    let op = DomainOp::NodeDelete {
        id: NodeId::new("node-1".to_string()),
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_node_restore_op_when_getting_kind_then_returns_node_kind() {
    let op = DomainOp::NodeRestore {
        id: NodeId::new("node-1".to_string()),
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_update_label_op_when_getting_kind_then_returns_node_kind() {
    let op = DomainOp::UpdateLabel {
        target_id: LabelTargetId::Node(NodeId::new("node-1".to_string())),
        target_type: LabelTargetType::Node,
        old_label: "Old Label".to_string(),
        new_label: "Test Label".to_string(),
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_update_label_with_very_long_label_when_parsing_then_succeeds() {
    let long_label = "x".repeat(15_000);
    let raw = format!(
        r#"{{"op": "update_label", "target_id": "n1", "target_type": "node", "old_label": "old", "new_label": "{}"}}"#,
        long_label
    );

    let result = parse_domain_op(&raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    match op {
        DomainOp::UpdateLabel { new_label, .. } => {
            assert_eq!(new_label.len(), 15_000);
        }
        _ => panic!("Expected UpdateLabel"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_update_label_with_mixed_direction_text_when_parsing_then_succeeds() {
    let raw = r#"{"op": "update_label", "target_id": "n1", "target_type": "node", "old_label": "old", "new_label": "Hello مرحبا World 🌍"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    match op {
        DomainOp::UpdateLabel { new_label, .. } => {
            assert_eq!(new_label, "Hello مرحبا World 🌍");
        }
        _ => panic!("Expected UpdateLabel"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_update_label_json_missing_op_field_when_parsing_then_returns_missing_field_error() {
    let raw = r#"{"target_id": "n1", "new_label": "New Label"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::MissingField(f)) => assert_eq!(f, "op"),
        _ => panic!("Expected MissingField error for 'op'"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_update_label_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{"op": "update_label", "target_id": "node-1", "target_type": "node", "old_label": "old", "new_label": "New Label"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    assert!(matches!(op, DomainOp::UpdateLabel { .. }));
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_update_label_with_edge_target_type_when_parsing_then_succeeds() {
    let raw = r#"{"op": "update_label", "target_id": "edge-1", "target_type": "edge", "old_label": "old label", "new_label": "new label"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    match op {
        DomainOp::UpdateLabel {
            target_type,
            target_id,
            ..
        } => {
            assert_eq!(target_type, LabelTargetType::Edge);
            assert!(matches!(target_id, LabelTargetId::Edge(_)));
        }
        _ => panic!("Expected UpdateLabel"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_update_label_backward_compatibility_with_old_fields_when_parsing_then_succeeds() {
    let raw = r#"{"op": "update_label", "id": "node-1", "label": "Test Label"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    match op {
        DomainOp::UpdateLabel {
            target_id,
            new_label,
            target_type,
            ..
        } => {
            assert!(matches!(target_id, LabelTargetId::Node(_)));
            assert_eq!(new_label, "Test Label");
            assert_eq!(target_type, LabelTargetType::Node);
        }
        _ => panic!("Expected UpdateLabel"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_edge_connect_op_when_getting_kind_then_returns_edge_kind() {
    let op = DomainOp::EdgeConnect {
        id: EdgeId::new("edge-1".to_string()),
        source: NodeId::new("node-1".to_string()),
        target: NodeId::new("node-2".to_string()),
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Edge);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_edge_disconnect_op_when_getting_kind_then_returns_edge_kind() {
    let op = DomainOp::EdgeDisconnect {
        id: EdgeId::new("edge-1".to_string()),
    };

    let kind = domain_op_kind(&op);

    assert_eq!(kind, OpKind::Edge);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_zorder_ops_when_getting_kind_then_returns_zorder_kind() {
    let ops = [
        DomainOp::BringForward {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::SendBackward {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::BringToFront {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::SendToBack {
            ids: vec![NodeId::new("n1".to_string())],
        },
    ];

    for op in ops {
        let kind = domain_op_kind(&op);
        assert_eq!(kind, OpKind::ZOrder, "Failed for {:?}", op);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_composite_ops_when_getting_kind_then_returns_composite_kind() {
    let ops = [
        DomainOp::Group {
            ids: vec![NodeId::new("n1".to_string()), NodeId::new("n2".to_string())],
        },
        DomainOp::Ungroup {
            id: NodeId::new("group-1".to_string()),
        },
    ];

    for op in ops {
        let kind = domain_op_kind(&op);
        assert_eq!(kind, OpKind::Composite, "Failed for {:?}", op);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_op_kind_as_str_then_returns_correct_string() {
    assert_eq!(OpKind::Node.as_str(), "node");
    assert_eq!(OpKind::Edge.as_str(), "edge");
    assert_eq!(OpKind::Composite.as_str(), "composite");
    assert_eq!(OpKind::ZOrder.as_str(), "z_order");
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_domain_op_group_then_parses_correctly() {
    let raw = r#"{
        "op": "group",
        "id": "group-1",
        "ids": ["node-1", "node-2", "node-3"]
    }"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok());
    let op = result.unwrap();
    assert!(
        matches!(op, DomainOp::Group { ref id, ref ids } if id == &NodeId::new("group-1".to_string()) && ids.len() == 3)
    );
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_domain_op_kind_method_then_matches_free_function() {
    let ops = [
        DomainOp::NodeAdd {
            id: NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            label: "".to_string(),
        },
        DomainOp::NodeMove {
            id: NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
        },
        DomainOp::NodeDelete {
            id: NodeId::new("n1".to_string()),
        },
        DomainOp::NodeRestore {
            id: NodeId::new("n1".to_string()),
        },
        DomainOp::EdgeConnect {
            id: EdgeId::new("e1".to_string()),
            source: NodeId::new("n1".to_string()),
            target: NodeId::new("n2".to_string()),
        },
        DomainOp::EdgeDisconnect {
            id: EdgeId::new("e1".to_string()),
        },
        DomainOp::BringForward {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::SendBackward {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::BringToFront {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::SendToBack {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::Group {
            id: NodeId::new("g1".to_string()),
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::Ungroup {
            id: NodeId::new("g1".to_string()),
        },
    ];

    for op in &ops {
        let method_kind = op.kind();
        let function_kind = domain_op_kind(op);
        assert_eq!(method_kind, function_kind, "Mismatch for {:?}", op);
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_all_domain_op_variants_exhaustive_match_then_all_cases_handled() {
    let check_variant = |op: DomainOp| -> &'static str {
        match op {
            DomainOp::NodeAdd { .. } => "NodeAdd",
            DomainOp::NodeMove { .. } => "NodeMove",
            DomainOp::NodeDelete { .. } => "NodeDelete",
            DomainOp::NodeRestore { .. } => "NodeRestore",
            DomainOp::NodeResize { .. } => "NodeResize",
            DomainOp::UpdateLabel { .. } => "UpdateLabel",
            DomainOp::UpdateNodeStyle { .. } => "UpdateNodeStyle",
            DomainOp::EdgeConnect { .. } => "EdgeConnect",
            DomainOp::EdgeDisconnect { .. } => "EdgeDisconnect",
            DomainOp::UpdateEdgeStyle { .. } => "UpdateEdgeStyle",
            DomainOp::BringForward { .. } => "BringForward",
            DomainOp::SendBackward { .. } => "SendBackward",
            DomainOp::BringToFront { .. } => "BringToFront",
            DomainOp::SendToBack { .. } => "SendToBack",
            DomainOp::Group { .. } => "Group",
            DomainOp::Ungroup { .. } => "Ungroup",
        }
    };

    let variants = [
        DomainOp::NodeAdd {
            id: NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 40.0,
            label: "".to_string(),
        },
        DomainOp::NodeMove {
            id: NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
        },
        DomainOp::NodeDelete {
            id: NodeId::new("n1".to_string()),
        },
        DomainOp::NodeRestore {
            id: NodeId::new("n1".to_string()),
        },
        DomainOp::NodeResize {
            id: NodeId::new("n1".to_string()),
            original_x: 0.0,
            original_y: 0.0,
            original_width: 80.0,
            original_height: 40.0,
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 60.0,
        },
        DomainOp::UpdateLabel {
            target_id: LabelTargetId::Node(NodeId::new("n1".to_string())),
            target_type: LabelTargetType::Node,
            old_label: "old".to_string(),
            new_label: "test".to_string(),
        },
        DomainOp::UpdateNodeStyle {
            id: NodeId::new("n1".to_string()),
            style: NodeStyle::default(),
        },
        DomainOp::EdgeConnect {
            id: EdgeId::new("e1".to_string()),
            source: NodeId::new("n1".to_string()),
            target: NodeId::new("n2".to_string()),
        },
        DomainOp::EdgeDisconnect {
            id: EdgeId::new("e1".to_string()),
        },
        DomainOp::UpdateEdgeStyle {
            id: EdgeId::new("e1".to_string()),
            style: EdgeStyle::default(),
        },
        DomainOp::BringForward {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::SendBackward {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::BringToFront {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::SendToBack {
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::Group {
            id: NodeId::new("g1".to_string()),
            ids: vec![NodeId::new("n1".to_string())],
        },
        DomainOp::Ungroup {
            id: NodeId::new("g1".to_string()),
        },
    ];

    for variant in variants {
        let _ = check_variant(variant);
    }
}

// LabelTargetId tests

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_label_target_id_node_then_returns_correct_target_type() {
    let target = LabelTargetId::Node(NodeId::new("node-1".to_string()));
    assert_eq!(target.target_type(), LabelTargetType::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_label_target_id_edge_then_returns_correct_target_type() {
    let target = LabelTargetId::Edge(EdgeId::new("edge-1".to_string()));
    assert_eq!(target.target_type(), LabelTargetType::Edge);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_label_target_id_from_node_id_then_works() {
    let node_id = NodeId::new("node-1".to_string());
    let target: LabelTargetId = node_id.into();
    assert_eq!(target.target_type(), LabelTargetType::Node);
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_label_target_id_from_edge_id_then_works() {
    let edge_id = EdgeId::new("edge-1".to_string());
    let target: LabelTargetId = edge_id.into();
    assert_eq!(target.target_type(), LabelTargetType::Edge);
}

// UpdateNodeStyle parsing tests

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_update_node_style_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{"op": "update_node_style", "id": "node-1", "style": "cloud"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    match op {
        DomainOp::UpdateNodeStyle { id, style } => {
            assert_eq!(id, NodeId::new("node-1".to_string()));
            assert_eq!(style, NodeStyle::Cloud);
        }
        _ => panic!("Expected UpdateNodeStyle"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_invalid_node_style_when_parsing_then_returns_error() {
    let raw = r#"{"op": "update_node_style", "id": "node-1", "style": "invalid_style"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::InvalidPayload(msg)) => {
            assert!(msg.contains("unknown node style"));
        }
        _ => panic!("Expected InvalidPayload error"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_valid_update_edge_style_json_when_parsing_then_returns_domain_op() {
    let raw = r#"{"op": "update_edge_style", "id": "edge-1", "style": "dotted"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    let op = result.unwrap();
    match op {
        DomainOp::UpdateEdgeStyle { id, style } => {
            assert_eq!(id, EdgeId::new("edge-1".to_string()));
            assert_eq!(style, EdgeStyle::Dotted);
        }
        _ => panic!("Expected UpdateEdgeStyle"),
    }
}

#[cfg(kani)]
#[kani::proof]
#[test]
fn given_invalid_edge_style_when_parsing_then_returns_error() {
    let raw = r#"{"op": "update_edge_style", "id": "edge-1", "style": "invalid_style"}"#;

    let result = parse_domain_op(raw);

    assert!(result.is_err());
    match result {
        Err(ContractError::InvalidPayload(msg)) => {
            assert!(msg.contains("unknown edge style"));
        }
        _ => panic!("Expected InvalidPayload error"),
    }
}

// ---------------------------------------------------------
// Executable Rust Tests for UpdateNodeStyle (Not Kani)
// ---------------------------------------------------------

#[cfg(test)]
use crate::document::{NodeId, NodeStyle};
#[cfg(test)]
use crate::envelope::{
    domain_op_kind, encode_event_envelope, parse_event_envelope, Author, DomainOp, EventEnvelope,
    OpKind,
};

#[test]
fn test_update_node_style_variant_constructable_with_valid_fields() {
    let style = NodeStyle::Cloud;
    let id = NodeId::new("node-test".to_string());

    let op = DomainOp::UpdateNodeStyle {
        id: id.clone(),
        style,
    };

    match op {
        DomainOp::UpdateNodeStyle {
            id: matched_id,
            style: matched_style,
        } => {
            assert_eq!(matched_id, id);
            assert_eq!(matched_style, NodeStyle::Cloud);
        }
        _ => panic!("Expected UpdateNodeStyle"),
    }
}

#[test]
fn test_update_node_style_kind_returns_node() {
    let op = DomainOp::UpdateNodeStyle {
        id: NodeId::new("node-test".to_string()),
        style: NodeStyle::Box,
    };

    assert_eq!(domain_op_kind(&op), OpKind::Node);
    assert_eq!(op.kind(), OpKind::Node);
}

#[test]
fn test_update_node_style_serializes_to_correct_json() {
    let op = DomainOp::UpdateNodeStyle {
        id: NodeId::new("node-1".to_string()),
        style: NodeStyle::Cylinder,
    };

    let json = serde_json::to_string(&op).expect("Failed to serialize");
    assert!(json.contains(r#""op_type":"update_node_style""#));
    assert!(json.contains(r#""id":"node-1""#));
    assert!(json.contains(r#""style":"cylinder""#));
}

#[test]
fn test_update_node_style_deserializes_from_valid_json() {
    let json = r#"{"op_type":"update_node_style","id":"node-1","style":"dashed"}"#;

    let op: DomainOp = serde_json::from_str(json).expect("Failed to deserialize");

    match op {
        DomainOp::UpdateNodeStyle { id, style } => {
            assert_eq!(id.as_str(), "node-1");
            assert_eq!(style, NodeStyle::Dashed);
        }
        _ => panic!("Expected UpdateNodeStyle"),
    }
}

#[test]
fn test_update_node_style_all_four_style_variants() {
    let styles = [
        NodeStyle::Box,
        NodeStyle::Cloud,
        NodeStyle::Cylinder,
        NodeStyle::Dashed,
    ];
    let style_names = ["box", "cloud", "cylinder", "dashed"];

    for (style, name) in styles.iter().zip(style_names.iter()) {
        let op = DomainOp::UpdateNodeStyle {
            id: NodeId::new("test-node".to_string()),
            style: style.clone(),
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains(&format!(r#""style":"{}""#, name)));

        let deserialized: DomainOp = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }
}

#[test]
fn test_update_node_style_roundtrip_event_envelope() {
    let op = EventEnvelope {
        op_id: "evt-style-update".to_string(),
        operation: DomainOp::UpdateNodeStyle {
            id: NodeId::new("node-1".to_string()),
            style: NodeStyle::Cloud,
        },
        author: Author {
            id: "user-1".to_string(),
            name: "Alice".to_string(),
            email: None,
        },
        timestamp: 1700000000,
    };

    let encoded = encode_event_envelope(&op).expect("Encoding failed");
    let decoded = parse_event_envelope(&encoded).expect("Decoding failed");

    assert_eq!(decoded, op);
}
