//! Location types for AI document references.
//!
//! Provides the `LocationType` enum and its parsing error type.

/// Error type for `LocationType` parsing failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationTypeParseError {
    /// The input string does not match any known variant.
    UnknownVariant,
}

/// Represents the type of location reference for an AI document.
///
/// Variants:
/// - `Gps`: Geographic coordinates (e.g., "37.7749,-122.4194")
/// - `FilePath`: Local filesystem path (e.g., "/home/user/document.md")
/// - `DocumentPosition`: Position within a document (e.g., "line:col 42:10")
/// - `Url`: Web URL (e.g., <https://example.com/document>)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationType {
    /// Geographic coordinates.
    Gps,
    /// Local filesystem path.
    FilePath,
    /// Position within a document.
    DocumentPosition,
    /// Web URL.
    Url,
}

impl LocationType {
    /// Parses a string slice into a `LocationType`.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(LocationType)` if the string matches a variant
    /// * `Err(LocationTypeParseError::UnknownVariant)` otherwise
    ///
    /// # Errors
    ///
    /// * `LocationTypeParseError::UnknownVariant` - If the string doesn't match any variant
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, LocationTypeParseError> {
        s.parse()
    }
}

impl std::str::FromStr for LocationType {
    type Err = LocationTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GPS" => Ok(Self::Gps),
            "file_path" => Ok(Self::FilePath),
            "document_position" => Ok(Self::DocumentPosition),
            "URL" => Ok(Self::Url),
            _ => Err(LocationTypeParseError::UnknownVariant),
        }
    }
}

impl TryFrom<&str> for LocationType {
    type Error = LocationTypeParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl std::fmt::Display for LocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gps => write!(f, "GPS"),
            Self::FilePath => write!(f, "file_path"),
            Self::DocumentPosition => write!(f, "document_position"),
            Self::Url => write!(f, "URL"),
        }
    }
}

#[cfg(test)]
impl LocationType {
    /// Returns all valid string representations for this variant.
    #[must_use]
    pub fn valid_str(&self) -> &'static str {
        match self {
            Self::Gps => "GPS",
            Self::FilePath => "file_path",
            Self::DocumentPosition => "document_position",
            Self::Url => "URL",
        }
    }
}
