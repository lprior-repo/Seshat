//! Pipeline Orchestrator State Machine
//!
//! Models the explicit state transitions for the mutation pipeline.
//! Each phase of the pipeline (conflict resolution, apply, validation, history append)
//! is a named state with typed transitions, making illegal pipeline states unrepresentable.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Pipeline must start from `Idle` phase
//! - P2: Each transition advances exactly one stage
//! - P3: `Failed` can only be reached from active stages
//!
//! ### Postconditions
//! - Q1: Every pipeline run ends in `Completed` or `Failed`
//! - Q2: The phase trace records every visited stage
//! - Q3: Backward-compatible with existing `run_mutation` API
//!
//! ### Invariants
//! - I1: Terminal states (`Completed`, `Failed`) never transition forward
//! - I2: `Idle` is the only valid starting state
//! - I3: No stage is ever skipped

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::mutation::error::MutationError;
use crate::mutation::pipeline::{RevisionPolicy, ValidationPolicy};
use crate::mutation::pipeline_stages::{
    apply::apply_stage, conflict_resolution::resolve_conflicts_stage, history_append::append_stage,
    validation::validate_stage,
};
use diagram_models::document::DiagramDocument;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelinePhase {
    Idle,
    ResolvingConflicts,
    Applying,
    Validating,
    AppendingHistory,
    Completed,
    Failed(PipelineFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineFailure {
    ConflictResolution(MutationError),
    Apply(MutationError),
    Validation(MutationError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineEvent {
    Start,
    ConflictsResolved,
    Applied,
    Validated,
    HistoryAppended,
    Failed(PipelineFailure),
    Reset,
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum PipelineTransitionError {
    #[error("invalid transition from {from:?} with event {event:?}")]
    InvalidTransition {
        from: PipelinePhase,
        event: PipelineEvent,
    },
}

pub fn calculate_transition(
    current: &PipelinePhase,
    event: PipelineEvent,
) -> Result<PipelinePhase, PipelineTransitionError> {
    match (current, &event) {
        (PipelinePhase::Idle, PipelineEvent::Start) => Ok(PipelinePhase::ResolvingConflicts),
        (PipelinePhase::ResolvingConflicts, PipelineEvent::ConflictsResolved) => {
            Ok(PipelinePhase::Applying)
        }
        (PipelinePhase::Applying, PipelineEvent::Applied) => Ok(PipelinePhase::Validating),
        (PipelinePhase::Validating, PipelineEvent::Validated) => {
            Ok(PipelinePhase::AppendingHistory)
        }
        (PipelinePhase::AppendingHistory, PipelineEvent::HistoryAppended) => {
            Ok(PipelinePhase::Completed)
        }
        (
            PipelinePhase::ResolvingConflicts,
            PipelineEvent::Failed(PipelineFailure::ConflictResolution(_)),
        ) => Ok(event.into_failed_phase()),
        (PipelinePhase::Applying, PipelineEvent::Failed(PipelineFailure::Apply(_))) => {
            Ok(event.into_failed_phase())
        }
        (PipelinePhase::Validating, PipelineEvent::Failed(PipelineFailure::Validation(_))) => {
            Ok(event.into_failed_phase())
        }
        (PipelinePhase::Completed, PipelineEvent::Reset) => Ok(PipelinePhase::Idle),
        (PipelinePhase::Failed(_), PipelineEvent::Reset) => Ok(PipelinePhase::Idle),
        _ => Err(PipelineTransitionError::InvalidTransition {
            from: current.clone(),
            event,
        }),
    }
}

impl PipelineEvent {
    fn into_failed_phase(self) -> PipelinePhase {
        match self {
            Self::Failed(f) => PipelinePhase::Failed(f),
            _ => {
                PipelinePhase::Failed(PipelineFailure::Apply(MutationError::Schema(String::new())))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineOutcome {
    pub phase: PipelinePhase,
    pub result: Result<DiagramDocument, MutationError>,
    pub trace: Vec<PipelinePhase>,
}

#[must_use]
pub fn run_pipeline<F>(
    current: &DiagramDocument,
    revision_policy: RevisionPolicy,
    validation_policy: ValidationPolicy,
    transform: F,
) -> PipelineOutcome
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>,
{
    let phase = PipelinePhase::Idle;
    let mut trace = vec![phase.clone()];

    let phase = advance(&phase, PipelineEvent::Start, &mut trace);

    let phase = match resolve_conflicts_stage(current) {
        Ok(()) => advance(&phase, PipelineEvent::ConflictsResolved, &mut trace),
        Err(e) => {
            let f = PipelineFailure::ConflictResolution(e);
            let phase = advance(&phase, PipelineEvent::Failed(f), &mut trace);
            let err = extract_failure(&phase);
            return PipelineOutcome {
                phase,
                result: err,
                trace,
            };
        }
    };

    match apply_stage(current, transform) {
        Ok(next) => {
            let phase = advance(&phase, PipelineEvent::Applied, &mut trace);

            match validate_stage(&next, validation_policy) {
                Ok(()) => {
                    let phase = advance(&phase, PipelineEvent::Validated, &mut trace);
                    let result = append_stage(next, current, revision_policy);
                    let phase = advance(&phase, PipelineEvent::HistoryAppended, &mut trace);
                    PipelineOutcome {
                        phase,
                        result: Ok(result),
                        trace,
                    }
                }
                Err(e) => {
                    let f = PipelineFailure::Validation(e);
                    let phase = advance(&phase, PipelineEvent::Failed(f), &mut trace);
                    let err = extract_failure(&phase);
                    PipelineOutcome {
                        phase,
                        result: err,
                        trace,
                    }
                }
            }
        }
        Err(e) => {
            let f = PipelineFailure::Apply(e);
            let phase = advance(&phase, PipelineEvent::Failed(f), &mut trace);
            let err = extract_failure(&phase);
            PipelineOutcome {
                phase,
                result: err,
                trace,
            }
        }
    }
}

fn advance(
    current: &PipelinePhase,
    event: PipelineEvent,
    trace: &mut Vec<PipelinePhase>,
) -> PipelinePhase {
    let next = calculate_transition(current, event);
    match next {
        Ok(p) => {
            trace.push(p.clone());
            p
        }
        Err(_) => current.clone(),
    }
}

fn extract_failure(phase: &PipelinePhase) -> Result<DiagramDocument, MutationError> {
    match phase {
        PipelinePhase::Failed(
            PipelineFailure::ConflictResolution(e)
            | PipelineFailure::Apply(e)
            | PipelineFailure::Validation(e),
        ) => Err(e.clone()),
        _ => Err(MutationError::Schema("unexpected pipeline state".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_idle_when_start_then_resolving_conflicts() {
        let result = calculate_transition(&PipelinePhase::Idle, PipelineEvent::Start);
        assert_eq!(result, Ok(PipelinePhase::ResolvingConflicts));
    }

    #[test]
    fn given_resolving_when_conflicts_resolved_then_applying() {
        let result = calculate_transition(
            &PipelinePhase::ResolvingConflicts,
            PipelineEvent::ConflictsResolved,
        );
        assert_eq!(result, Ok(PipelinePhase::Applying));
    }

    #[test]
    fn given_applying_when_applied_then_validating() {
        let result = calculate_transition(&PipelinePhase::Applying, PipelineEvent::Applied);
        assert_eq!(result, Ok(PipelinePhase::Validating));
    }

    #[test]
    fn given_validating_when_validated_then_appending() {
        let result = calculate_transition(&PipelinePhase::Validating, PipelineEvent::Validated);
        assert_eq!(result, Ok(PipelinePhase::AppendingHistory));
    }

    #[test]
    fn given_appending_when_history_appended_then_completed() {
        let result = calculate_transition(
            &PipelinePhase::AppendingHistory,
            PipelineEvent::HistoryAppended,
        );
        assert_eq!(result, Ok(PipelinePhase::Completed));
    }

    #[test]
    fn given_completed_when_reset_then_idle() {
        let result = calculate_transition(&PipelinePhase::Completed, PipelineEvent::Reset);
        assert_eq!(result, Ok(PipelinePhase::Idle));
    }

    #[test]
    fn given_failed_when_reset_then_idle() {
        let err = MutationError::Schema("test".into());
        let phase = PipelinePhase::Failed(PipelineFailure::ConflictResolution(err));
        let result = calculate_transition(&phase, PipelineEvent::Reset);
        assert_eq!(result, Ok(PipelinePhase::Idle));
    }

    #[test]
    fn given_resolving_when_conflict_failure_then_failed() {
        let err = MutationError::Schema("conflict".into());
        let failure = PipelineFailure::ConflictResolution(err);
        let result = calculate_transition(
            &PipelinePhase::ResolvingConflicts,
            PipelineEvent::Failed(failure.clone()),
        );
        assert_eq!(result, Ok(PipelinePhase::Failed(failure)));
    }

    #[test]
    fn given_applying_when_apply_failure_then_failed() {
        let err = MutationError::Schema("apply".into());
        let failure = PipelineFailure::Apply(err);
        let result = calculate_transition(
            &PipelinePhase::Applying,
            PipelineEvent::Failed(failure.clone()),
        );
        assert_eq!(result, Ok(PipelinePhase::Failed(failure)));
    }

    #[test]
    fn given_validating_when_validation_failure_then_failed() {
        let err = MutationError::Semantic("valid".into());
        let failure = PipelineFailure::Validation(err);
        let result = calculate_transition(
            &PipelinePhase::Validating,
            PipelineEvent::Failed(failure.clone()),
        );
        assert_eq!(result, Ok(PipelinePhase::Failed(failure)));
    }

    #[test]
    fn given_idle_when_non_start_then_invalid() {
        let result = calculate_transition(&PipelinePhase::Idle, PipelineEvent::ConflictsResolved);
        assert!(result.is_err());
    }

    #[test]
    fn given_completed_when_start_then_invalid() {
        let result = calculate_transition(&PipelinePhase::Completed, PipelineEvent::Start);
        assert!(result.is_err());
    }

    #[test]
    fn given_resolving_when_applied_then_invalid() {
        let result =
            calculate_transition(&PipelinePhase::ResolvingConflicts, PipelineEvent::Applied);
        assert!(result.is_err());
    }

    #[test]
    fn given_resolving_when_wrong_failure_type_then_invalid() {
        let err = MutationError::Schema("wrong".into());
        let failure = PipelineFailure::Apply(err);
        let result = calculate_transition(
            &PipelinePhase::ResolvingConflicts,
            PipelineEvent::Failed(failure),
        );
        assert!(result.is_err());
    }

    #[test]
    fn given_validating_when_wrong_failure_type_then_invalid() {
        let err = MutationError::Schema("wrong".into());
        let failure = PipelineFailure::ConflictResolution(err);
        let result =
            calculate_transition(&PipelinePhase::Validating, PipelineEvent::Failed(failure));
        assert!(result.is_err());
    }

    #[test]
    fn given_full_success_trace_then_all_phases_present() {
        let current = DiagramDocument::default();
        let outcome = run_pipeline(
            &current,
            RevisionPolicy::Increment,
            ValidationPolicy::default(),
            |doc| Ok(doc.clone()),
        );
        assert!(outcome.result.is_ok());
        assert_eq!(outcome.phase, PipelinePhase::Completed);
        assert_eq!(
            outcome.trace,
            vec![
                PipelinePhase::Idle,
                PipelinePhase::ResolvingConflicts,
                PipelinePhase::Applying,
                PipelinePhase::Validating,
                PipelinePhase::AppendingHistory,
                PipelinePhase::Completed,
            ]
        );
    }

    #[test]
    fn given_identity_transform_then_revision_increments() {
        let current = DiagramDocument::default();
        let outcome = run_pipeline(
            &current,
            RevisionPolicy::Increment,
            ValidationPolicy::default(),
            |doc| Ok(doc.clone()),
        );
        let doc = outcome.result.expect("should succeed");
        assert_eq!(doc.revision, current.revision.increment());
    }

    #[test]
    fn given_preserve_policy_then_revision_unchanged() {
        let current = DiagramDocument::default();
        let outcome = run_pipeline(
            &current,
            RevisionPolicy::Preserve,
            ValidationPolicy::default(),
            |doc| Ok(doc.clone()),
        );
        let doc = outcome.result.expect("should succeed");
        assert_eq!(doc.revision, current.revision);
    }

    #[test]
    fn given_failing_transform_then_apply_failure_trace() {
        let current = DiagramDocument::default();
        let outcome = run_pipeline(
            &current,
            RevisionPolicy::Increment,
            ValidationPolicy::default(),
            |_| Err(MutationError::Schema("boom".into())),
        );
        assert!(outcome.result.is_err());
        assert!(matches!(
            outcome.phase,
            PipelinePhase::Failed(PipelineFailure::Apply(_))
        ));
        assert_eq!(
            outcome.trace,
            vec![
                PipelinePhase::Idle,
                PipelinePhase::ResolvingConflicts,
                PipelinePhase::Applying,
                PipelinePhase::Failed(PipelineFailure::Apply(MutationError::Schema("boom".into()))),
            ]
        );
    }

    #[test]
    fn given_skip_validation_then_pipeline_succeeds() {
        let current = DiagramDocument::default();
        let outcome = run_pipeline(
            &current,
            RevisionPolicy::Increment,
            ValidationPolicy::Skip,
            |doc| Ok(doc.clone()),
        );
        assert!(outcome.result.is_ok());
        assert_eq!(outcome.phase, PipelinePhase::Completed);
    }
}
