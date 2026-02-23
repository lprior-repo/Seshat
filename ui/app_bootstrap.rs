use crate::graph::{ExecutionState, Node, NodeCategory, NodeId, Viewport, Workflow};

#[must_use]
pub fn default_workflow() -> Workflow {
    Workflow {
        nodes: vec![
            Node {
                id: NodeId::new(),
                name: "HTTP Handler".to_string(),
                description: "POST /SignupWorkflow/{userId}/run".to_string(),
                node_type: "http-handler".to_string(),
                category: NodeCategory::Entry,
                icon: "globe".to_string(),
                x: 350.0,
                y: 40.0,
                config: serde_json::json!({"configured": true, "status": "completed", "journalIndex": 0}),
                last_output: None,
                selected: false,
                executing: false,
                skipped: false,
                error: None,
                execution_state: ExecutionState::default(),
            },
            Node {
                id: NodeId::new(),
                name: "Durable Step".to_string(),
                description: "Create user in database".to_string(),
                node_type: "run".to_string(),
                category: NodeCategory::Durable,
                icon: "shield".to_string(),
                x: 350.0,
                y: 170.0,
                config: serde_json::json!({"configured": true, "status": "completed", "durableStepName": "create-user", "journalIndex": 1}),
                last_output: None,
                selected: false,
                executing: false,
                skipped: false,
                error: None,
                execution_state: ExecutionState::default(),
            },
            Node {
                id: NodeId::new(),
                name: "If / Else".to_string(),
                description: "Check if user creation succeeded".to_string(),
                node_type: "condition".to_string(),
                category: NodeCategory::Flow,
                icon: "git-branch".to_string(),
                x: 350.0,
                y: 300.0,
                config: serde_json::json!({"configured": true, "status": "completed", "journalIndex": 2}),
                last_output: None,
                selected: false,
                executing: false,
                skipped: false,
                error: None,
                execution_state: ExecutionState::default(),
            },
        ],
        connections: vec![],
        viewport: Viewport {
            x: 0.0,
            y: 0.0,
            zoom: 0.85,
        },
        execution_queue: vec![],
        current_step: 0,
        history: vec![],
        execution_records: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::default_workflow;
    use crate::graph::NodeCategory;

    #[test]
    fn given_default_workflow_when_created_then_it_contains_expected_starter_nodes() {
        let workflow = default_workflow();

        assert_eq!(workflow.nodes.len(), 3);
        assert_eq!(workflow.nodes[0].node_type, "http-handler");
        assert_eq!(workflow.nodes[1].node_type, "run");
        assert_eq!(workflow.nodes[2].node_type, "condition");
        assert_eq!(workflow.nodes[0].category, NodeCategory::Entry);
    }

    #[test]
    fn given_default_workflow_when_created_then_viewport_defaults_are_expected() {
        let workflow = default_workflow();

        assert_eq!(workflow.viewport.x, 0.0);
        assert_eq!(workflow.viewport.y, 0.0);
        assert_eq!(workflow.viewport.zoom, 0.85);
    }
}
