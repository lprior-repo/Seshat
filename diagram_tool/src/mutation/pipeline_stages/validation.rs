#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use crate::mutation::pipeline::ValidationPolicy;
use diagram_models::document::DiagramDocument;
use diagram_models::validation::validate_document;

/// Validate the document according to policy.
///
/// # Errors
/// Returns an error if semantic validation fails when enabled.
pub fn validate_stage(
    document: &DiagramDocument,
    policy: ValidationPolicy,
) -> Result<(), MutationError> {
    if policy == ValidationPolicy::Skip {
        return Ok(());
    }

    let issues = validate_document(document);
    issues
        .first()
        .map_or(Ok(()), |issue| Err(MutationError::from_issue(issue)))
}
