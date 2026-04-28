use std::future::Future;
use std::pin::Pin;

use sqlx::SqlitePool;

use crate::store::types::durable_types::{OperationRecord, OperationState};
use crate::store::types::saga_types::{
    compensation_order, next_pending_step_index, FailedCompensation, SagaDef, SagaError,
    SagaOutcome, SagaResult,
};

use super::saga_journal::{
    create_saga_operation, ensure_saga_tables, read_operation, read_step_journal,
    record_step_complete, record_step_failed, record_step_start, update_operation_state,
};

pub type StepActionFn = Box<
    dyn Fn(
            SqlitePool,
            String,
            u32,
        ) -> Pin<Box<dyn Future<Output = Result<Option<i64>, String>> + Send>>
        + Send
        + Sync,
>;

pub struct SagaExecutor {
    pool: SqlitePool,
    step_actions: Vec<StepActionFn>,
    compensation_actions: Vec<Option<StepActionFn>>,
}

impl SagaExecutor {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            step_actions: Vec::new(),
            compensation_actions: Vec::new(),
        }
    }

    pub fn add_step(
        mut self,
        action: StepActionFn,
        compensation: Option<StepActionFn>,
    ) -> Self {
        self.step_actions.push(action);
        self.compensation_actions.push(compensation);
        self
    }

    pub async fn execute(&self, saga_def: &SagaDef) -> Result<SagaResult, SagaError> {
        ensure_saga_tables(&self.pool)
            .await
            .map_err(|e| SagaError::Store(e.to_string()))?;

        let total_steps = saga_def.total_steps();

        let existing = read_operation(&self.pool, &saga_def.saga_id)
            .await
            .map_err(|e| SagaError::JournalRead(e.to_string()))?;

        if let Some(record) = &existing {
            match record.state {
                OperationState::Completed => {
                    return Ok(SagaResult {
                        operation_id: saga_def.saga_id.clone(),
                        outcome: SagaOutcome::Completed {
                            final_revision: record.final_revision,
                        },
                        completed_steps: record.current_step,
                        total_steps,
                    });
                }
                OperationState::Failed => {
                    return resume_failed_saga(self, saga_def, &record.error_message.clone().unwrap_or_default()).await;
                }
                OperationState::Started | OperationState::InProgress => {
                    return resume_saga(self, saga_def).await;
                }
            }
        }

        let operation = OperationRecord {
            operation_id: saga_def.saga_id.clone(),
            state: OperationState::Started,
            current_step: 0,
            total_steps,
            started_at: epoch_seconds(),
            completed_at: None,
            final_revision: None,
            error_message: None,
            author_id: String::new(),
            description: saga_def.description.clone(),
        };

        create_saga_operation(&self.pool, &operation)
            .await
            .map_err(|e| SagaError::JournalRead(e.to_string()))?;

        update_operation_state(&self.pool, &saga_def.saga_id, OperationState::InProgress, 0, None, None)
            .await
            .map_err(|e| SagaError::JournalRead(e.to_string()))?;

        run_forward_pass(self, saga_def).await
    }
}

async fn run_forward_pass(
    executor: &SagaExecutor,
    saga_def: &SagaDef,
) -> Result<SagaResult, SagaError> {
    let _journal = read_step_journal(&executor.pool, &saga_def.saga_id)
        .await
        .map_err(|e| SagaError::JournalRead(e.to_string()))?;

    let total_steps = saga_def.total_steps();

    loop {
        let current_journal = read_step_journal(&executor.pool, &saga_def.saga_id)
            .await
            .map_err(|e| SagaError::JournalRead(e.to_string()))?;

        let step_idx = match next_pending_step_index(&current_journal, total_steps) {
            Some(idx) => idx,
            None => break,
        };

        let step_def = saga_def
            .step_def(step_idx)
            .ok_or(SagaError::InvalidStepIndex {
                index: step_idx,
                total: total_steps,
            })?;

        record_step_start(
            &executor.pool,
            &saga_def.saga_id,
            step_idx,
            &step_def.step_name,
        )
        .await
        .map_err(|e| SagaError::JournalRead(e.to_string()))?;

        let step_idx_usize = usize::try_from(step_idx).unwrap_or(usize::MAX);
        let action = executor
            .step_actions
            .get(step_idx_usize)
            .ok_or(SagaError::InvalidStepIndex {
                index: step_idx,
                total: total_steps,
            })?;

        match action(executor.pool.clone(), saga_def.saga_id.clone(), step_idx).await {
            Ok(revision) => {
                record_step_complete(&executor.pool, &saga_def.saga_id, step_idx, revision)
                    .await
                    .map_err(|e| SagaError::JournalRead(e.to_string()))?;
            }
            Err(error) => {
                record_step_failed(&executor.pool, &saga_def.saga_id, step_idx, &error)
                    .await
                    .map_err(|e| SagaError::JournalRead(e.to_string()))?;

                update_operation_state(
                    &executor.pool,
                    &saga_def.saga_id,
                    OperationState::InProgress,
                    step_idx,
                    Some(&error),
                    None,
                )
                .await
                .map_err(|e| SagaError::JournalRead(e.to_string()))?;

                return run_compensation(executor, saga_def, step_idx, &error).await;
            }
        }

        update_operation_state(
            &executor.pool,
            &saga_def.saga_id,
            OperationState::InProgress,
            step_idx,
            None,
            None,
        )
        .await
        .map_err(|e| SagaError::JournalRead(e.to_string()))?;
    }

    let journal = read_step_journal(&executor.pool, &saga_def.saga_id)
        .await
        .map_err(|e| SagaError::JournalRead(e.to_string()))?;

    let final_revision = journal
        .iter()
        .filter_map(|r| r.event_revision)
        .max();

    update_operation_state(
        &executor.pool,
        &saga_def.saga_id,
        OperationState::Completed,
        total_steps,
        None,
        final_revision,
    )
    .await
    .map_err(|e| SagaError::JournalRead(e.to_string()))?;

    Ok(SagaResult {
        operation_id: saga_def.saga_id.clone(),
        outcome: SagaOutcome::Completed { final_revision },
        completed_steps: total_steps,
        total_steps,
    })
}

async fn run_compensation(
    executor: &SagaExecutor,
    saga_def: &SagaDef,
    failed_step: u32,
    error: &str,
) -> Result<SagaResult, SagaError> {
    let journal = read_step_journal(&executor.pool, &saga_def.saga_id)
        .await
        .map_err(|e| SagaError::JournalRead(e.to_string()))?;

    let to_compensate = compensation_order(&journal);

    let mut compensated: Vec<u32> = Vec::new();
    let mut failed_compensation: Option<FailedCompensation> = None;

    for step_idx in &to_compensate {
        let idx_usize = usize::try_from(*step_idx).unwrap_or(usize::MAX);
        let compensation = executor
            .compensation_actions
            .get(idx_usize)
            .and_then(|opt| opt.as_ref());

        match compensation {
            Some(comp_fn) => {
                match comp_fn(executor.pool.clone(), saga_def.saga_id.clone(), *step_idx).await {
                    Ok(_) => {
                        compensated.push(*step_idx);
                    }
                    Err(comp_error) => {
                        failed_compensation = Some(FailedCompensation {
                            step_index: *step_idx,
                            error: comp_error,
                        });
                        break;
                    }
                }
            }
            None => {
                compensated.push(*step_idx);
            }
        }
    }

    let outcome = match failed_compensation {
        Some(ref fc) => SagaOutcome::PartiallyCompensated {
            failed_step,
            error: error.to_string(),
            compensation_failed_step: fc.step_index,
            compensation_error: fc.error.clone(),
        },
        None => SagaOutcome::Compensated {
            failed_step,
            error: error.to_string(),
        },
    };

    update_operation_state(
        &executor.pool,
        &saga_def.saga_id,
        OperationState::Failed,
        failed_step,
        Some(error),
        None,
    )
    .await
    .map_err(|e| SagaError::JournalRead(e.to_string()))?;

    Ok(SagaResult {
        operation_id: saga_def.saga_id.clone(),
        outcome,
        completed_steps: u32::try_from(compensated.len()).unwrap_or(u32::MAX),
        total_steps: saga_def.total_steps(),
    })
}

async fn resume_saga(
    executor: &SagaExecutor,
    saga_def: &SagaDef,
) -> Result<SagaResult, SagaError> {
    run_forward_pass(executor, saga_def).await
}

async fn resume_failed_saga(
    executor: &SagaExecutor,
    saga_def: &SagaDef,
    _previous_error: &str,
) -> Result<SagaResult, SagaError> {
    let journal = read_step_journal(&executor.pool, &saga_def.saga_id)
        .await
        .map_err(|e| SagaError::JournalRead(e.to_string()))?;

    let failed_step = journal
        .iter()
        .find(|r| r.status == crate::store::types::durable_types::StepStatus::Failed);

    match failed_step {
        Some(step) => {
            let error = step.error_message.clone().unwrap_or_default();
            run_compensation(executor, saga_def, step.step_index, &error).await
        }
        None => run_forward_pass(executor, saga_def).await,
    }
}

fn epoch_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_epoch_seconds_when_called_then_returns_positive_value() {
        let ts = epoch_seconds();
        assert!(ts > 1_700_000_000, "timestamp should be post-2023");
    }
}
