#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use diagram_models::document::DiagramDocument;

/// Apply the transformation to the document.
///
/// # Errors
/// Returns an error if the transformation fails.
pub fn apply_stage<F>(
    current: &DiagramDocument,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    transform(current)
}
