#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ExecutionState, NodeId};

/// Overall status of a complete workflow execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionOverallStatus {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl ExecutionOverallStatus {
    /// Returns `true` if the execution has reached a terminal state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// The output produced by a single step execution.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum StepOutput {
    /// Step completed successfully with this JSON value as its output.
    Success(serde_json::Value),
    /// Step failed with an error message and optional stack trace.
    Failure {
        error: String,
        stack_trace: Option<String>,
    },
    /// Step has not yet produced output.
    Pending,
}

/// A frozen record of a single step's execution within a workflow run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepRecord {
    /// Human-readable name of the step (from the node's `name` field).
    pub step_name: String,
    /// The node type identifier (e.g. `"run"`, `"http-handler"`).
    pub step_type: String,
    /// Per-step execution status.
    pub status: ExecutionState,
    /// Wall-clock time when this step began executing.
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Wall-clock time when this step finished executing.
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Which attempt number this record belongs to (1-indexed).
    pub attempt: u32,
    /// The JSON value that was passed into this step as its input.
    pub input: Option<serde_json::Value>,
    /// The output produced (success value, failure detail, or pending).
    pub output: StepOutput,
}

impl StepRecord {
    /// Computes the wall-clock duration of this step in milliseconds.
    ///
    /// Returns `None` if either `start_time` or `end_time` is absent,
    /// or if the duration cannot be represented as `i64`.
    #[must_use]
    pub fn duration_ms(&self) -> Option<i64> {
        let start = self.start_time?;
        let end = self.end_time?;
        end.signed_duration_since(start).num_milliseconds().into()
    }
}

/// A complete, frozen snapshot of a single workflow execution run.
///
/// Once `status` is terminal (`is_frozen()` returns `true`) this record
/// must not be mutated — it serves as the immutable history entry for
/// replay and inspection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Unique identifier for this execution run.
    pub id: Uuid,
    /// Name of the workflow that was executed.
    pub workflow_name: String,
    /// Overall status of the execution.
    pub status: ExecutionOverallStatus,
    /// Wall-clock time when execution began.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Wall-clock time when execution finished (absent while still running).
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Ordered list of `(NodeId, StepRecord)` pairs — one entry per node
    /// that was visited during this run. Order is deterministic (topological).
    pub steps: Vec<(NodeId, StepRecord)>,
    /// Number of steps that completed successfully.
    pub steps_completed: u32,
    /// Number of steps that failed.
    pub steps_failed: u32,
}

impl ExecutionRecord {
    /// Computes the total wall-clock duration of this execution in milliseconds.
    ///
    /// Returns `None` if `end_time` has not yet been set.
    #[must_use]
    pub fn duration_ms(&self) -> Option<i64> {
        let end = self.end_time?;
        end.signed_duration_since(self.start_time)
            .num_milliseconds()
            .into()
    }

    /// Returns `true` if this record is frozen, i.e. the execution has reached
    /// a terminal state and no further mutations should occur.
    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.status.is_terminal()
    }

    /// Looks up the [`StepRecord`] for a given [`NodeId`].
    ///
    /// Returns `None` if no step with that ID was recorded.
    #[must_use]
    pub fn step_for_node(&self, id: NodeId) -> Option<&StepRecord> {
        self.steps
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, record)| record)
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

/// Converts a legacy [`super::RunRecord`] into an [`ExecutionRecord`] for
/// display purposes.
///
/// Pure function — no side effects.
#[must_use]
pub fn from_run_record(record: &super::RunRecord) -> ExecutionRecord {
    #[allow(clippy::cast_possible_truncation)]
    let steps_completed = record.results.len() as u32;
    let steps_failed = u32::from(!record.success);
    let status = if record.success {
        ExecutionOverallStatus::Succeeded
    } else {
        ExecutionOverallStatus::Failed
    };

    ExecutionRecord {
        id: record.id,
        workflow_name: String::new(),
        status,
        start_time: record.timestamp,
        end_time: None,
        steps: record
            .results
            .iter()
            .map(|(node_id, output)| {
                use crate::graph::ExecutionState;
                (
                    *node_id,
                    StepRecord {
                        step_name: String::new(),
                        step_type: String::new(),
                        status: if record.success {
                            ExecutionState::Succeeded
                        } else {
                            ExecutionState::Failed
                        },
                        start_time: Some(record.timestamp),
                        end_time: None,
                        attempt: 1,
                        input: None,
                        output: StepOutput::Success(output.clone()),
                    },
                )
            })
            .collect(),
        steps_completed,
        steps_failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(secs: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.timestamp_opt(secs, 0).single().unwrap()
    }

    fn make_step(start: Option<i64>, end: Option<i64>) -> StepRecord {
        StepRecord {
            step_name: "test-step".to_string(),
            step_type: "run".to_string(),
            status: ExecutionState::Succeeded,
            start_time: start.map(utc),
            end_time: end.map(utc),
            attempt: 1,
            input: None,
            output: StepOutput::Pending,
        }
    }

    fn make_record(
        start: i64,
        end: Option<i64>,
        status: ExecutionOverallStatus,
    ) -> ExecutionRecord {
        ExecutionRecord {
            id: Uuid::new_v4(),
            workflow_name: "wf".to_string(),
            status,
            start_time: utc(start),
            end_time: end.map(utc),
            steps: Vec::new(),
            steps_completed: 0,
            steps_failed: 0,
        }
    }

    // ── StepRecord::duration_ms ──────────────────────────────────────────────

    #[test]
    fn given_step_with_both_times_when_duration_ms_then_correct_delta_returned() {
        let step = make_step(Some(1_000), Some(1_002));
        assert_eq!(step.duration_ms(), Some(2_000));
    }

    #[test]
    fn given_step_with_no_start_time_when_duration_ms_then_none_returned() {
        let step = make_step(None, Some(1_002));
        assert_eq!(step.duration_ms(), None);
    }

    #[test]
    fn given_step_with_no_end_time_when_duration_ms_then_none_returned() {
        let step = make_step(Some(1_000), None);
        assert_eq!(step.duration_ms(), None);
    }

    #[test]
    fn given_step_with_no_times_when_duration_ms_then_none_returned() {
        let step = make_step(None, None);
        assert_eq!(step.duration_ms(), None);
    }

    // ── ExecutionRecord::duration_ms ─────────────────────────────────────────

    #[test]
    fn given_record_with_end_time_when_duration_ms_then_correct_delta_returned() {
        let record = make_record(2_000, Some(2_005), ExecutionOverallStatus::Succeeded);
        assert_eq!(record.duration_ms(), Some(5_000));
    }

    #[test]
    fn given_record_with_no_end_time_when_duration_ms_then_none_returned() {
        let record = make_record(2_000, None, ExecutionOverallStatus::Running);
        assert_eq!(record.duration_ms(), None);
    }

    // ── ExecutionRecord::is_frozen ───────────────────────────────────────────

    #[test]
    fn given_running_record_when_is_frozen_then_false() {
        let record = make_record(0, None, ExecutionOverallStatus::Running);
        assert!(!record.is_frozen());
    }

    #[test]
    fn given_succeeded_record_when_is_frozen_then_true() {
        let record = make_record(0, Some(1), ExecutionOverallStatus::Succeeded);
        assert!(record.is_frozen());
    }

    #[test]
    fn given_failed_record_when_is_frozen_then_true() {
        let record = make_record(0, Some(1), ExecutionOverallStatus::Failed);
        assert!(record.is_frozen());
    }

    #[test]
    fn given_cancelled_record_when_is_frozen_then_true() {
        let record = make_record(0, Some(1), ExecutionOverallStatus::Cancelled);
        assert!(record.is_frozen());
    }

    // ── ExecutionRecord::step_for_node ───────────────────────────────────────

    #[test]
    fn given_record_with_step_when_step_for_node_with_matching_id_then_step_returned() {
        let node_id = NodeId::new();
        let step = make_step(Some(0), Some(1));
        let record = ExecutionRecord {
            id: Uuid::new_v4(),
            workflow_name: "wf".to_string(),
            status: ExecutionOverallStatus::Succeeded,
            start_time: utc(0),
            end_time: Some(utc(1)),
            steps: vec![(node_id, step.clone())],
            steps_completed: 1,
            steps_failed: 0,
        };

        let found = record.step_for_node(node_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().step_name, "test-step");
    }

    #[test]
    fn given_record_with_step_when_step_for_node_with_unknown_id_then_none_returned() {
        let node_id = NodeId::new();
        let other_id = NodeId::new();
        let step = make_step(Some(0), Some(1));
        let record = ExecutionRecord {
            id: Uuid::new_v4(),
            workflow_name: "wf".to_string(),
            status: ExecutionOverallStatus::Succeeded,
            start_time: utc(0),
            end_time: Some(utc(1)),
            steps: vec![(node_id, step)],
            steps_completed: 1,
            steps_failed: 0,
        };

        assert!(record.step_for_node(other_id).is_none());
    }

    #[test]
    fn given_empty_record_when_step_for_node_then_none_returned() {
        let record = make_record(0, Some(1), ExecutionOverallStatus::Succeeded);
        assert!(record.step_for_node(NodeId::new()).is_none());
    }

    // ── ExecutionOverallStatus::is_terminal ──────────────────────────────────

    #[test]
    fn given_running_status_when_is_terminal_then_false() {
        assert!(!ExecutionOverallStatus::Running.is_terminal());
    }

    #[test]
    fn given_succeeded_status_when_is_terminal_then_true() {
        assert!(ExecutionOverallStatus::Succeeded.is_terminal());
    }

    #[test]
    fn given_failed_status_when_is_terminal_then_true() {
        assert!(ExecutionOverallStatus::Failed.is_terminal());
    }

    #[test]
    fn given_cancelled_status_when_is_terminal_then_true() {
        assert!(ExecutionOverallStatus::Cancelled.is_terminal());
    }

    // ── Serde round-trips ────────────────────────────────────────────────────

    #[test]
    fn step_output_success_roundtrips_through_json() {
        let output = StepOutput::Success(serde_json::json!({"key": 42}));
        let json = serde_json::to_string(&output).unwrap();
        let back: StepOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(output, back);
    }

    #[test]
    fn step_output_failure_roundtrips_through_json() {
        let output = StepOutput::Failure {
            error: "oops".to_string(),
            stack_trace: Some("at line 1".to_string()),
        };
        let json = serde_json::to_string(&output).unwrap();
        let back: StepOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(output, back);
    }

    #[test]
    fn step_output_pending_roundtrips_through_json() {
        let output = StepOutput::Pending;
        let json = serde_json::to_string(&output).unwrap();
        let back: StepOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(output, back);
    }

    #[test]
    fn execution_overall_status_roundtrips_through_json() {
        let statuses = [
            ExecutionOverallStatus::Running,
            ExecutionOverallStatus::Succeeded,
            ExecutionOverallStatus::Failed,
            ExecutionOverallStatus::Cancelled,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let back: ExecutionOverallStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    // ── from_run_record ──────────────────────────────────────────────────────

    #[test]
    fn given_successful_run_record_when_converting_then_execution_record_reflects_success() {
        use super::{from_run_record, NodeId};
        use crate::graph::RunRecord;
        use std::collections::HashMap;

        let node_id = NodeId::new();
        let mut results = HashMap::new();
        results.insert(node_id, serde_json::json!({"ok": true}));

        let run = RunRecord {
            id: Uuid::new_v4(),
            timestamp: utc(1_000),
            results,
            success: true,
        };

        let record = from_run_record(&run);

        assert_eq!(record.id, run.id);
        assert_eq!(record.status, ExecutionOverallStatus::Succeeded);
        assert_eq!(record.steps_completed, 1);
        assert_eq!(record.steps_failed, 0);
        assert_eq!(record.start_time, utc(1_000));
        assert!(record.end_time.is_none());
        assert_eq!(record.steps.len(), 1);
        assert_eq!(record.steps[0].0, node_id);
    }

    #[test]
    fn given_failed_run_record_when_converting_then_execution_record_reflects_failure() {
        use super::{from_run_record, NodeId};
        use crate::graph::RunRecord;
        use std::collections::HashMap;

        let node_id = NodeId::new();
        let mut results = HashMap::new();
        results.insert(node_id, serde_json::json!(null));

        let run = RunRecord {
            id: Uuid::new_v4(),
            timestamp: utc(2_000),
            results,
            success: false,
        };

        let record = from_run_record(&run);

        assert_eq!(record.status, ExecutionOverallStatus::Failed);
        assert_eq!(record.steps_completed, 1);
        assert_eq!(record.steps_failed, 1);
    }

    #[test]
    fn given_empty_run_record_when_converting_then_steps_are_empty() {
        use super::from_run_record;
        use crate::graph::RunRecord;
        use std::collections::HashMap;

        let run = RunRecord {
            id: Uuid::new_v4(),
            timestamp: utc(0),
            results: HashMap::new(),
            success: true,
        };

        let record = from_run_record(&run);
        assert!(record.steps.is_empty());
        assert_eq!(record.steps_completed, 0);
        assert_eq!(record.steps_failed, 0);
    }
}
