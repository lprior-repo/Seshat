//! AI Document newtype with validation.
//!
//! Provides the `AiDocument` struct that wraps `ai_document` fields
//! with validation.

use crate::schema_ai_documents::error::AiDocumentError;
use crate::schema_ai_documents::location::LocationType;

/// Newtype wrapper for JSON payload data.
/// Ensures the wrapped string is valid JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPayload(String);

impl JsonPayload {
    /// Creates a new `JsonPayload` from a string.
    ///
    /// The `json_payload` field is stored as TEXT and accepts any string,
    /// including empty strings. JSON validation is not enforced on this field
    /// to allow the storage of any payload content.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw string to wrap
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` always - any string is valid
    #[must_use]
    pub fn new(data: String) -> Self {
        Self(data)
    }

    /// Returns the inner JSON string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapper for location-specific data.
/// Validates that the data matches the expected format for its location type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationData(String);

impl LocationData {
    /// Creates a new `LocationData` after validating format based on location type.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw location data string
    /// * `location_type` - The type of location
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if validation passes
    /// * `Err(AiDocumentError::EmptyLocationData)` if data is empty
    /// * `Err(AiDocumentError::InvalidLocationDataFormat)` if format doesn't match
    ///
    /// # Errors
    ///
    /// * `AiDocumentError::EmptyLocationData` - If `data` is empty
    /// * `AiDocumentError::InvalidLocationDataFormat` - If format doesn't match `location_type`
    pub fn new(data: String, location_type: LocationType) -> Result<Self, AiDocumentError> {
        if data.is_empty() {
            return Err(AiDocumentError::EmptyLocationData);
        }

        let is_valid = match location_type {
            LocationType::Gps => Self::validate_gps_format(&data),
            LocationType::FilePath => Self::validate_file_path_format(&data),
            LocationType::DocumentPosition => Self::validate_document_position_format(&data),
            LocationType::Url => Self::validate_url_format(&data),
        };

        if is_valid {
            Ok(Self(data))
        } else {
            Err(AiDocumentError::InvalidLocationDataFormat)
        }
    }

    /// Validates GPS coordinate format (lat,lng with optional sign and decimals).
    /// Accepts strings like "37.7749,-122.4194" or "-33.8688,151.2093".
    fn validate_gps_format(data: &str) -> bool {
        let data = data.trim();
        let parts: Vec<&str> = data.split(',').collect();

        if parts.len() != 2 {
            return false;
        }

        let lat_result = parts[0].parse::<f64>();
        let lon_result = parts[1].parse::<f64>();

        match (lat_result, lon_result) {
            (Ok(lat), Ok(lon)) => (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon),
            _ => false,
        }
    }

    /// Validates file path format.
    /// Accepts valid filesystem paths - non-empty, no null bytes, reasonable length.
    fn validate_file_path_format(data: &str) -> bool {
        let data = data.trim();

        // Null bytes are not valid in filesystem paths
        if data.contains('\0') {
            return false;
        }

        // Reasonable length limit for filesystem paths (4096 is common MAX_PATH on Linux)
        if data.len() > 4096 {
            return false;
        }

        // Must be non-empty
        !data.is_empty()
    }

    /// Validates document position format (line:col pattern).
    /// Accepts "line:col 42:10" or "42:10" format.
    fn validate_document_position_format(data: &str) -> bool {
        let data = data.trim();
        if let Some(stripped) = data.strip_prefix("line:col ") {
            Self::validate_line_col(stripped)
        } else {
            Self::validate_line_col(data)
        }
    }

    /// Validates line:col format.
    fn validate_line_col(s: &str) -> bool {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return false;
        }
        parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
    }

    /// Validates URL format.
    /// Checks for http:// or https:// prefix and a host portion.
    fn validate_url_format(data: &str) -> bool {
        let lower = data.to_lowercase();
        if !lower.starts_with("http://") && !lower.starts_with("https://") {
            return false;
        }
        let after_scheme_idx = lower.find("://").map_or(0, |i| i + 3);
        let after_scheme = &data[after_scheme_idx..];
        !after_scheme.is_empty() && after_scheme.contains('.')
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated AI document record.
///
/// Represents a row from the `ai_documents` table with validated fields.
/// The `id` and `key` fields are guaranteed to be non-empty after trimming.
/// The `location_data` field is validated to match the `location_type` format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiDocument {
    id: String,
    key: String,
    json_payload: JsonPayload,
    location_type: LocationType,
    location_data: LocationData,
    created_at: i64,
}

impl AiDocument {
    /// Creates a new `AiDocument` after validating required fields.
    ///
    /// # Arguments
    ///
    /// * `id` - Document identifier (must be non-empty after trimming)
    /// * `key` - Document key (must be non-empty after trimming)
    /// * `json_payload` - JSON data (can be any string including empty)
    /// * `location_type` - Type of location reference
    /// * `location_data` - Location reference data
    /// * `created_at` - Unix timestamp of creation
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if validation passes
    /// * `Err(AiDocumentError::EmptyId)` if id is empty or whitespace
    /// * `Err(AiDocumentError::EmptyKey)` if key is empty or whitespace
    /// * `Err(AiDocumentError::EmptyLocationData)` if `location_data` is empty
    /// * `Err(AiDocumentError::InvalidLocationDataFormat)` if `location_data` format doesn't match `location_type`
    ///
    /// # Errors
    ///
    /// * `AiDocumentError::EmptyId` - If `id` is empty or whitespace
    /// * `AiDocumentError::EmptyKey` - If `key` is empty or whitespace
    /// * `AiDocumentError::EmptyLocationData` - If `location_data` is empty
    /// * `AiDocumentError::InvalidLocationDataFormat` - If `location_data` format doesn't match `location_type`
    #[allow(clippy::similar_names)]
    pub fn new(
        id: String,
        key: String,
        json_payload: String,
        location_type: LocationType,
        location_data: String,
        created_at: i64,
    ) -> Result<Self, AiDocumentError> {
        if id.trim().is_empty() {
            return Err(AiDocumentError::EmptyId);
        }
        if key.trim().is_empty() {
            return Err(AiDocumentError::EmptyKey);
        }
        let json_payload = JsonPayload::new(json_payload);
        let location_data = LocationData::new(location_data, location_type)?;
        Ok(Self {
            id,
            key,
            json_payload,
            location_type,
            location_data,
            created_at,
        })
    }

    /// Returns the document identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the document key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the JSON payload.
    #[must_use]
    pub fn json_payload(&self) -> &str {
        self.json_payload.as_str()
    }

    /// Returns the location type.
    #[must_use]
    pub fn location_type(&self) -> &LocationType {
        &self.location_type
    }

    /// Returns the location data.
    #[must_use]
    pub fn location_data(&self) -> &str {
        self.location_data.as_str()
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> &i64 {
        &self.created_at
    }
}
