use std::collections::HashMap;

use super::model::{GhostDiffError, GhostDiffState, GhostDiffStateMode, PendingProposal};

fn invalid_transition(mode: &GhostDiffStateMode, action: &str) -> GhostDiffError {
    GhostDiffError::InvalidStateTransition {
        from: mode.clone(),
        action: action.to_string(),
    }
}

fn invalid_index(index: usize, change_count: usize) -> GhostDiffError {
    GhostDiffError::InvalidProposalIndex {
        index,
        max_valid: change_count.checked_sub(1).map_or(0, |value| value),
    }
}

fn initialize_toggles(change_count: usize) -> HashMap<usize, bool> {
    (0..change_count).map(|index| (index, true)).collect()
}

fn prune_toggles(toggled: &HashMap<usize, bool>, change_count: usize) -> HashMap<usize, bool> {
    (0..change_count)
        .map(|index| {
            let accepted = toggled.get(&index).copied().is_none_or(|value| value);
            (index, accepted)
        })
        .collect()
}

fn accepted_change_count(toggled: &HashMap<usize, bool>, change_count: usize) -> usize {
    toggled
        .iter()
        .filter(|(index, accepted)| **index < change_count && **accepted)
        .count()
}

fn validate_reviewing_index(index: usize, change_count: usize) -> Result<(), GhostDiffError> {
    if index < change_count {
        Ok(())
    } else {
        Err(invalid_index(index, change_count))
    }
}

fn compact_proposal(proposal: &PendingProposal, accepted_count: usize) -> PendingProposal {
    if accepted_count == proposal.change_count {
        proposal.clone()
    } else {
        PendingProposal {
            change_count: accepted_count,
            summary: format!("{accepted_count} changes"),
        }
    }
}

impl GhostDiffState {
    pub fn receive_proposal(&mut self, proposal: PendingProposal) -> Result<(), GhostDiffError> {
        match self.mode {
            GhostDiffStateMode::Idle => {
                *self = Self {
                    mode: GhostDiffStateMode::Reviewing,
                    toggled: initialize_toggles(proposal.change_count),
                    pending: Some(proposal),
                };
                Ok(())
            }
            GhostDiffStateMode::Reviewing | GhostDiffStateMode::Applying => {
                Err(invalid_transition(&self.mode, "receive_proposal"))
            }
        }
    }

    pub fn toggle_change(&mut self, index: usize) -> Result<(), GhostDiffError> {
        match (&self.mode, &self.pending) {
            (GhostDiffStateMode::Idle | GhostDiffStateMode::Applying, _) => {
                Err(invalid_transition(&self.mode, "toggle_change"))
            }
            (GhostDiffStateMode::Reviewing, None) => Err(GhostDiffError::NoPendingProposal),
            (GhostDiffStateMode::Reviewing, Some(proposal)) => {
                validate_reviewing_index(index, proposal.change_count)?;
                let next_value = self
                    .toggled
                    .get(&index)
                    .copied()
                    .is_some_and(|value| !value);
                self.toggled.insert(index, next_value);
                debug_assert!(self.toggled.keys().all(|key| *key < proposal.change_count));
                Ok(())
            }
        }
    }

    pub fn accept_all(&mut self) -> Result<(), GhostDiffError> {
        match (&self.mode, &self.pending) {
            (GhostDiffStateMode::Idle | GhostDiffStateMode::Applying, _) => {
                Err(invalid_transition(&self.mode, "accept_all"))
            }
            (GhostDiffStateMode::Reviewing, None) => Err(GhostDiffError::NoPendingProposal),
            (GhostDiffStateMode::Reviewing, Some(proposal)) => {
                self.mode = GhostDiffStateMode::Applying;
                self.toggled = initialize_toggles(proposal.change_count);
                Ok(())
            }
        }
    }

    pub fn reject_all(&mut self) -> Result<(), GhostDiffError> {
        match (&self.mode, &self.pending) {
            (GhostDiffStateMode::Idle | GhostDiffStateMode::Applying, _) => {
                Err(invalid_transition(&self.mode, "reject_all"))
            }
            (GhostDiffStateMode::Reviewing, None) => Err(GhostDiffError::NoPendingProposal),
            (GhostDiffStateMode::Reviewing, Some(_)) => {
                *self = Self::new();
                Ok(())
            }
        }
    }

    pub fn accept_toggled(&mut self) -> Result<(), GhostDiffError> {
        match (&self.mode, &self.pending) {
            (GhostDiffStateMode::Idle | GhostDiffStateMode::Applying, _) => {
                Err(invalid_transition(&self.mode, "accept_toggled"))
            }
            (GhostDiffStateMode::Reviewing, None) => Err(GhostDiffError::NoPendingProposal),
            (GhostDiffStateMode::Reviewing, Some(proposal)) => {
                let normalized_toggles = prune_toggles(&self.toggled, proposal.change_count);
                let accepted_count =
                    accepted_change_count(&normalized_toggles, proposal.change_count);
                let compacted = compact_proposal(proposal, accepted_count);
                *self = Self {
                    mode: GhostDiffStateMode::Applying,
                    toggled: initialize_toggles(compacted.change_count),
                    pending: Some(compacted),
                };
                Ok(())
            }
        }
    }
}
