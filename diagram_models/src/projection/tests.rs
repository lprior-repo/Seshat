//! Tests for the projection module

#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::document::LockState;
    use crate::envelope::Author;
    use crate::projection::replay::{apply_event, replay_events};
    use crate::projection::types::DiagramProjection;

    fn make_author(is_human: bool) -> Author {
        if is_human {
            Author {
                id: "human-1".to_string(),
                name: "Alice".to_string(),
                email: None,
            }
        } else {
            Author {
                id: "ai-1".to_string(),
                name: "AI Assistant".to_string(),
                email: None,
            }
        }
    }

    fn make_event(
        op_id: &str,
        revision: u64,
        operation: crate::envelope::DomainOp,
        is_human: bool,
    ) -> crate::projection::types::EventRecord {
        crate::projection::types::EventRecord {
            op_id: op_id.to_string(),
            revision,
            operation,
            author: make_author(is_human),
            timestamp: 1700000000,
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_empty_events_when_replaying_then_returns_empty_projection() {
        let events: &[crate::projection::types::EventRecord] = &[];
        let result = replay_events(events);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 0);
        assert!(projection.nodes.is_empty());
        assert!(projection.edges.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_single_node_add_when_replaying_then_includes_node() {
        let events = [make_event(
            "op-1",
            0,
            crate::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 80.0,
                height: 40.0,
                label: "Test Node".to_string(),
            },
            true,
        )];

        let result = replay_events(&events);

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 1);
        assert_eq!(projection.nodes.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_events_when_replaying_then_increments_revision() {
        let events = [
            make_event(
                "op-1",
                0,
                crate::envelope::DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 1".to_string(),
                },
                true,
            ),
            make_event(
                "op-2",
                1,
                crate::envelope::DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 2".to_string(),
                },
                true,
            ),
            make_event(
                "op-3",
                2,
                crate::envelope::DomainOp::EdgeConnect {
                    id: "edge-1".to_string(),
                    source: "node-1".to_string(),
                    target: "node-2".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 3);
        assert_eq!(projection.nodes.len(), 2);
        assert_eq!(projection.edges.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_revision_gap_when_replaying_then_returns_error() {
        let events = [
            make_event(
                "op-1",
                0,
                crate::envelope::DomainOp::NodeAdd {
                    id: "node-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 1".to_string(),
                },
                true,
            ),
            // Skip revision 1 - gap!
            make_event(
                "op-2",
                2,
                crate::envelope::DomainOp::NodeAdd {
                    id: "node-2".to_string(),
                    x: 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 40.0,
                    label: "Node 2".to_string(),
                },
                true,
            ),
        ];

        let result = replay_events(&events);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::projection::types::ReplayError::InvariantViolation(_)
        ));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_apply_event_on_valid_state_then_returns_updated_state() {
        let initial = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            crate::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 50.0,
                y: 75.0,
                width: 100.0,
                height: 50.0,
                label: "Test".to_string(),
            },
            true,
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let projection = result.unwrap();
        assert_eq!(projection.revision, 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_human_author_then_priority_map_has_true() {
        let initial = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            crate::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "Human Node".to_string(),
            },
            true, // Human author
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.author_priority.get("op-1"), Some(&true));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_ai_author_then_priority_map_has_false() {
        let initial = DiagramProjection::empty();
        let event = make_event(
            "op-1",
            0,
            crate::envelope::DomainOp::NodeAdd {
                id: "node-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 40.0,
                label: "AI Node".to_string(),
            },
            false, // AI author
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok());
        let projection = result.unwrap();
        assert_eq!(projection.author_priority.get("op-1"), Some(&false));
    }

    #[test]
    fn given_update_node_style_event_then_returns_updated_state() {
        let mut initial = DiagramProjection::empty();
        // Setup a node to update
        let node_id = crate::document::NodeId::new("node-1".to_string());
        initial.nodes.insert(
            node_id.clone(),
            crate::document::Node {
                kind: crate::document::NodeKind::Node,
                icon: String::new(),
                label: "Test".to_string(),
                x: crate::document::OrderedFloat(0.0),
                y: crate::document::OrderedFloat(0.0),
                width: crate::document::OrderedFloat(100.0),
                height: crate::document::OrderedFloat(100.0),
                style: Some(crate::document::NodeStyle::Box),
                font_size: None,
                font_weight: None,
                lock_state: LockState::Unlocked,
                parent: None,
                dag_rank: None,
                tags: im::Vector::new(),
                metadata: im::HashMap::new(),
                z_index: 0,
                collapsed: Some(false),
            },
        );

        let event = make_event(
            "op-1",
            0,
            crate::envelope::DomainOp::UpdateNodeStyle {
                id: node_id.clone(),
                style: crate::document::NodeStyle::Cloud,
            },
            true,
        );

        let result = apply_event(initial, &event);

        assert!(result.is_ok(), "Error: {:?}", result.err());
        let projection = result.unwrap();
        let updated_node = projection.nodes.get(&node_id).expect("Node should exist");
        assert_eq!(updated_node.style, Some(crate::document::NodeStyle::Cloud));
    }
}
