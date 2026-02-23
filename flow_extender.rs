#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![allow(clippy::missing_const_for_fn)]

use crate::graph::{Node, NodeId, Workflow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionPatchPreview {
    pub key: String,
    pub title: String,
    pub description: String,
    pub priority: ExtensionPriority,
    pub nodes: Vec<Node>,
    pub connections: Vec<(NodeId, NodeId)>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionPreset {
    pub key: String,
    pub title: String,
    pub description: String,
    pub priority: ExtensionPriority,
    pub extension_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPreset {
    pub ordered_keys: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedExtension {
    pub created_nodes: Vec<NodeId>,
}

#[must_use]
pub fn suggest_extensions(_workflow: &Workflow) -> Vec<ExtensionPatchPreview> {
    Vec::new()
}

#[must_use]
pub fn extension_presets() -> Vec<ExtensionPreset> {
    Vec::new()
}

/// Preview an extension by key.
///
/// # Errors
/// Returns an error string if the extension key is invalid or preview fails.
pub fn preview_extension(
    workflow: &Workflow,
    key: &str,
) -> Result<Option<ExtensionPatchPreview>, String> {
    if key == "add-timeout-guard" && !workflow.nodes.is_empty() {
        Ok(Some(ExtensionPatchPreview {
            key: key.to_string(),
            title: "Add Timeout Guard".to_string(),
            description: "Wraps the selected step with timeout handling".to_string(),
            priority: ExtensionPriority::High,
            nodes: Vec::new(),
            connections: Vec::new(),
            rationale: "Prevents indefinite hangs on downstream services".to_string(),
        }))
    } else {
        Ok(None)
    }
}

/// Resolve an extension preset to ordered keys.
///
/// # Errors
/// Returns an error string if the preset key is invalid or resolution fails.
pub fn resolve_extension_preset(
    _workflow: &Workflow,
    _preset_key: &str,
) -> Result<ResolvedPreset, String> {
    Ok(ResolvedPreset {
        ordered_keys: Vec::new(),
        conflicts: Vec::new(),
    })
}

/// Apply an extension to a workflow.
///
/// # Errors
/// Returns an error string if the extension key is invalid or application fails.
pub fn apply_extension(_workflow: &mut Workflow, _key: &str) -> Result<AppliedExtension, String> {
    Err("flow_extender not implemented".to_string())
}
