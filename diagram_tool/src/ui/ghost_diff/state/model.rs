use std::collections::HashMap;

use itertools::Itertools;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GhostDiffStateMode {
    Idle,
    Reviewing,
    Applying,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum GhostDiffError {
    #[error("invalid transition from {from:?} using {action}")]
    InvalidStateTransition {
        from: GhostDiffStateMode,
        action: String,
    },
    #[error("invalid proposal index {index}; max valid is {max_valid}")]
    InvalidProposalIndex { index: usize, max_valid: usize },
    #[error("no pending proposal")]
    NoPendingProposal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingProposal {
    pub change_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhostDiffState {
    pub mode: GhostDiffStateMode,
    pub pending: Option<PendingProposal>,
    pub toggled: HashMap<usize, bool>,
}

impl GhostDiffState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: GhostDiffStateMode::Idle,
            pending: None,
            toggled: HashMap::new(),
        }
    }

    #[must_use]
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    #[must_use]
    pub fn change_count(&self) -> usize {
        self.pending
            .as_ref()
            .map_or(0, |proposal| proposal.change_count)
    }

    #[must_use]
    pub fn accepted_indices(&self) -> Vec<usize> {
        let change_count = self.change_count();
        self.toggled
            .iter()
            .filter_map(|(&index, &accepted)| (accepted && index < change_count).then_some(index))
            .sorted_unstable()
            .collect()
    }
}

impl Default for GhostDiffState {
    fn default() -> Self {
        Self::new()
    }
}
