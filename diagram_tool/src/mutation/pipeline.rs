#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use crate::mutation::pipeline_stages::{
    apply::apply_stage, conflict_resolution::resolve_conflicts_stage, history_append::append_stage,
    validation::validate_stage,
};
use diagram_models::document::DiagramDocument;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionPolicy {
    Increment,
    Preserve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ValidationPolicy {
    #[default]
    Validate,
    Skip,
}

/// Run a mutation and check validation.
///
/// # Errors
/// Returns an error if the transformed document is invalid.
pub fn run_mutation<F>(
    current: &DiagramDocument,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    run_mutation_with_policy(
        current,
        RevisionPolicy::Increment,
        ValidationPolicy::default(),
        transform,
    )
}

/// Run a mutation with an explicit policy.
///
/// # Errors
/// Returns an error if the transformation fails or validation fails depending on the policy.
pub fn run_mutation_with_policy<F>(
    current: &DiagramDocument,
    revision_policy: RevisionPolicy,
    validation_policy: ValidationPolicy,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    resolve_conflicts_stage(current)?;

    let next = apply_stage(current, transform)?;

    validate_stage(&next, validation_policy)?;

    Ok(append_stage(next, current, revision_policy))
}

/// Run a mutation without validation.
///
/// # Errors
/// Returns an error if the transformation fails.
pub fn run_mutation_unchecked<F>(
    current: &DiagramDocument,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    run_mutation_with_policy(
        current,
        RevisionPolicy::Increment,
        ValidationPolicy::Skip,
        transform,
    )
}
