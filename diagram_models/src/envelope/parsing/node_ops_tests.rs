#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(test)]
mod tests {
    use crate::document::NodeStyle;
    use crate::envelope::domain_ops::DomainOp;
    use crate::envelope::parsing::node_ops::{
        extract_and_validate_dimensions, parse_node_add, parse_node_delete, parse_node_move,
        parse_node_resize, parse_node_restore, parse_update_label, parse_update_node_style,
    };
    use crate::envelope::types::{ContractError, LabelTargetId, LabelTargetType};
    use serde_json::json;

    #[test]
    fn given_valid_json_when_parse_node_add_then_returns_op() {
        let json = json!({
            "id": "n1",
            "x": 10.0,
            "y": 20.0,
            "width": 100.0,
            "height": 50.0,
            "label": "Test Node"
        });

        let result = parse_node_add(&json).unwrap();
        if let DomainOp::NodeAdd {
            id,
            x,
            y,
            width,
            height,
            label,
        } = result
        {
            assert_eq!(id.as_str(), "n1");
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
            assert_eq!(width, 100.0);
            assert_eq!(height, 50.0);
            assert_eq!(label, "Test Node");
        } else {
            panic!("Expected NodeAdd");
        }
    }

    #[test]
    fn given_missing_id_when_parse_node_add_then_returns_error() {
        let json = json!({
            "x": 10.0,
            "y": 20.0,
            "width": 100.0,
            "height": 50.0
        });

        let result = parse_node_add(&json);
        assert!(matches!(result, Err(ContractError::MissingField("id"))));
    }

    #[test]
    fn given_valid_json_when_parse_node_move_then_returns_op() {
        let json = json!({
            "id": "n1",
            "x": 10.0,
            "y": 20.0
        });

        let result = parse_node_move(&json).unwrap();
        if let DomainOp::NodeMove { id, x, y } = result {
            assert_eq!(id.as_str(), "n1");
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
        } else {
            panic!("Expected NodeMove");
        }
    }

    #[test]
    fn given_valid_json_when_parse_node_delete_then_returns_op() {
        let json = json!({ "id": "n1" });
        let result = parse_node_delete(&json).unwrap();
        if let DomainOp::NodeDelete { id } = result {
            assert_eq!(id.as_str(), "n1");
        } else {
            panic!("Expected NodeDelete");
        }
    }

    #[test]
    fn given_valid_json_when_parse_node_restore_then_returns_op() {
        let json = json!({ "id": "n1" });
        let result = parse_node_restore(&json).unwrap();
        if let DomainOp::NodeRestore { id } = result {
            assert_eq!(id.as_str(), "n1");
        } else {
            panic!("Expected NodeRestore");
        }
    }

    #[test]
    fn given_valid_json_when_parse_node_resize_then_returns_op() {
        let json = json!({
            "id": "n1",
            "original_x": 0.0,
            "original_y": 0.0,
            "original_width": 100.0,
            "original_height": 100.0,
            "x": 10.0,
            "y": 10.0,
            "width": 120.0,
            "height": 120.0
        });

        let result = parse_node_resize(&json).unwrap();
        if let DomainOp::NodeResize {
            id,
            x,
            y,
            width,
            height,
            original_x,
            original_y,
            original_width,
            original_height,
        } = result
        {
            assert_eq!(id.as_str(), "n1");
            assert_eq!(x, 10.0);
            assert_eq!(y, 10.0);
            assert_eq!(width, 120.0);
            assert_eq!(height, 120.0);
            assert_eq!(original_x, 0.0);
            assert_eq!(original_y, 0.0);
            assert_eq!(original_width, 100.0);
            assert_eq!(original_height, 100.0);
        } else {
            panic!("Expected NodeResize");
        }
    }

    #[test]
    fn given_invalid_dimension_when_parse_node_resize_then_returns_error() {
        let json = json!({
            "id": "n1",
            "original_x": 0.0,
            "original_y": 0.0,
            "original_width": -100.0, // Invalid
            "original_height": 100.0,
            "x": 10.0,
            "y": 10.0,
            "width": 120.0,
            "height": 120.0
        });

        let result = parse_node_resize(&json);
        assert!(matches!(result, Err(ContractError::InvalidPayload(_))));
    }

    #[test]
    fn given_valid_node_target_when_parse_update_label_then_returns_op() {
        let json = json!({
            "target_type": "node",
            "target_id": "n1",
            "old_label": "old",
            "new_label": "new"
        });

        let result = parse_update_label(&json).unwrap();
        if let DomainOp::UpdateLabel {
            target_id,
            target_type,
            old_label,
            new_label,
        } = result
        {
            assert_eq!(target_type, LabelTargetType::Node);
            assert!(matches!(target_id, LabelTargetId::Node(id) if id.as_str() == "n1"));
            assert_eq!(old_label, "old");
            assert_eq!(new_label, "new");
        } else {
            panic!("Expected UpdateLabel");
        }
    }

    #[test]
    fn given_legacy_id_field_when_parse_update_label_then_returns_op() {
        let json = json!({
            "target_type": "edge",
            "id": "e1",
            "label": "new_legacy"
        });

        let result = parse_update_label(&json).unwrap();
        if let DomainOp::UpdateLabel {
            target_id,
            target_type,
            old_label,
            new_label,
        } = result
        {
            assert_eq!(target_type, LabelTargetType::Edge);
            assert!(matches!(target_id, LabelTargetId::Edge(id) if id.as_str() == "e1"));
            assert_eq!(old_label, "");
            assert_eq!(new_label, "new_legacy");
        } else {
            panic!("Expected UpdateLabel");
        }
    }

    #[test]
    fn given_valid_json_when_parse_update_node_style_then_returns_op() {
        let json = json!({
            "id": "n1",
            "style": "cloud"
        });

        let result = parse_update_node_style(&json).unwrap();
        if let DomainOp::UpdateNodeStyle { id, style } = result {
            assert_eq!(id.as_str(), "n1");
            assert_eq!(style, NodeStyle::Cloud);
        } else {
            panic!("Expected UpdateNodeStyle");
        }
    }

    #[test]
    fn given_invalid_style_when_parse_update_node_style_then_returns_error() {
        let json = json!({
            "id": "n1",
            "style": "invalid_style"
        });

        let result = parse_update_node_style(&json);
        assert!(matches!(result, Err(ContractError::InvalidPayload(_))));
    }
}
