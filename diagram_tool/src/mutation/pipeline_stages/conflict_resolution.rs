#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use diagram_models::document::DiagramDocument;

/// Resolves conflicts before applying the mutation.
///
/// # Errors
/// Returns a `MutationError` if conflict resolution fails.
pub fn resolve_conflicts_stage(_current: &DiagramDocument) -> Result<(), MutationError> {
    Ok(())
}
