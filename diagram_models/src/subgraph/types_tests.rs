#[cfg(test)]
mod tests {
    use crate::document::{DocumentData, LockState, Node, NodeId, NodeKind, OrderedFloat};
    use crate::geometry::Point;
    use crate::subgraph::types::{
        apply_viewport_transform, calculate_container_bounds, calculate_container_bounds_from_ids,
        create_empty_subgraph, BoundingBox, Error, Padding, PositiveScale,
    };
    use im::HashMap;

    fn create_test_node(x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
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
    fn given_bounding_box_when_new_called_then_correct_values() {
        let bbox = BoundingBox::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(bbox.min_x, 10.0);
        assert_eq!(bbox.min_y, 20.0);
        assert_eq!(bbox.max_x, 30.0);
        assert_eq!(bbox.max_y, 40.0);
    }

    #[test]
    fn given_positive_value_when_positive_scale_try_new_then_ok() {
        let scale = PositiveScale::try_new(OrderedFloat(2.0)).unwrap();
        assert_eq!(scale.value(), 2.0);
    }

    #[test]
    fn given_zero_or_negative_value_when_positive_scale_try_new_then_err() {
        assert_eq!(
            PositiveScale::try_new(OrderedFloat(0.0)),
            Err(Error::InvalidTransform)
        );
        assert_eq!(
            PositiveScale::try_new(OrderedFloat(-1.0)),
            Err(Error::InvalidTransform)
        );
    }

    #[test]
    fn given_node_and_scale_when_apply_viewport_transform_then_coords_and_dims_scaled() {
        let node = create_test_node(10.0, 20.0, 30.0, 40.0);
        let scale = PositiveScale::try_new(OrderedFloat(2.0)).unwrap();

        let transformed = apply_viewport_transform(&node, scale).unwrap();
        assert_eq!(transformed.x.0, 20.0);
        assert_eq!(transformed.y.0, 40.0);
        assert_eq!(transformed.width.0, 60.0);
        assert_eq!(transformed.height.0, 80.0);
    }

    #[test]
    fn given_no_children_when_calculate_container_bounds_then_returns_zero_rect() {
        let padding = Padding {
            top: 10,
            right: 10,
            bottom: 10,
            left: 10,
        };
        let bounds = calculate_container_bounds(&[], padding).unwrap();
        assert_eq!(bounds, BoundingBox::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn given_children_and_padding_when_calculate_container_bounds_then_returns_padded_bbox() {
        let n1 = create_test_node(0.0, 0.0, 10.0, 10.0);
        let n2 = create_test_node(20.0, 20.0, 10.0, 10.0);
        let padding = Padding {
            top: 5,
            right: 5,
            bottom: 5,
            left: 5,
        };

        let bounds = calculate_container_bounds(&[n1, n2], padding).unwrap();

        // min_x = 0, min_y = 0, max_x = 30, max_y = 30
        // with padding: min_x = -5, min_y = -5, max_x = 35, max_y = 35
        assert_eq!(bounds, BoundingBox::new(-5.0, -5.0, 35.0, 35.0));
    }

    #[test]
    fn given_no_ids_when_calculate_container_bounds_from_ids_then_returns_zero_rect() {
        let doc = DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };
        let padding = Padding {
            top: 10,
            right: 10,
            bottom: 10,
            left: 10,
        };
        let bounds = calculate_container_bounds_from_ids(&doc, &[], padding).unwrap();
        assert_eq!(bounds, BoundingBox::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn given_missing_id_when_calculate_container_bounds_from_ids_then_returns_error() {
        let doc = DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };
        let padding = Padding {
            top: 10,
            right: 10,
            bottom: 10,
            left: 10,
        };
        let result = calculate_container_bounds_from_ids(
            &doc,
            &[NodeId::new("missing".to_string())],
            padding,
        );
        assert!(matches!(result, Err(Error::NodeNotFound(_))));
    }

    #[test]
    fn given_valid_ids_when_calculate_container_bounds_from_ids_then_returns_padded_bbox() {
        let mut doc = DocumentData {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };
        let id1 = NodeId::new("n1".to_string());
        let id2 = NodeId::new("n2".to_string());
        doc.nodes
            .insert(id1.clone(), create_test_node(0.0, 0.0, 10.0, 10.0));
        doc.nodes
            .insert(id2.clone(), create_test_node(20.0, 20.0, 10.0, 10.0));

        let padding = Padding {
            top: 5,
            right: 5,
            bottom: 5,
            left: 5,
        };
        let bounds = calculate_container_bounds_from_ids(&doc, &[id1, id2], padding).unwrap();
        assert_eq!(bounds, BoundingBox::new(-5.0, -5.0, 35.0, 35.0));
    }

    #[test]
    fn given_position_when_create_empty_subgraph_then_returns_subgraph_node() {
        let pos = Point::new(10.0, 20.0);
        let id = NodeId::new("sg1".to_string());
        let node = create_empty_subgraph(id, pos).unwrap();

        assert_eq!(node.kind, NodeKind::Subgraph);
        assert_eq!(node.x.0, 10.0);
        assert_eq!(node.y.0, 20.0);
        assert!(node.width.0 >= 100.0);
        assert!(node.height.0 >= 60.0);
    }
}
