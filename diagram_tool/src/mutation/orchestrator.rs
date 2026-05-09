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
#[path = "orchestrator_tests.rs"]
mod tests;
