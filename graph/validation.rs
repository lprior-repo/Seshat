#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::{Node, NodeCategory, NodeId, Workflow};
use crate::graph::workflow_node::WorkflowNode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub node_id: Option<NodeId>,
}

impl ValidationIssue {
    #[must_use]
    pub const fn error(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            message,
            node_id: None,
        }
    }

    #[must_use]
    pub const fn error_for_node(message: String, node_id: NodeId) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            message,
            node_id: Some(node_id),
        }
    }

    #[must_use]
    pub const fn warning(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            message,
            node_id: None,
        }
    }

    #[must_use]
    pub const fn warning_for_node(message: String, node_id: NodeId) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            message,
            node_id: Some(node_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Warning)
    }

    #[must_use]
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .count()
    }

    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .count()
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }
}

#[must_use]
pub fn validate_workflow(workflow: &Workflow) -> ValidationResult {
    let mut issues = Vec::new();

    if workflow.nodes.is_empty() {
        issues.push(ValidationIssue::error("Workflow has no nodes".to_string()));
        return ValidationResult { issues };
    }

    validate_entry_points(workflow, &mut issues);
    validate_reachability(workflow, &mut issues);
    validate_orphan_nodes(workflow, &mut issues);
    validate_required_config(workflow, &mut issues);
    validate_connection_validity(workflow, &mut issues);

    ValidationResult { issues }
}

fn validate_entry_points(workflow: &Workflow, issues: &mut Vec<ValidationIssue>) {
    if !workflow
        .nodes
        .iter()
        .any(|n| n.category == NodeCategory::Entry)
    {
        issues.push(ValidationIssue::error(
            "Workflow has no entry point (e.g., HTTP Handler, Kafka Handler)".to_string(),
        ));
    }
}

fn validate_reachability(workflow: &Workflow, issues: &mut Vec<ValidationIssue>) {
    if workflow.nodes.is_empty() || workflow.connections.is_empty() {
        return;
    }

    let entry_ids: HashSet<NodeId> = workflow
        .nodes
        .iter()
        .filter(|n| n.category == NodeCategory::Entry)
        .map(|n| n.id)
        .collect();

    if entry_ids.is_empty() {
        return;
    }

    let mut reachable = HashSet::new();
    let mut stack: Vec<NodeId> = entry_ids.iter().copied().collect();

    while let Some(current) = stack.pop() {
        if reachable.insert(current) {
            for conn in workflow.connections.iter().filter(|c| c.source == current) {
                if !reachable.contains(&conn.target) {
                    stack.push(conn.target);
                }
            }
        }
    }

    for node in &workflow.nodes {
        if !reachable.contains(&node.id) && node.category != NodeCategory::Entry {
            let has_incoming = workflow.connections.iter().any(|c| c.target == node.id);
            if !has_incoming {
                issues.push(ValidationIssue::warning_for_node(
                    format!("Node '{}' is not reachable from any entry point", node.name),
                    node.id,
                ));
            }
        }
    }
}

fn validate_orphan_nodes(workflow: &Workflow, issues: &mut Vec<ValidationIssue>) {
    for node in &workflow.nodes {
        if node.category == NodeCategory::Entry {
            continue;
        }

        let has_incoming = workflow.connections.iter().any(|c| c.target == node.id);
        let has_outgoing = workflow.connections.iter().any(|c| c.source == node.id);

        if !has_incoming && !has_outgoing && workflow.nodes.len() > 1 {
            issues.push(ValidationIssue::warning_for_node(
                format!("Node '{}' is not connected to anything", node.name),
                node.id,
            ));
        } else if !has_incoming && workflow.nodes.len() > 1 {
            issues.push(ValidationIssue::warning_for_node(
                format!("Node '{}' has no incoming connections", node.name),
                node.id,
            ));
        }
    }
}

fn validate_required_config(workflow: &Workflow, issues: &mut Vec<ValidationIssue>) {
    for node in &workflow.nodes {
        let Ok(workflow_node) = node.node_type.parse::<WorkflowNode>() else {
            issues.push(ValidationIssue::error_for_node(
                format!("Unknown node type: {}", node.node_type),
                node.id,
            ));
            continue;
        };

        validate_node_config(&workflow_node, node, issues);
    }
}

#[allow(clippy::too_many_lines)]
fn validate_node_config(
    workflow_node: &WorkflowNode,
    node: &Node,
    issues: &mut Vec<ValidationIssue>,
) {
    match workflow_node {
        WorkflowNode::HttpHandler(config) => {
            if config.path.is_none()
                || config
                    .path
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "HTTP Handler requires a path".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::KafkaHandler(config) => {
            if config.topic.is_none()
                || config
                    .topic
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Kafka Handler requires a topic".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::CronTrigger(config) => {
            if config.schedule.is_none()
                || config
                    .schedule
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Cron Trigger requires a schedule".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::WorkflowSubmit(config) => {
            if config.workflow_name.is_none()
                || config
                    .workflow_name
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Workflow Submit requires a workflow name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::ServiceCall(config) => {
            if config.service.is_none()
                || config
                    .service
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Service Call requires a service name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::ObjectCall(config) => {
            if config.object_name.is_none()
                || config
                    .object_name
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Object Call requires an object name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::WorkflowCall(config) => {
            if config.workflow_name.is_none()
                || config
                    .workflow_name
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Workflow Call requires a workflow name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::SendMessage(config) => {
            if config.target.is_none()
                || config
                    .target
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Send Message requires a target".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::DelayedSend(config) => {
            if config.target.is_none()
                || config
                    .target
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Delayed Send requires a target".to_string(),
                    node.id,
                ));
            }
            if config.delay_ms.is_none() || config.delay_ms.is_some_and(|d| d == 0) {
                issues.push(ValidationIssue::warning_for_node(
                    "Delayed Send should have a non-zero delay".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::GetState(config) => {
            if config.key.is_none()
                || config
                    .key
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Get State requires a key".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::SetState(config) => {
            if config.key.is_none()
                || config
                    .key
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Set State requires a key".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::ClearState(config) => {
            if config.key.is_none()
                || config
                    .key
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Clear State requires a key".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Condition(config) => {
            if config.expression.is_none()
                || config
                    .expression
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Condition requires an expression".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Switch(config) => {
            if config.expression.is_none()
                || config
                    .expression
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Switch requires an expression".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Loop(config) => {
            if config.iterator.is_none()
                || config
                    .iterator
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::warning_for_node(
                    "Loop should have an iterator expression".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Parallel(config) => {
            if config.branches.is_none() || config.branches.is_some_and(|b| b == 0) {
                issues.push(ValidationIssue::warning_for_node(
                    "Parallel should have at least one branch".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Sleep(config) => {
            if config.duration_ms.is_none() || config.duration_ms.is_some_and(|d| d == 0) {
                issues.push(ValidationIssue::warning_for_node(
                    "Sleep should have a non-zero duration".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Timeout(config) => {
            if config.timeout_ms.is_none() || config.timeout_ms.is_some_and(|t| t == 0) {
                issues.push(ValidationIssue::warning_for_node(
                    "Timeout should have a non-zero duration".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::DurablePromise(config) => {
            if config.promise_name.is_none()
                || config
                    .promise_name
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Durable Promise requires a promise name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Awakeable(config) => {
            if config.awakeable_id.is_none()
                || config
                    .awakeable_id
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Awakeable requires an awakeable ID".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::ResolvePromise(config) => {
            if config.promise_name.is_none()
                || config
                    .promise_name
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Resolve Promise requires a promise name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::SignalHandler(config) => {
            if config.signal_name.is_none()
                || config
                    .signal_name
                    .as_ref()
                    .is_some_and(std::string::String::is_empty)
            {
                issues.push(ValidationIssue::error_for_node(
                    "Signal Handler requires a signal name".to_string(),
                    node.id,
                ));
            }
        }
        WorkflowNode::Run(_) | WorkflowNode::Compensate(_) => {}
    }
}

fn validate_connection_validity(workflow: &Workflow, issues: &mut Vec<ValidationIssue>) {
    let node_ids: HashSet<NodeId> = workflow.nodes.iter().map(|n| n.id).collect();

    for conn in &workflow.connections {
        if !node_ids.contains(&conn.source) {
            issues.push(ValidationIssue::error(
                "Connection references non-existent source node".to_string(),
            ));
        }
        if !node_ids.contains(&conn.target) {
            issues.push(ValidationIssue::error(
                "Connection references non-existent target node".to_string(),
            ));
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn make_workflow() -> Workflow {
        Workflow::default()
    }

    fn add_entry_node(wf: &mut Workflow) -> NodeId {
        wf.add_node("http-handler", 0.0, 0.0)
    }

    fn add_non_entry_node(wf: &mut Workflow) -> NodeId {
        wf.add_node("run", 200.0, 0.0)
    }

    mod empty_workflow {
        use super::*;

        #[test]
        fn given_empty_workflow_when_validating_then_has_error() {
            let workflow = make_workflow();
            let result = validate_workflow(&workflow);

            assert!(result.has_errors());
            assert!(!result.is_valid());
            assert_eq!(result.error_count(), 1);
            assert!(result.issues[0].message.contains("no nodes"));
        }
    }

    mod entry_point_validation {
        use super::*;

        #[test]
        fn given_workflow_without_entry_point_when_validating_then_has_error() {
            let mut workflow = make_workflow();
            let _ = add_non_entry_node(&mut workflow);

            let result = validate_workflow(&workflow);

            assert!(result.has_errors());
            assert!(result
                .issues
                .iter()
                .any(|i| i.message.contains("no entry point")));
        }

        #[test]
        fn given_workflow_with_entry_point_when_validating_then_no_entry_error() {
            let mut workflow = make_workflow();
            let _ = add_entry_node(&mut workflow);

            let result = validate_workflow(&workflow);

            assert!(!result
                .issues
                .iter()
                .any(|i| i.message.contains("no entry point")));
        }
    }

    mod reachability {
        use super::*;

        #[test]
        fn given_connected_nodes_when_validating_then_reachable() {
            let mut workflow = make_workflow();
            let entry_id = add_entry_node(&mut workflow);
            let run_id = add_non_entry_node(&mut workflow);
            workflow.add_connection(
                entry_id,
                run_id,
                &crate::graph::PortName("main".to_string()),
                &crate::graph::PortName("main".to_string()),
            );

            let result = validate_workflow(&workflow);

            assert!(!result
                .issues
                .iter()
                .any(|i| i.message.contains("not reachable")));
        }

        #[test]
        fn given_disconnected_node_when_validating_then_unreachable_warning() {
            let mut workflow = make_workflow();
            let _ = add_entry_node(&mut workflow);
            let _ = add_non_entry_node(&mut workflow);

            let result = validate_workflow(&workflow);

            assert!(result.has_warnings());
        }
    }

    mod config_validation {
        use super::*;

        #[test]
        fn given_http_handler_without_path_when_validating_then_error() {
            let mut workflow = make_workflow();
            let node_id = workflow.add_node("http-handler", 0.0, 0.0);

            let result = validate_workflow(&workflow);

            assert!(result.has_errors());
            assert!(result
                .issues
                .iter()
                .any(|i| i.node_id == Some(node_id) && i.message.contains("path")));
        }

        #[test]
        fn given_kafka_handler_without_topic_when_validating_then_error() {
            let mut workflow = make_workflow();
            let node_id = workflow.add_node("kafka-handler", 0.0, 0.0);

            let result = validate_workflow(&workflow);

            assert!(result.has_errors());
            assert!(result
                .issues
                .iter()
                .any(|i| i.node_id == Some(node_id) && i.message.contains("topic")));
        }

        #[test]
        fn given_cron_trigger_without_schedule_when_validating_then_error() {
            let mut workflow = make_workflow();
            let node_id = workflow.add_node("cron-trigger", 0.0, 0.0);

            let result = validate_workflow(&workflow);

            assert!(result.has_errors());
            assert!(result
                .issues
                .iter()
                .any(|i| i.node_id == Some(node_id) && i.message.contains("schedule")));
        }

        #[test]
        fn given_unknown_node_type_when_validating_then_error_for_node_is_emitted() {
            let mut workflow = make_workflow();
            let _ = add_entry_node(&mut workflow);
            let node_id = workflow.add_node("unknown-node-type", 120.0, 0.0);

            let result = validate_workflow(&workflow);

            assert!(result.issues.iter().any(|issue| {
                issue.severity == ValidationSeverity::Error
                    && issue.node_id == Some(node_id)
                    && issue.message.contains("Unknown node type")
            }));
        }

        #[test]
        fn given_delayed_send_zero_delay_when_validating_then_warning_is_emitted() {
            let mut workflow = make_workflow();
            let _ = add_entry_node(&mut workflow);
            let node_id = workflow.add_node("delayed-send", 160.0, 0.0);
            if let Some(node) = workflow.nodes.iter_mut().find(|node| node.id == node_id) {
                node.config = serde_json::json!({
                    "target": "queue-name",
                    "delay_ms": 0
                });
            }

            let result = validate_workflow(&workflow);

            assert!(result.issues.iter().any(|issue| {
                issue.severity == ValidationSeverity::Warning
                    && issue.node_id == Some(node_id)
                    && issue.message.contains("non-zero delay")
            }));
        }
    }

    mod orphan_validation {
        use super::*;

        #[test]
        fn given_orphan_non_entry_node_when_validating_then_orphan_warning_is_emitted() {
            let mut workflow = make_workflow();
            let _ = add_entry_node(&mut workflow);
            let orphan_id = add_non_entry_node(&mut workflow);

            let result = validate_workflow(&workflow);

            assert!(result.issues.iter().any(|issue| {
                issue.severity == ValidationSeverity::Warning
                    && issue.node_id == Some(orphan_id)
                    && (issue.message.contains("not connected")
                        || issue.message.contains("no incoming"))
            }));
        }
    }

    mod connection_validation {
        use super::*;
        use crate::graph::{Connection, PortName};
        use uuid::Uuid;

        #[test]
        fn given_connection_with_missing_source_when_validating_then_source_error_is_reported() {
            let mut workflow = make_workflow();
            let target = add_entry_node(&mut workflow);
            workflow.connections.push(Connection {
                id: Uuid::new_v4(),
                source: NodeId::new(),
                target,
                source_port: PortName("main".to_string()),
                target_port: PortName("main".to_string()),
            });

            let result = validate_workflow(&workflow);

            assert!(result.issues.iter().any(|issue| {
                issue.severity == ValidationSeverity::Error
                    && issue.message.contains("non-existent source")
            }));
        }

        #[test]
        fn given_connection_with_missing_target_when_validating_then_target_error_is_reported() {
            let mut workflow = make_workflow();
            let source = add_entry_node(&mut workflow);
            workflow.connections.push(Connection {
                id: Uuid::new_v4(),
                source,
                target: NodeId::new(),
                source_port: PortName("main".to_string()),
                target_port: PortName("main".to_string()),
            });

            let result = validate_workflow(&workflow);

            assert!(result.issues.iter().any(|issue| {
                issue.severity == ValidationSeverity::Error
                    && issue.message.contains("non-existent target")
            }));
        }
    }

    mod validation_result {
        use super::*;

        #[test]
        fn given_empty_issues_when_checking_then_valid() {
            let result = ValidationResult::default();
            assert!(result.is_valid());
            assert!(!result.has_errors());
            assert!(!result.has_warnings());
        }

        #[test]
        fn given_only_warnings_when_checking_then_valid() {
            let result = ValidationResult {
                issues: vec![ValidationIssue::warning("test".to_string())],
            };
            assert!(result.is_valid());
            assert!(!result.has_errors());
            assert!(result.has_warnings());
        }

        #[test]
        fn given_error_when_checking_then_invalid() {
            let result = ValidationResult {
                issues: vec![ValidationIssue::error("test".to_string())],
            };
            assert!(!result.is_valid());
            assert!(result.has_errors());
        }

        #[test]
        fn given_mixed_issues_when_counting_then_correct_counts() {
            let result = ValidationResult {
                issues: vec![
                    ValidationIssue::error("e1".to_string()),
                    ValidationIssue::warning("w1".to_string()),
                    ValidationIssue::error("e2".to_string()),
                    ValidationIssue::warning("w2".to_string()),
                ],
            };
            assert_eq!(result.error_count(), 2);
            assert_eq!(result.warning_count(), 2);
        }
    }
}
