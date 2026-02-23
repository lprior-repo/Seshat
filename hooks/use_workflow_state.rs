#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::errors::{WorkflowError, WorkflowResult};
use crate::graph::{
    Connection, ConnectionError, ConnectionResult, Node, NodeId, PortName, Viewport, Workflow,
};
use dioxus::prelude::*;
use std::collections::HashMap;

fn push_undo_snapshot(undo_stack: &mut Vec<Workflow>, snapshot: Workflow, cap: usize) {
    undo_stack.push(snapshot);
    if undo_stack.len() > cap {
        let _ = undo_stack.remove(0);
    }
}

fn apply_undo(
    workflow: &mut Workflow,
    undo_stack: &mut Vec<Workflow>,
    redo_stack: &mut Vec<Workflow>,
) -> bool {
    undo_stack.pop().is_some_and(|snapshot| {
        let current = workflow.clone();
        redo_stack.push(current);
        *workflow = snapshot;
        true
    })
}

fn apply_redo(
    workflow: &mut Workflow,
    undo_stack: &mut Vec<Workflow>,
    redo_stack: &mut Vec<Workflow>,
) -> bool {
    redo_stack.pop().is_some_and(|snapshot| {
        let current = workflow.clone();
        undo_stack.push(current);
        *workflow = snapshot;
        true
    })
}

/// Workflow state hook - manages workflow data, undo/redo, and derived views.
///
/// This follows the functional reactive pattern where:
/// - State is stored in Copy signals
/// - Actions are methods that mutate state immutably (clone, modify, set)
/// - Derived data is computed via Memo for performance
#[derive(Clone, Copy, PartialEq)]
pub struct WorkflowState {
    workflow: Signal<Workflow>,
    workflow_name: Signal<String>,
    undo_stack: Signal<Vec<Workflow>>,
    redo_stack: Signal<Vec<Workflow>>,
    nodes: Memo<Vec<Node>>,
    nodes_by_id: Memo<HashMap<NodeId, Node>>,
    connections: Memo<Vec<Connection>>,
    viewport: Memo<Viewport>,
}

async fn run_workflow_detached(mut workflow: Workflow) -> Workflow {
    workflow.run().await;
    workflow
}

impl WorkflowState {
    /// Access to workflow data signal
    #[must_use]
    pub const fn workflow(&self) -> Signal<Workflow> {
        self.workflow
    }

    /// Access to workflow name signal
    #[must_use]
    pub const fn workflow_name(&self) -> Signal<String> {
        self.workflow_name
    }

    /// Read-only access to nodes list (memoized)
    #[must_use]
    pub fn nodes(&self) -> ReadSignal<Vec<Node>> {
        self.nodes.into()
    }

    /// Read-only access to nodes by ID map (memoized)
    #[must_use]
    pub fn nodes_by_id(&self) -> ReadSignal<HashMap<NodeId, Node>> {
        self.nodes_by_id.into()
    }

    /// Read-only access to connections (memoized)
    #[must_use]
    pub fn connections(&self) -> ReadSignal<Vec<Connection>> {
        self.connections.into()
    }

    /// Read-only access to viewport (memoized)
    #[must_use]
    pub fn viewport(&self) -> ReadSignal<Viewport> {
        self.viewport.into()
    }

    /// Access to undo stack signal
    #[must_use]
    pub const fn undo_stack(&self) -> Signal<Vec<Workflow>> {
        self.undo_stack
    }

    /// Access to redo stack signal
    #[must_use]
    pub const fn redo_stack(&self) -> Signal<Vec<Workflow>> {
        self.redo_stack
    }

    /// Save current state to undo stack before mutation
    pub fn save_undo_point(mut self) {
        let current = self.workflow.read().clone();
        push_undo_snapshot(&mut self.undo_stack.write(), current, 60);
        self.redo_stack.write().clear();
    }

    /// Add a new node at the specified position
    #[must_use]
    pub fn add_node(mut self, node_type: &str, x: f32, y: f32) -> NodeId {
        self.save_undo_point();
        self.workflow.write().add_node(node_type, x, y)
    }

    /// Add a node at the viewport center
    #[must_use]
    pub fn add_node_at_viewport_center(mut self, node_type: &str) -> NodeId {
        self.save_undo_point();
        let vp = self.workflow.read().viewport.clone();
        self.workflow.write().add_node(node_type, vp.x, vp.y)
    }

    /// Remove a node by ID
    ///
    /// # Errors
    /// Returns `WorkflowError::NodeNotFound` if the node does not exist.
    pub fn remove_node(mut self, node_id: NodeId) -> WorkflowResult<()> {
        let exists = self.workflow.read().nodes.iter().any(|n| n.id == node_id);
        if !exists {
            return Err(WorkflowError::NodeNotFound(node_id));
        }
        self.save_undo_point();
        self.workflow.write().remove_node(node_id);
        Ok(())
    }

    /// Add a connection between two nodes
    ///
    /// # Errors
    /// Returns `WorkflowError` if the connection is invalid (e.g., creates a cycle, duplicate connection, or invalid ports).
    pub fn add_connection(
        mut self,
        source: NodeId,
        target: NodeId,
        source_port: &PortName,
        target_port: &PortName,
    ) -> WorkflowResult<()> {
        self.save_undo_point();
        let result =
            self.workflow
                .write()
                .add_connection_checked(source, target, source_port, target_port);

        match result {
            Ok(ConnectionResult::Created | ConnectionResult::CreatedWithTypeWarning(_)) => Ok(()),
            Err(error) => Err(map_connection_error(error)),
        }
    }

    /// Zoom the viewport
    pub fn zoom(mut self, delta: f32, center_x: f32, center_y: f32) {
        self.workflow.write().zoom(delta, center_x, center_y);
    }

    /// Pan the viewport
    pub fn pan(mut self, dx: f32, dy: f32) {
        self.workflow.write().viewport.x += dx;
        self.workflow.write().viewport.y += dy;
    }

    /// Fit view to show all nodes
    pub fn fit_view(mut self, width: f32, height: f32, padding: f32) {
        self.workflow.write().fit_view(width, height, padding);
    }

    /// Apply auto-layout to nodes
    pub fn apply_layout(mut self) {
        self.save_undo_point();
        self.workflow.write().apply_layout();
    }

    /// Undo last action
    #[must_use]
    pub fn undo(mut self) -> bool {
        let mut workflow = self.workflow.read().clone();
        let did_undo = apply_undo(
            &mut workflow,
            &mut self.undo_stack.write(),
            &mut self.redo_stack.write(),
        );
        if did_undo {
            self.workflow.set(workflow);
        }
        did_undo
    }

    /// Redo last undone action
    #[must_use]
    pub fn redo(mut self) -> bool {
        let mut workflow = self.workflow.read().clone();
        let did_redo = apply_redo(
            &mut workflow,
            &mut self.undo_stack.write(),
            &mut self.redo_stack.write(),
        );
        if did_redo {
            self.workflow.set(workflow);
        }
        did_redo
    }

    /// Check if undo is available
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.read().is_empty()
    }

    /// Check if redo is available
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.read().is_empty()
    }

    /// Update node position
    pub fn update_node_position(mut self, node_id: NodeId, dx: f32, dy: f32) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.workflow.write().update_node_position(node_id, dx, dy);
    }

    /// Run the workflow asynchronously
    pub fn run(self) {
        let mut workflow_signal = self.workflow;
        let workflow_snapshot = workflow_signal.read().clone();

        spawn(async move {
            let workflow_result = run_workflow_detached(workflow_snapshot).await;
            workflow_signal.set(workflow_result);
        });
    }

    /// Find downstream nodes (nodes connected FROM the given node)
    #[must_use]
    pub fn downstream_nodes(&self, node_id: NodeId) -> Vec<NodeId> {
        self.connections
            .read()
            .iter()
            .filter(|c| c.source == node_id)
            .map(|c| c.target)
            .collect()
    }

    /// Find upstream nodes (nodes connected TO the given node)
    #[must_use]
    pub fn upstream_nodes(&self, node_id: NodeId) -> Vec<NodeId> {
        self.connections
            .read()
            .iter()
            .filter(|c| c.target == node_id)
            .map(|c| c.source)
            .collect()
    }

    /// Move a node by a delta amount (for keyboard navigation)
    pub fn move_node_by(mut self, node_id: NodeId, dx: f32, dy: f32) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        if let Some(node) = self
            .workflow
            .write()
            .nodes
            .iter_mut()
            .find(|n| n.id == node_id)
        {
            node.x += dx;
            node.y += dy;
        }
    }

    /// Get the first node in the workflow (for initial selection)
    #[must_use]
    pub fn first_node_id(&self) -> Option<NodeId> {
        self.nodes.read().first().map(|n| n.id)
    }
}

fn map_connection_error(error: ConnectionError) -> WorkflowError {
    match error {
        ConnectionError::SelfConnection => WorkflowError::SelfConnection,
        ConnectionError::WouldCreateCycle => WorkflowError::CycleDetected,
        ConnectionError::Duplicate => WorkflowError::DuplicateConnection,
        ConnectionError::TypeMismatch {
            source_type,
            target_type,
        } => WorkflowError::InvalidConnection(format!(
            "Type mismatch: {source_type} is not compatible with {target_type}"
        )),
    }
}

pub fn use_workflow_state() -> WorkflowState {
    let workflow = use_signal(|| {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;
            let storage = window().and_then(|w| w.local_storage().ok()).flatten();
            if let Some(s) = storage {
                match s.get_item("flow-wasm-v1-workflow") {
                    Ok(Some(json)) => {
                        if let Ok(parsed) = serde_json::from_str::<Workflow>(&json) {
                            return parsed;
                        }
                    }
                    _ => {}
                }
            }
        }
        crate::ui::app_bootstrap::default_workflow()
    });

    let workflow_name = use_signal(|| "SignupWorkflow".to_string());
    let undo_stack = use_signal(Vec::<Workflow>::new);
    let redo_stack = use_signal(Vec::<Workflow>::new);

    // Derived memos for performance
    let nodes = use_memo(move || workflow.read().nodes.clone());
    let nodes_by_id = use_memo(move || {
        workflow
            .read()
            .nodes
            .iter()
            .map(|n| (n.id, n.clone()))
            .collect()
    });
    let connections = use_memo(move || workflow.read().connections.clone());
    let viewport = use_memo(move || workflow.read().viewport.clone());

    WorkflowState {
        workflow,
        workflow_name,
        undo_stack,
        redo_stack,
        nodes,
        nodes_by_id,
        connections,
        viewport,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_redo, apply_undo, map_connection_error, push_undo_snapshot, run_workflow_detached,
    };
    use crate::errors::WorkflowError;
    use crate::graph::ConnectionError;
    use crate::graph::Workflow;

    #[tokio::test]
    async fn given_detached_workflow_when_running_then_history_is_recorded() {
        let mut workflow = Workflow::new();
        let _ = workflow.add_node("http-handler", 0.0, 0.0);

        let updated = run_workflow_detached(workflow).await;

        assert_eq!(updated.history.len(), 1);
        assert!(updated.history[0].success);
    }

    #[test]
    fn given_workflow_state_source_when_running_async_then_no_write_guard_spans_await() {
        let source = include_str!("use_workflow_state.rs");
        let bad_pattern = [".write()", ".run()", ".await"].join("");

        assert!(!source.contains(&bad_pattern));
    }

    #[test]
    fn given_connection_error_when_mapping_to_workflow_error_then_taxonomy_is_preserved() {
        assert_eq!(
            map_connection_error(ConnectionError::SelfConnection),
            WorkflowError::SelfConnection
        );
        assert_eq!(
            map_connection_error(ConnectionError::WouldCreateCycle),
            WorkflowError::CycleDetected
        );
        assert_eq!(
            map_connection_error(ConnectionError::Duplicate),
            WorkflowError::DuplicateConnection
        );

        let mismatch = map_connection_error(ConnectionError::TypeMismatch {
            source_type: "event".to_string(),
            target_type: "signal".to_string(),
        });
        assert!(matches!(mismatch, WorkflowError::InvalidConnection(_)));
    }

    #[test]
    fn given_more_than_sixty_snapshots_when_pushing_undo_then_stack_is_capped() {
        let mut undo_stack = Vec::new();
        let workflow = Workflow::new();

        for _ in 0..65 {
            push_undo_snapshot(&mut undo_stack, workflow.clone(), 60);
        }

        assert_eq!(undo_stack.len(), 60);
    }

    #[test]
    fn given_undo_then_redo_sequence_when_applied_then_snapshots_restore_correctly() {
        let mut workflow = Workflow::new();
        let mut undo_stack = Vec::new();
        let mut redo_stack = Vec::new();

        let mut older = Workflow::new();
        let _ = older.add_node("http-handler", 0.0, 0.0);
        let mut newer = Workflow::new();
        let _ = newer.add_node("http-handler", 0.0, 0.0);
        let _ = newer.add_node("run", 0.0, 0.0);

        workflow.clone_from(&newer);
        undo_stack.push(older.clone());

        assert!(apply_undo(&mut workflow, &mut undo_stack, &mut redo_stack));
        assert_eq!(workflow.nodes.len(), 1);
        assert_eq!(redo_stack.len(), 1);

        assert!(apply_redo(&mut workflow, &mut undo_stack, &mut redo_stack));
        assert_eq!(workflow.nodes.len(), 2);
    }
}
