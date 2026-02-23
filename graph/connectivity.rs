use uuid::Uuid;

use super::{Connection, NodeId, PortName, Workflow};
use crate::graph::restate_types::types_compatible;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError {
    SelfConnection,
    WouldCreateCycle,
    Duplicate,
    TypeMismatch {
        source_type: String,
        target_type: String,
    },
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfConnection => write!(f, "Cannot connect node to itself"),
            Self::WouldCreateCycle => write!(f, "Connection would create a cycle"),
            Self::Duplicate => write!(f, "Connection already exists"),
            Self::TypeMismatch {
                source_type,
                target_type,
            } => write!(
                f,
                "Type mismatch: {source_type} is not compatible with {target_type}"
            ),
        }
    }
}

impl std::error::Error for ConnectionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionResult {
    Created,
    CreatedWithTypeWarning(String),
}

impl Workflow {
    pub fn add_connection(
        &mut self,
        source: NodeId,
        target: NodeId,
        source_port: &PortName,
        target_port: &PortName,
    ) -> bool {
        self.add_connection_checked(source, target, source_port, target_port)
            .is_ok_and(|r| {
                matches!(
                    r,
                    ConnectionResult::Created | ConnectionResult::CreatedWithTypeWarning(_)
                )
            })
    }

    /// Adds a connection with full type checking.
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError` if:
    /// - `source` and `target` are the same node
    /// - The connection would create a cycle
    /// - An identical connection already exists
    pub fn add_connection_checked(
        &mut self,
        source: NodeId,
        target: NodeId,
        source_port: &PortName,
        target_port: &PortName,
    ) -> Result<ConnectionResult, ConnectionError> {
        if source == target {
            return Err(ConnectionError::SelfConnection);
        }
        if Self::path_exists(&self.connections, target, source) {
            return Err(ConnectionError::WouldCreateCycle);
        }
        if self.connections.iter().any(|c| {
            c.source == source
                && c.target == target
                && c.source_port == *source_port
                && c.target_port == *target_port
        }) {
            return Err(ConnectionError::Duplicate);
        }

        let type_warning = Self::check_port_type_compatibility(&self.nodes, source, target);

        self.connections.push(Connection {
            id: Uuid::new_v4(),
            source,
            target,
            source_port: source_port.clone(),
            target_port: target_port.clone(),
        });

        type_warning.map_or(Ok(ConnectionResult::Created), |warning| {
            Ok(ConnectionResult::CreatedWithTypeWarning(warning))
        })
    }

    fn check_port_type_compatibility(
        nodes: &[super::Node],
        source: NodeId,
        target: NodeId,
    ) -> Option<String> {
        let source_node = nodes.iter().find(|n| n.id == source)?;
        let target_node = nodes.iter().find(|n| n.id == target)?;

        let source_type = Self::get_node_output_port_type(source_node)?;
        let target_type = Self::get_node_input_port_type(target_node)?;

        if !types_compatible(source_type, target_type) {
            return Some(format!("Type warning: {source_type} -> {target_type}"));
        }
        None
    }

    fn get_node_output_port_type(
        node: &super::Node,
    ) -> Option<crate::graph::restate_types::PortType> {
        let workflow_node: crate::graph::workflow_node::WorkflowNode =
            node.node_type.parse().ok()?;
        Some(workflow_node.output_port_type())
    }

    fn get_node_input_port_type(
        node: &super::Node,
    ) -> Option<crate::graph::restate_types::PortType> {
        let workflow_node: crate::graph::workflow_node::WorkflowNode =
            node.node_type.parse().ok()?;
        Some(workflow_node.input_port_type())
    }

    fn path_exists(connections: &[Connection], from: NodeId, to: NodeId) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![from];

        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }

            if visited.insert(current) {
                connections
                    .iter()
                    .filter(|connection| connection.source == current)
                    .for_each(|connection| stack.push(connection.target));
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_self_connection_when_adding_checked_connection_then_self_connection_error_is_returned()
    {
        let mut workflow = Workflow::new();
        let node_id = workflow.add_node("http-handler", 0.0, 0.0);
        let main = PortName("main".to_string());

        let result = workflow.add_connection_checked(node_id, node_id, &main, &main);

        assert_eq!(result, Err(ConnectionError::SelfConnection));
    }

    #[test]
    fn given_duplicate_connection_when_adding_checked_connection_then_duplicate_error_is_returned()
    {
        let mut workflow = Workflow::new();
        let source = workflow.add_node("http-handler", 0.0, 0.0);
        let target = workflow.add_node("run", 100.0, 0.0);
        let main = PortName("main".to_string());

        let first = workflow.add_connection_checked(source, target, &main, &main);
        assert!(matches!(first, Ok(ConnectionResult::Created)));

        let duplicate = workflow.add_connection_checked(source, target, &main, &main);

        assert_eq!(duplicate, Err(ConnectionError::Duplicate));
    }

    #[test]
    fn given_back_edge_when_adding_checked_connection_then_cycle_error_is_returned() {
        let mut workflow = Workflow::new();
        let first = workflow.add_node("http-handler", 0.0, 0.0);
        let second = workflow.add_node("run", 100.0, 0.0);
        let main = PortName("main".to_string());

        let created = workflow.add_connection_checked(first, second, &main, &main);
        assert!(matches!(created, Ok(ConnectionResult::Created)));

        let cycle = workflow.add_connection_checked(second, first, &main, &main);

        assert_eq!(cycle, Err(ConnectionError::WouldCreateCycle));
    }

    #[test]
    fn given_type_mismatch_ports_when_adding_checked_connection_then_warning_result_is_returned() {
        let mut workflow = Workflow::new();
        let source = workflow.add_node("condition", 0.0, 0.0);
        let target = workflow.add_node("signal-handler", 100.0, 0.0);
        let main = PortName("main".to_string());

        let result = workflow.add_connection_checked(source, target, &main, &main);

        assert!(matches!(
            result,
            Ok(ConnectionResult::CreatedWithTypeWarning(_))
        ));
    }
}
