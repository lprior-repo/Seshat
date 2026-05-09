#![allow(clippy::expect_used)]

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
    let result = calculate_transition(&PipelinePhase::ResolvingConflicts, PipelineEvent::Applied);
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
    let result = calculate_transition(&PipelinePhase::Validating, PipelineEvent::Failed(failure));
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
