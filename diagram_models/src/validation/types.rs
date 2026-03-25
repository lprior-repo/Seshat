use std::fmt;

/// Severity of a validation issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warning,
    Error,
}

impl PartialOrd for ValidationSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValidationSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Error, Self::Error) | (Self::Warning, Self::Warning) => {
                std::cmp::Ordering::Equal
            }
            (Self::Error, Self::Warning) => std::cmp::Ordering::Greater,
            (Self::Warning, Self::Error) => std::cmp::Ordering::Less,
        }
    }
}

/// Strongly typed validation code, eliminating primitive obsession.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ValidationCode(pub std::borrow::Cow<'static, str>);

impl ValidationCode {
    pub const EDGE_DANGLING: Self = Self(std::borrow::Cow::Borrowed("edge-dangling"));
    pub const INVALID_PARENT: Self = Self(std::borrow::Cow::Borrowed("invalid-parent"));
    pub const INVALID_NUMERIC: Self = Self(std::borrow::Cow::Borrowed("invalid-numeric"));
    pub const DAG_CYCLE: Self = Self(std::borrow::Cow::Borrowed("dag-cycle"));
    pub const DAG_DISCONNECTED: Self = Self(std::borrow::Cow::Borrowed("dag-disconnected"));
    pub const INTERNAL_ERROR: Self = Self(std::borrow::Cow::Borrowed("internal-error"));
    pub const SCHEMA: Self = Self(std::borrow::Cow::Borrowed("schema"));
    pub const INVALID_VERSION: Self = Self(std::borrow::Cow::Borrowed("invalid-version"));
    pub const PARENT_CYCLE: Self = Self(std::borrow::Cow::Borrowed("parent-cycle"));
    pub const EDGE_INVALID_OFFSET: Self = Self(std::borrow::Cow::Borrowed("edge-invalid-offset"));
    pub const EDGE_INVALID_THICKNESS: Self =
        Self(std::borrow::Cow::Borrowed("edge-invalid-thickness"));
    pub const EDGE_INVALID_COLOR: Self = Self(std::borrow::Cow::Borrowed("edge-invalid-color"));
    pub const EDGE_INVALID_FONT_SIZE: Self =
        Self(std::borrow::Cow::Borrowed("edge-invalid-font-size"));
    pub const EDITOR_INVALID_STATE: Self = Self(std::borrow::Cow::Borrowed("editor-invalid-state"));

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a template string with placeholders for fix parameters.
    /// Returns None for unknown codes.
    #[must_use]
    pub fn default_fix_hint(&self) -> Option<&'static str> {
        match self.0.as_ref() {
            "edge-dangling" => Some("Remove edge {edge_id} or create missing node {missing_node_id}"),
            "invalid-parent" => Some("Set node {node_id}.parent to None or reference an existing Subgraph node"),
            "invalid-numeric" => Some("Ensure numeric field {field_name} is finite. Got: {actual_value}"),
            "dag-cycle" => Some("Remove one edge in the cycle to break it. Cycle path: {cycle_path}"),
            "dag-disconnected" => Some("Add edges to connect all {n} components into a single connected graph"),
            "internal-error" => Some("This is a bug. Report to developers with reproduction steps."),
            "schema" => Some("Fix schema violation at {path}: expected {expected}, got {actual}"),
            "invalid-version" => Some("Set document version to 2. Current: {actual}"),
            "parent-cycle" => Some("Break the parent chain cycle by setting {node_id}.parent = None"),
            "edge-invalid-offset" => Some("Set edge {edge_id}.label_offset_t to a value in [0.0, 1.0]. Got: {actual}"),
            "edge-invalid-thickness" => Some("Set edge {edge_id}.thickness to a finite non-negative value. Got: {actual}"),
            "edge-invalid-color" => Some("Set edge {edge_id}.color to hex format #RGB, #RGBA, #RRGGBB, or #RRGGBBAA. Got: {actual}"),
            "edge-invalid-font-size" => Some("Set edge {edge_id}.font_size to a finite value. Got: {actual}"),
            "editor-invalid-state" => Some("Set editor.{field} to a finite value. Got: {actual}"),
            _ => None,
        }
    }
}

impl From<&'static str> for ValidationCode {
    fn from(s: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(s))
    }
}

impl From<String> for ValidationCode {
    fn from(s: String) -> Self {
        Self(std::borrow::Cow::Owned(s))
    }
}

impl fmt::Display for ValidationCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<&str> for ValidationCode {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<str> for ValidationCode {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

/// A single validation issue discovered in a `DiagramDocument`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub code: ValidationCode,
    pub message: String,
    pub subject: Option<String>,
    /// Actionable hint for AI agents to fix this issue
    pub fix_hint: Option<String>,
}

impl ValidationIssue {
    /// Create a new error issue with auto-populated `fix_hint`.
    pub fn error(
        code: ValidationCode,
        message: impl Into<String>,
        subject: Option<String>,
    ) -> Self {
        let fix_hint = code.default_fix_hint().map(str::to_string);
        Self {
            severity: ValidationSeverity::Error,
            code,
            message: message.into(),
            subject,
            fix_hint,
        }
    }

    /// Create an issue with an explicit custom fix hint.
    pub fn with_fix_hint(
        severity: ValidationSeverity,
        code: ValidationCode,
        message: impl Into<String>,
        subject: Option<String>,
        fix_hint: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            subject,
            fix_hint: Some(fix_hint.into()),
        }
    }
}
