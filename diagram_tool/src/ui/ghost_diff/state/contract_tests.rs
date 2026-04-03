#![allow(clippy::unwrap_used, unused_variables)]

use proptest::prelude::*;

use super::{GhostDiffState, GhostDiffStateMode, PendingProposal};

fn make_proposal(n: usize) -> PendingProposal {
    PendingProposal {
        change_count: n,
        summary: format!("{n} changes"),
    }
}

proptest! {
    #[test]
    fn state_proposal_linkage_invariant(
        operations in prop::collection::vec(0..5u8, 0..50),
        proposal_sizes in prop::collection::vec(0..10usize, 0..50),
        toggle_indices in prop::collection::vec(0..15usize, 0..50)
    ) {
        let mut state = GhostDiffState::new();
        let mut proposal_index = 0usize;
        let mut toggle_index = 0usize;

        for operation in operations {
            match operation {
                0 => {
                    let size = *proposal_sizes.get(proposal_index).unwrap_or(&3);
                    let _ = state.receive_proposal(make_proposal(size));
                    proposal_index += 1;
                }
                1 => {
                    let index = *toggle_indices.get(toggle_index).unwrap_or(&0);
                    let _ = state.toggle_change(index);
                    toggle_index += 1;
                }
                2 => {
                    let _ = state.accept_all();
                }
                3 => {
                    let _ = state.reject_all();
                }
                4 => {
                    let _ = state.accept_toggled();
                }
                _ => unreachable!(),
            }

            match state.mode {
                GhostDiffStateMode::Idle => {
                    prop_assert!(state.pending.is_none());
                    prop_assert!(state.toggled.is_empty());
                }
                GhostDiffStateMode::Reviewing | GhostDiffStateMode::Applying => {
                    prop_assert!(state.pending.is_some());
                }
            }
        }
    }

    #[test]
    fn index_bounds_invariant(
        operations in prop::collection::vec(0..5u8, 0..50),
        proposal_sizes in prop::collection::vec(0..10usize, 0..50),
        toggle_indices in prop::collection::vec(0..15usize, 0..50)
    ) {
        let mut state = GhostDiffState::new();
        let mut proposal_index = 0usize;
        let mut toggle_index = 0usize;

        for operation in operations {
            match operation {
                0 => {
                    let size = *proposal_sizes.get(proposal_index).unwrap_or(&3);
                    let _ = state.receive_proposal(make_proposal(size));
                    proposal_index += 1;
                }
                1 => {
                    let index = *toggle_indices.get(toggle_index).unwrap_or(&0);
                    let _ = state.toggle_change(index);
                    toggle_index += 1;
                }
                2 => {
                    let _ = state.accept_all();
                }
                3 => {
                    let _ = state.reject_all();
                }
                4 => {
                    let _ = state.accept_toggled();
                }
                _ => unreachable!(),
            }

            if let Some(pending) = &state.pending {
                for key in state.toggled.keys() {
                    prop_assert!(*key < pending.change_count);
                }
            }
        }
    }
}
