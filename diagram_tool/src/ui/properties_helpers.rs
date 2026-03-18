//! Pure helper functions for the properties panel.
//! These functions are separated to keep the main component file under 300 lines
//! and to follow the functional core / imperative shell pattern.

use diagram_models::document::{DiagramDocument, EdgeStyle, NodeId, NodeKind, NodeStyle};
use diagram_models::envelope::EventEnvelope;
use dioxus::prelude::Coroutine;

/// Removes all selected nodes and their connected edges from the document.
#[allow(dead_code)]
pub fn remove_selected(doc: &mut DiagramDocument) {
    let selected = doc.editor_state.selected_items.clone();
    if selected.is_empty() {
        return;
    }

    doc.document.nodes = doc
        .document
        .nodes
        .iter()
        .filter(|(id, _)| !selected.contains(&id.to_string()))
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect();

    let node_ids: im::HashSet<NodeId> = doc.document.nodes.keys().cloned().collect();
    doc.document.edges = doc
        .document
        .edges
        .iter()
        .filter(|(id, edge)| {
            node_ids.contains(&edge.source)
                && node_ids.contains(&edge.target)
                && !selected.contains(&id.to_string())
        })
        .map(|(id, edge)| (id.clone(), edge.clone()))
        .collect();

    doc.editor_state.selected_items.clear();
    doc.revision = doc.revision.increment();
}

/// Error type for style parsing operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum StyleError {
    #[error("Invalid node style: {0}")]
    InvalidNodeStyle(String),
    #[error("Invalid edge style: {0}")]
    InvalidEdgeStyle(String),
    #[error("Invalid arrow type: {0}")]
    InvalidArrowType(String),
}

// === Edge Style Parsing ===

/// Parses a string into an `EdgeStyle`.
#[allow(dead_code)]
#[must_use]
pub fn parse_edge_style(v: &str) -> EdgeStyle {
    match v {
        "dashed" => EdgeStyle::Dashed,
        "dotted" => EdgeStyle::Dotted,
        _ => EdgeStyle::Solid,
    }
}

/// Converts an `EdgeStyle` to its string representation.
#[allow(dead_code)]
#[must_use]
pub const fn edge_style_str(v: EdgeStyle) -> &'static str {
    match v {
        EdgeStyle::Solid => "solid",
        EdgeStyle::Dashed => "dashed",
        EdgeStyle::Dotted => "dotted",
    }
}

// === Arrow Type Parsing ===

/// Parses a string into an `ArrowType`.
#[allow(dead_code)]
#[must_use]
pub fn parse_arrow_type(v: &str) -> diagram_models::document::ArrowType {
    use diagram_models::document::ArrowType;
    match v {
        "straight" | "open" => ArrowType::Straight,
        "step" | "diamond" => ArrowType::Step,
        "curved" | "circle" => ArrowType::Curved,
        "sharp" | "none" => ArrowType::Sharp,
        _ => ArrowType::Default,
    }
}

/// Converts an `ArrowType` to its string representation.
#[allow(dead_code)]
#[must_use]
pub const fn arrow_type_str(v: diagram_models::document::ArrowType) -> &'static str {
    use diagram_models::document::ArrowType;
    match v {
        ArrowType::Default => "default",
        ArrowType::Straight => "straight",
        ArrowType::Step => "step",
        ArrowType::Curved => "curved",
        ArrowType::Sharp => "sharp",
    }
}

// === Node Style Parsing ===

/// Parses a string into a `NodeStyle`, returning an error for invalid values.
#[allow(dead_code)]
pub fn parse_node_style(v: &str) -> Result<NodeStyle, StyleError> {
    match v {
        "box" => Ok(NodeStyle::Box),
        "cloud" => Ok(NodeStyle::Cloud),
        "cylinder" => Ok(NodeStyle::Cylinder),
        "dashed" => Ok(NodeStyle::Dashed),
        _ => Err(StyleError::InvalidNodeStyle(v.to_string())),
    }
}

/// Converts an Option<NodeStyle> to its string representation.
#[allow(dead_code)]
#[must_use]
pub const fn node_style_str(style: &Option<NodeStyle>) -> &'static str {
    match style.as_ref() {
        Some(NodeStyle::Box) => "box",
        Some(NodeStyle::Cloud) => "cloud",
        Some(NodeStyle::Cylinder) => "cylinder",
        Some(NodeStyle::Dashed) => "dashed",
        None => "box",
    }
}

/// Converts a `NodeKind` to its string representation.
#[allow(dead_code)]
#[must_use]
pub const fn node_kind_str(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Node => "node",
        NodeKind::Subgraph => "subgraph",
        NodeKind::Text => "text",
    }
}

// === Dispatch Helpers ===

use crate::ui::dispatch::{dispatch_update_edge_style, dispatch_update_node_style, DispatchResult};

/// Dispatches a node style update to the database transaction.
#[allow(dead_code)]
pub fn dispatch_style_change(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    node_id: &str,
    style: NodeStyle,
) -> Result<DispatchResult, crate::ui::dispatch::DispatchError> {
    dispatch_update_node_style(db_tx, node_id, style)
}

/// Dispatches an edge style update to the database transaction.
#[allow(dead_code)]
pub fn dispatch_edge_style_change(
    db_tx: &Option<Coroutine<EventEnvelope>>,
    edge_id: &str,
    style: EdgeStyle,
) -> Result<DispatchResult, crate::ui::dispatch::DispatchError> {
    dispatch_update_edge_style(db_tx, edge_id, style)
}

// === Node Label Helpers ===

/// Gets the display label for a node, falling back to the node ID if the label is empty.
#[allow(dead_code)]
#[must_use]
pub fn node_label_with_id_fallback(doc: &DiagramDocument, id: &NodeId) -> String {
    doc.document.nodes.get(id).map_or_else(
        || id.to_string(),
        |node| {
            let trimmed = node.label.trim();
            if trimmed.is_empty() {
                id.to_string()
            } else {
                trimmed.to_string()
            }
        },
    )
}
