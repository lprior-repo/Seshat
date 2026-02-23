#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::graph::{NodeId, RunRecord};
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Frozen mode: replays a historical `RunRecord` onto the canvas view.
/// Does NOT mutate the live workflow — it's a read-only overlay.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct FrozenModeState {
    active_run_id: Signal<Option<Uuid>>,
    frozen_results: Signal<HashMap<NodeId, serde_json::Value>>,
}

#[allow(dead_code)]
impl FrozenModeState {
    /// Returns a read-only signal for the currently active run ID.
    #[must_use]
    pub fn active_run_id(&self) -> ReadSignal<Option<Uuid>> {
        ReadSignal::from(self.active_run_id)
    }

    /// Returns `true` when a historical run is loaded in frozen mode.
    #[must_use]
    pub fn is_frozen(&self) -> bool {
        self.active_run_id.read().is_some()
    }

    /// Returns a read-only signal for the frozen node-result map.
    #[must_use]
    pub fn frozen_results(&self) -> ReadSignal<HashMap<NodeId, serde_json::Value>> {
        ReadSignal::from(self.frozen_results)
    }

    /// Activate frozen mode by loading a `RunRecord`.
    pub fn activate(&mut self, run: &RunRecord) {
        self.active_run_id.set(Some(run.id));
        self.frozen_results.set(run.results.clone());
    }

    /// Exit frozen mode and clear the overlay.
    pub fn deactivate(&mut self) {
        self.active_run_id.set(None);
        self.frozen_results.set(HashMap::new());
    }

    /// Returns the recorded output for `node_id`, or `None` if not present.
    #[must_use]
    pub fn result_for_node(&self, node_id: NodeId) -> Option<serde_json::Value> {
        self.frozen_results.read().get(&node_id).cloned()
    }
}

/// Initialises the frozen-mode signals inside a Dioxus component.
#[allow(dead_code)]
pub fn use_frozen_mode() -> FrozenModeState {
    FrozenModeState {
        active_run_id: use_signal(|| None),
        frozen_results: use_signal(HashMap::new),
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Given a list of run records, find one by ID.
/// Pure function, no side effects.
#[must_use]
#[allow(dead_code)]
pub fn find_run_by_id(history: &[RunRecord], id: Uuid) -> Option<&RunRecord> {
    history.iter().find(|r| r.id == id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::find_run_by_id;
    use crate::graph::RunRecord;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_run(id: Uuid) -> RunRecord {
        RunRecord {
            id,
            timestamp: chrono::Utc::now(),
            results: HashMap::new(),
            success: true,
        }
    }

    #[test]
    fn given_empty_history_when_finding_run_then_returns_none() {
        let result = find_run_by_id(&[], Uuid::new_v4());
        assert!(result.is_none());
    }

    #[test]
    fn given_history_with_run_when_finding_by_id_then_returns_run() {
        let id = Uuid::new_v4();
        let run = make_run(id);
        let history = vec![run];

        let result = find_run_by_id(&history, id);
        assert!(result.is_some());
        assert_eq!(result.map(|r| r.id), Some(id));
    }

    #[test]
    fn given_history_with_multiple_runs_when_finding_by_wrong_id_then_returns_none() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let wrong_id = Uuid::new_v4();
        let history = vec![make_run(id_a), make_run(id_b)];

        let result = find_run_by_id(&history, wrong_id);
        assert!(result.is_none());
    }
}
