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

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
}

impl ValidationIssue {
    /// Create a new error issue.
    pub fn error(
        code: ValidationCode,
        message: impl Into<String>,
        subject: Option<String>,
    ) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            code,
            message: message.into(),
            subject,
        }
    }
}
