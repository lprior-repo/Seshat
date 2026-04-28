#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use crate::mutation::orchestrator::run_pipeline;
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
    let outcome = run_pipeline(current, revision_policy, validation_policy, transform);
    outcome.result
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
