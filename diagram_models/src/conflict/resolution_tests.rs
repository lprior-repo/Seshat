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
    use crate::conflict::resolution::{
        evaluate_human_priority, evaluate_human_priority_with_projection,
        extract_affected_entities, is_human_author, record_conflict_rejection,
    };
    use crate::conflict::{ConflictDecision, ConflictError, ProjectionState};
    use crate::document::{EdgeId, NodeId};
    use crate::envelope::{Author, DomainOp, EventEnvelope, LabelTargetType};
    use crate::projection::DiagramProjection;
    use im::HashMap;

    fn make_human_author() -> Author {
        Author {
            id: "human-123".to_string(),
            name: "Human Being".to_string(),
            email: None,
        }
    }

    fn make_ai_author() -> Author {
        Author {
            id: "agent-1".to_string(),
            name: "AI Agent".to_string(),
            email: None,
        }
    }

    fn make_envelope(op_id: &str, author: Author, op: DomainOp) -> EventEnvelope {
        EventEnvelope {
            op_id: op_id.to_string(),
            author,
            operation: op,
            timestamp: 0,
        }
    }

    #[test]
    fn given_human_author_when_checked_then_is_human() {
        assert!(is_human_author(&make_human_author()));
        let mut mixed = make_human_author();
        mixed.id = "some-id".to_string(); // name still has "human"
        assert!(is_human_author(&mixed));
    }

    #[test]
    fn given_ai_author_when_checked_then_is_not_human() {
        assert!(!is_human_author(&make_ai_author()));
    }

    #[test]
    fn given_node_add_op_when_extract_entities_then_returns_node_entity() {
        let op = DomainOp::NodeAdd {
            id: NodeId::new("n1".to_string()),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            label: "test".to_string(),
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["node:n1"]);
    }

    #[test]
    fn given_edge_connect_op_when_extract_entities_then_returns_edge_and_nodes() {
        let op = DomainOp::EdgeConnect {
            id: EdgeId::new("e1".to_string()),
            source: NodeId::new("n1".to_string()),
            target: NodeId::new("n2".to_string()),
        };
        let entities = extract_affected_entities(&op);
        assert_eq!(entities, vec!["edge:e1", "node:n1", "node:n2"]);
    }

    #[test]
    fn given_human_op_when_evaluate_priority_then_allows() {
        let state = ProjectionState::new();
        let op = make_envelope(
            "op1",
            make_human_author(),
            DomainOp::NodeDelete {
                id: NodeId::new("n1".to_string()),
            },
        );
        let decision = evaluate_human_priority(&op, &state).unwrap();
        assert_eq!(decision, ConflictDecision::Allow);
    }

    #[test]
    fn given_ai_op_with_no_conflicts_when_evaluate_priority_then_allows() {
        let state = ProjectionState::new();
        let op = make_envelope(
            "op1",
            make_ai_author(),
            DomainOp::NodeDelete {
                id: NodeId::new("n1".to_string()),
            },
        );
        let decision = evaluate_human_priority(&op, &state).unwrap();
        assert_eq!(decision, ConflictDecision::Allow);
    }

    #[test]
    fn given_ai_op_with_conflict_when_evaluate_priority_then_rejects() {
        let mut state = ProjectionState::new();
        state.register_human_edit("node:n1", "human-1");

        let op = make_envelope(
            "op1",
            make_ai_author(),
            DomainOp::NodeDelete {
                id: NodeId::new("n1".to_string()),
            },
        );
        let decision = evaluate_human_priority(&op, &state).unwrap();

        if let ConflictDecision::Reject {
            reason,
            conflicting_entities,
        } = decision
        {
            assert!(matches!(reason, ConflictError::HumanPriorityBlock(_)));
            assert_eq!(conflicting_entities, vec!["node:n1"]);
        } else {
            panic!("Expected reject");
        }
    }

    #[test]
    fn given_ai_op_already_processed_when_evaluate_priority_then_allows() {
        let mut state = ProjectionState::new();
        state.mark_processed("op1");
        state.register_human_edit("node:n1", "human-1"); // Conflicting state

        let op = make_envelope(
            "op1",
            make_ai_author(),
            DomainOp::NodeDelete {
                id: NodeId::new("n1".to_string()),
            },
        );
        let decision = evaluate_human_priority(&op, &state).unwrap();
        assert_eq!(
            decision,
            ConflictDecision::Allow,
            "Already processed should bypass conflict checks"
        );
    }

    #[test]
    fn given_missing_node_on_delete_when_evaluate_with_projection_then_returns_missing_error() {
        let state = ProjectionState::new();
        let projection = DiagramProjection {
            version: 0,
            revision: 0,
            nodes: HashMap::new(), // Empty projection
            edges: HashMap::new(),
            author_priority: HashMap::new(),
            cycle_policy: Default::default(),
        };

        let op = make_envelope(
            "op1",
            make_ai_author(),
            DomainOp::NodeDelete {
                id: NodeId::new("n1".to_string()),
            },
        );
        let result = evaluate_human_priority_with_projection(&op, &state, &projection);

        assert!(matches!(result, Err(ConflictError::MissingEntity(e)) if e == "node:n1"));
    }

    #[test]
    fn given_human_op_when_recorded_then_edits_registered() {
        let mut state = ProjectionState::new();
        let op = make_envelope(
            "op1",
            make_human_author(),
            DomainOp::NodeDelete {
                id: NodeId::new("n1".to_string()),
            },
        );

        record_conflict_rejection(&mut state, &op, &ConflictDecision::Allow);

        assert!(state.is_processed("op1"));
        assert!(state.has_active_human_edit("node:n1"));
    }
}
