//! Auto-save module for diagram persistence to localStorage
//!
//! This module provides automatic persistence of diagram state to browser localStorage
//! when changes are detected (via revision tracking).
//!
//! Note: Most of this module is only used in WASM32 builds. The functions are still
//! compiled for all targets but will trigger `dead_code` warnings on non-WASM.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[allow(dead_code)]
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle, Revision};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::ui::editor::ToolMode;

/// Key used for localStorage persistence
#[allow(dead_code)]
pub const AUTO_SAVE_KEY: &str = "diagram_tool.autosave";

/// Schema version for auto-saved data
#[allow(dead_code)]
pub const AUTO_SAVE_VERSION: u32 = 1;

/// The data structure saved to localStorage
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct AutoSavedDiagram {
    pub version: u32,
    pub document: DiagramDocument,
    pub tool_mode: String,
    pub edge_style: EdgeStyle,
    pub arrow_type: ArrowType,
}

impl AutoSavedDiagram {
    /// Create a new auto-saved diagram from current state
    #[cfg(target_arch = "wasm32")]
    pub fn new(
        document: &DiagramDocument,
        tool_mode: &ToolMode,
        edge_style: EdgeStyle,
        arrow_type: ArrowType,
    ) -> Self {
        Self {
            version: AUTO_SAVE_VERSION,
            document: document.clone(),
            tool_mode: tool_mode.persisted_key().to_string(),
            edge_style,
            arrow_type,
        }
    }
}

/// Errors that can occur during auto-save operations
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AutoSaveError {
    /// Serialization to JSON failed
    Serialize(String),
    /// Deserialization from JSON failed
    Deserialize(String),
}

impl std::fmt::Display for AutoSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(msg) => write!(f, "serialize error: {msg}"),
            Self::Deserialize(msg) => write!(f, "deserialize error: {msg}"),
        }
    }
}

/// Serialize diagram to JSON string
///
/// # Errors
///
/// Returns `AutoSaveError::Serialize` if serialization fails.
#[allow(dead_code)]
pub fn serialize_diagram(diagram: &AutoSavedDiagram) -> Result<String, AutoSaveError> {
    serde_json::to_string(diagram).map_err(|e| AutoSaveError::Serialize(e.to_string()))
}

/// Deserialize diagram from JSON string
///
/// # Errors
///
/// Returns `AutoSaveError::Deserialize` if deserialization fails.
#[allow(dead_code)]
pub fn deserialize_diagram(contents: &str) -> Result<AutoSavedDiagram, AutoSaveError> {
    serde_json::from_str(contents).map_err(|e| AutoSaveError::Deserialize(e.to_string()))
}

/// Check if a revision has changed (for tracking)
#[allow(dead_code)]
#[must_use]
pub fn has_revision_changed(current: Revision, previous: Option<Revision>) -> bool {
    previous != Some(current)
}

/// Default revision to use when no history
#[allow(dead_code)]
#[must_use]
pub const fn default_revision() -> Revision {
    Revision::INITIAL
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::DiagramDocument;

    fn sample_document() -> DiagramDocument {
        DiagramDocument::default()
    }

    #[cfg(target_arch = "wasm32")]
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_diagram_when_serializing_then_produces_valid_json() {
        use crate::ui::editor::ToolMode;

        let diagram = AutoSavedDiagram::new(
            &sample_document(),
            &ToolMode::Select,
            EdgeStyle::default(),
            ArrowType::default(),
        );
        let result = serialize_diagram(&diagram);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"version\":1"));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_json_when_deserializing_then_returns_diagram() {
        let diagram = AutoSavedDiagram {
            version: AUTO_SAVE_VERSION,
            document: sample_document(),
            tool_mode: "select".to_string(),
            edge_style: EdgeStyle::default(),
            arrow_type: ArrowType::default(),
        };
        let json = serialize_diagram(&diagram).unwrap();
        let result = deserialize_diagram(&json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, AUTO_SAVE_VERSION);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_json_when_deserializing_then_returns_error() {
        let result = deserialize_diagram("{not-valid-json}");
        assert!(matches!(result, Err(AutoSaveError::Deserialize(_))));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_current_revision_when_comparing_with_none_then_returns_true() {
        let revision = Revision::INITIAL;
        assert!(has_revision_changed(revision, None));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_same_revisions_when_comparing_then_returns_false() {
        let revision = Revision::INITIAL;
        assert!(!has_revision_changed(revision, Some(revision)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_different_revisions_when_comparing_then_returns_true() {
        let current = Revision::INITIAL.increment();
        let previous = Revision::INITIAL;
        assert!(has_revision_changed(current, Some(previous)));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_default_revision_when_creating_then_returns_initial() {
        let default = default_revision();
        assert_eq!(default, Revision::INITIAL);
    }
}
