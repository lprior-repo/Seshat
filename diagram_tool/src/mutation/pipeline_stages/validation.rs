#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use crate::mutation::pipeline::ValidationPolicy;
use diagram_models::document::DiagramDocument;
use diagram_models::schema::validate_schema;
use diagram_models::validation::validate_document;

/// Validate the document according to policy.
///
/// # Errors
/// Returns an error if schema validation fails or if semantic validation fails when enabled.
pub fn validate_stage(
    document: &DiagramDocument,
    policy: ValidationPolicy,
) -> Result<(), MutationError> {
    validate_schema(document).map_err(|err| MutationError::Schema(err.to_string()))?;

    if policy == ValidationPolicy::Skip {
        return Ok(());
    }

    let issues = validate_document(document);
    issues
        .first()
        .map_or(Ok(()), |issue| Err(MutationError::from_issue(issue)))
}
