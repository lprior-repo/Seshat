use serde::{Deserialize, Serialize};

use super::durable_types::{OperationState, StepRecord, StepStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaStepDef {
    pub step_name: String,
    pub compensation_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaDef {
    pub saga_id: String,
    pub steps: Vec<SagaStepDef>,
    pub description: String,
}

impl SagaDef {
    #[must_use]
    pub fn total_steps(&self) -> u32 {
        u32::try_from(self.steps.len()).unwrap_or(u32::MAX)
    }

    #[must_use]
    pub fn step_def(&self, index: u32) -> Option<&SagaStepDef> {
        self.steps.get(usize::try_from(index).unwrap_or(usize::MAX))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaResult {
    pub operation_id: String,
    pub outcome: SagaOutcome,
    pub completed_steps: u32,
    pub total_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SagaOutcome {
    Completed {
        final_revision: Option<i64>,
    },
    Compensated {
        failed_step: u32,
        error: String,
    },
    PartiallyCompensated {
        failed_step: u32,
        error: String,
        compensation_failed_step: u32,
        compensation_error: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompensationResult {
    pub compensated_steps: Vec<u32>,
    pub failed_compensation: Option<FailedCompensation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedCompensation {
    pub step_index: u32,
    pub error: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SagaError {
    #[error("Step {step_index} failed: {error}")]
    StepFailed { step_index: u32, error: String },
    #[error("Compensation for step {step_index} failed: {compensation_error}. Original failure at step {failed_step}: {original_error}")]
    CompensationFailed {
        failed_step: u32,
        original_error: String,
        step_index: u32,
        compensation_error: String,
    },
    #[error("Journal read error: {0}")]
    JournalRead(String),
    #[error("Invalid step index {index} for saga with {total} steps")]
    InvalidStepIndex { index: u32, total: u32 },
    #[error("Store error: {0}")]
    Store(String),
    #[error("Saga already completed with outcome: {0:?}")]
    AlreadyCompleted(OperationState),
}

#[must_use]
pub fn next_pending_step_index(journal: &[StepRecord], total_steps: u32) -> Option<u32> {
    for idx in 0..total_steps {
        let step_entry = journal.iter().find(|r| r.step_index == idx);
        match step_entry {
            None => return Some(idx),
            Some(record) => match record.status {
                StepStatus::Pending | StepStatus::Running => return Some(idx),
                StepStatus::Failed => return None,
                StepStatus::Completed | StepStatus::Skipped => continue,
            },
        }
    }
    None
}

#[must_use]
pub fn completed_step_indices(journal: &[StepRecord]) -> Vec<u32> {
    let mut indices: Vec<u32> = journal
        .iter()
        .filter(|r| r.status == StepStatus::Completed)
        .map(|r| r.step_index)
        .collect();
    indices.sort();
    indices
}

#[must_use]
pub fn compensation_order(journal: &[StepRecord]) -> Vec<u32> {
    let mut completed = completed_step_indices(journal);
    completed.reverse();
    completed
}

#[must_use]
pub fn classify_journal_outcome(
    journal: &[StepRecord],
    total_steps: u32,
) -> Option<OperationState> {
    if journal.is_empty() {
        return None;
    }

    let has_failed = journal.iter().any(|r| r.status == StepStatus::Failed);
    let has_running = journal.iter().any(|r| r.status == StepStatus::Running);
    let has_pending = journal.iter().any(|r| r.status == StepStatus::Pending);

    if has_running {
        return Some(OperationState::InProgress);
    }

    if has_failed {
        let all_compensated_or_skipped = journal
            .iter()
            .filter(|r| r.status == StepStatus::Completed)
            .all(|_| {
                journal
                    .iter()
                    .any(|r2| r2.status == StepStatus::Skipped || r2.status == StepStatus::Failed)
            });
        if all_compensated_or_skipped {
            return Some(OperationState::Failed);
        }
        return Some(OperationState::InProgress);
    }

    if has_pending {
        return Some(OperationState::InProgress);
    }

    let completed_count = journal
        .iter()
        .filter(|r| r.status == StepStatus::Completed)
        .count() as u32;

    if completed_count == total_steps {
        return Some(OperationState::Completed);
    }

    Some(OperationState::InProgress)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step_record(index: u32, status: StepStatus) -> StepRecord {
        StepRecord {
            operation_id: "test-op".to_string(),
            step_index: index,
            step_name: format!("step-{index}"),
            status,
            event_revision: None,
            created_at: 0,
            started_at: None,
            completed_at: None,
            error_message: None,
        }
    }

    #[test]
    fn given_empty_journal_when_next_pending_then_returns_first_step() {
        assert_eq!(next_pending_step_index(&[], 3), Some(0));
    }

    #[test]
    fn given_all_completed_when_next_pending_then_returns_none() {
        let journal = vec![
            make_step_record(0, StepStatus::Completed),
            make_step_record(1, StepStatus::Completed),
            make_step_record(2, StepStatus::Completed),
        ];
        assert_eq!(next_pending_step_index(&journal, 3), None);
    }

    #[test]
    fn given_step_1_failed_when_next_pending_then_returns_none() {
        let journal = vec![
            make_step_record(0, StepStatus::Completed),
            make_step_record(1, StepStatus::Failed),
        ];
        assert_eq!(next_pending_step_index(&journal, 3), None);
    }

    #[test]
    fn given_step_0_completed_when_next_pending_then_returns_step_1() {
        let journal = vec![make_step_record(0, StepStatus::Completed)];
        assert_eq!(next_pending_step_index(&journal, 3), Some(1));
    }

    #[test]
    fn given_steps_0_1_completed_when_compensation_order_then_returns_1_0() {
        let journal = vec![
            make_step_record(0, StepStatus::Completed),
            make_step_record(1, StepStatus::Completed),
        ];
        assert_eq!(compensation_order(&journal), vec![1, 0]);
    }

    #[test]
    fn given_mixed_statuses_when_compensation_order_then_returns_only_completed_reversed() {
        let journal = vec![
            make_step_record(0, StepStatus::Completed),
            make_step_record(1, StepStatus::Failed),
            make_step_record(2, StepStatus::Completed),
        ];
        assert_eq!(compensation_order(&journal), vec![2, 0]);
    }

    #[test]
    fn given_all_completed_when_classify_then_returns_completed() {
        let journal = vec![
            make_step_record(0, StepStatus::Completed),
            make_step_record(1, StepStatus::Completed),
        ];
        assert_eq!(
            classify_journal_outcome(&journal, 2),
            Some(OperationState::Completed)
        );
    }

    #[test]
    fn given_empty_journal_when_classify_then_returns_none() {
        assert_eq!(classify_journal_outcome(&[], 3), None);
    }
}
