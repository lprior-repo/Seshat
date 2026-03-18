use std::fmt;

/// Error type for invalid diagram IDs
#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid diagram ID: {0}")]
pub struct DiagramIdError(pub String);

/// Newtype for Diagram Identifier to prevent primitive obsession.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagramId(String);

impl Default for DiagramId {
    fn default() -> Self {
        Self("default_diagram".to_string())
    }
}

impl DiagramId {
    /// Create a new DiagramId with validation.
    ///
    /// # Errors
    /// Returns `DiagramIdError` if the ID contains path traversal characters
    /// like `..`, `/`, or `\`, or is empty.
    pub fn new(id: String) -> Result<Self, DiagramIdError> {
        if id.is_empty() {
            return Err(DiagramIdError("ID cannot be empty".to_string()));
        }
        if id.contains("..") || id.contains('/') || id.contains('\\') {
            return Err(DiagramIdError(format!(
                "ID '{id}' contains invalid characters (../  \\)"
            )));
        }
        Ok(Self(id))
    }

    /// Create a new DiagramId without validation (for trusted sources).
    #[must_use]
    pub const fn new_unchecked(id: String) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DiagramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DiagramId {
    type Error = DiagramIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        DiagramId::new(s)
    }
}

impl TryFrom<&str> for DiagramId {
    type Error = DiagramIdError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        DiagramId::new(s.to_string())
    }
}
