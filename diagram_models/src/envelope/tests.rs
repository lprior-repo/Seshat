#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[cfg(test)]
mod tests {
    use crate::document::{EdgeId, EdgeStyle, NodeId, NodeStyle};
    use crate::envelope::domain_ops::{domain_op_kind, DomainOp};
    use crate::envelope::parsing::parse_domain_op;
    use crate::envelope::types::{Author, ContractError, LabelTargetId, LabelTargetType, OpKind};
    use crate::envelope::{
        decode_envelope, encode_envelope, encode_event_envelope, parse_event_envelope,
        EventEnvelope,
    };

    #[test]
    fn given_invalid_json_when_parsing_domain_op_then_returns_error() {
        let result = parse_domain_op("not json");
        assert!(matches!(result, Err(ContractError::InvalidJson(_))));
    }

    #[test]
    fn given_json_missing_op_when_parsing_domain_op_then_returns_error() {
        let result = parse_domain_op(r#"{"id": "n1"}"#);
        assert!(matches!(result, Err(ContractError::MissingField("op"))));
    }

    #[test]
    fn given_unknown_op_when_parsing_domain_op_then_returns_error() {
        let result = parse_domain_op(r#"{"op": "unknown_action"}"#);
        assert!(matches!(result, Err(ContractError::UnknownOpType(op)) if op == "unknown_action"));
    }

    #[test]
    fn given_node_add_json_when_parsing_domain_op_then_returns_node_add_op() {
        let valid_node_add = r#"{"op": "node_add", "id": "n1", "x": 10.0, "y": 20.0, "width": 100.0, "height": 50.0, "label": "test"}"#;
        let result = parse_domain_op(valid_node_add);
        assert!(matches!(result, Ok(DomainOp::NodeAdd { .. })));
    }

    #[test]
    fn given_node_move_json_when_parsing_domain_op_then_returns_node_move_op() {
        let valid_node_move = r#"{"op": "node_move", "id": "n1", "x": 10.0, "y": 20.0}"#;
        assert!(matches!(
            parse_domain_op(valid_node_move),
            Ok(DomainOp::NodeMove { .. })
        ));
    }

    #[test]
    fn given_node_delete_json_when_parsing_domain_op_then_returns_node_delete_op() {
        let valid = r#"{"op": "node_delete", "id": "n1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::NodeDelete { .. })
        ));
    }

    #[test]
    fn given_node_restore_json_when_parsing_domain_op_then_returns_node_restore_op() {
        let valid = r#"{"op": "node_restore", "id": "n1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::NodeRestore { .. })
        ));
    }

    #[test]
    fn given_node_resize_json_when_parsing_domain_op_then_returns_node_resize_op() {
        let valid = r#"{"op": "node_resize", "id": "n1", "original_x": 0.0, "original_y": 0.0, "original_width": 10.0, "original_height": 10.0, "x": 10.0, "y": 20.0, "width": 100.0, "height": 50.0}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::NodeResize { .. })
        ));
    }

    #[test]
    fn given_update_label_json_when_parsing_domain_op_then_returns_update_label_op() {
        let valid = r#"{"op": "update_label", "target_id": "n1", "target_type": "node", "old_label": "a", "new_label": "b"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::UpdateLabel { .. })
        ));
    }

    #[test]
    fn given_update_node_style_json_when_parsing_domain_op_then_returns_update_node_style_op() {
        let valid = r#"{"op": "update_node_style", "id": "n1", "style": "box"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::UpdateNodeStyle { .. })
        ));
    }

    #[test]
    fn given_edge_connect_json_when_parsing_domain_op_then_returns_edge_connect_op() {
        let valid = r#"{"op": "edge_connect", "id": "e1", "source": "n1", "target": "n2"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::EdgeConnect { .. })
        ));
    }

    #[test]
    fn given_edge_disconnect_json_when_parsing_domain_op_then_returns_edge_disconnect_op() {
        let valid = r#"{"op": "edge_disconnect", "id": "e1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::EdgeDisconnect { .. })
        ));
    }

    #[test]
    fn given_update_edge_style_json_when_parsing_domain_op_then_returns_update_edge_style_op() {
        let valid = r#"{"op": "update_edge_style", "id": "e1", "style": "solid"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::UpdateEdgeStyle { .. })
        ));
    }

    #[test]
    fn given_bring_forward_json_when_parsing_domain_op_then_returns_bring_forward_op() {
        let valid = r#"{"op": "bring_forward", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::BringForward { .. })
        ));
    }

    #[test]
    fn given_send_backward_json_when_parsing_domain_op_then_returns_send_backward_op() {
        let valid = r#"{"op": "send_backward", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::SendBackward { .. })
        ));
    }

    #[test]
    fn given_bring_to_front_json_when_parsing_domain_op_then_returns_bring_to_front_op() {
        let valid = r#"{"op": "bring_to_front", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::BringToFront { .. })
        ));
    }

    #[test]
    fn given_send_to_back_json_when_parsing_domain_op_then_returns_send_to_back_op() {
        let valid = r#"{"op": "send_to_back", "ids": ["n1"]}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::SendToBack { .. })
        ));
    }

    #[test]
    fn given_group_json_when_parsing_domain_op_then_returns_group_op() {
        let valid = r#"{"op": "group", "id": "g1", "ids": ["n1"]}"#;
        assert!(matches!(parse_domain_op(valid), Ok(DomainOp::Group { .. })));
    }

    #[test]
    fn given_ungroup_json_when_parsing_domain_op_then_returns_ungroup_op() {
        let valid = r#"{"op": "ungroup", "id": "g1"}"#;
        assert!(matches!(
            parse_domain_op(valid),
            Ok(DomainOp::Ungroup { .. })
        ));
    }

    #[test]
    fn given_valid_fields_when_constructing_update_node_style_then_succeeds() {
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
    fn given_update_node_style_op_when_getting_kind_then_returns_node_kind() {
        let op = DomainOp::UpdateNodeStyle {
            id: NodeId::new("node-test".to_string()),
            style: NodeStyle::Box,
        };

        assert_eq!(domain_op_kind(&op), OpKind::Node);
        assert_eq!(op.kind(), OpKind::Node);
    }

    #[test]
    fn given_update_node_style_op_when_serializing_then_returns_correct_json() {
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
    fn given_valid_json_when_deserializing_update_node_style_then_succeeds() {
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
    fn given_all_style_variants_when_serializing_then_all_succeed() {
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
    fn given_update_node_style_envelope_when_roundtripping_then_matches_original() {
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
}
#[cfg(test)]
mod legacy_tests {
    use super::tests::*;
    use crate::document::NodeId;
    use crate::envelope::domain_ops::DomainOp;
    use crate::envelope::{decode_envelope, encode_envelope, Author, EventEnvelope};

    #[test]
    #[allow(deprecated)]
    fn given_legacy_envelope_when_encoding_and_decoding_then_matches_original() {
        let op = EventEnvelope {
            op_id: "evt-legacy".to_string(),
            operation: DomainOp::NodeDelete {
                id: NodeId::new("node-1".to_string()),
            },
            author: Author {
                id: "user-1".to_string(),
                name: "Alice".to_string(),
                email: None,
            },
            timestamp: 1700000000,
        };

        let encoded = encode_envelope(&op).expect("Encoding failed");
        let decoded = decode_envelope(&encoded).expect("Decoding failed");

        assert_eq!(decoded, op);
    }
}
