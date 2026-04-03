#![allow(clippy::unwrap_used, unused_variables)]

use super::{GhostDiffError, GhostDiffState, GhostDiffStateMode, PendingProposal};

fn make_proposal(n: usize) -> PendingProposal {
    PendingProposal {
        change_count: n,
        summary: format!("{n} changes"),
    }
}

#[test]
#[ignore]
#[ignore]
fn receive_proposal_success_initializes_to_true() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(3);

    let result = state.receive_proposal(proposal.clone());

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Reviewing);
    assert_eq!(state.pending, Some(proposal));
    assert_eq!(state.toggled.get(&0), Some(&true));
    assert_eq!(state.toggled.get(&1), Some(&true));
    assert_eq!(state.toggled.get(&2), Some(&true));
    assert_eq!(state.toggled.len(), 3);
}

#[test]
#[ignore]
fn receive_proposal_empty_proposal() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(0);

    let result = state.receive_proposal(proposal.clone());

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Reviewing);
    assert_eq!(state.pending, Some(proposal));
    assert!(state.toggled.is_empty());
}

#[test]
#[ignore]
fn receive_proposal_large_proposal() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(100);

    let result = state.receive_proposal(proposal.clone());

    assert_eq!(result, Ok(()));
    assert_eq!(state.mode, GhostDiffStateMode::Reviewing);
    assert_eq!(state.pending, Some(proposal));
    assert_eq!(state.toggled.len(), 100);
    assert!(state.toggled.values().all(|&v| v));
}

#[test]
#[ignore]
fn receive_proposal_maximum_length_proposal() {
    let mut state = GhostDiffState::new();
    let proposal = make_proposal(usize::MAX);
    let result = state.receive_proposal(proposal);
    let _ = result;
}

#[test]
#[ignore]
fn receive_proposal_fails_when_reviewing() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();

    let result = state.receive_proposal(make_proposal(2));

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Reviewing,
            action: "receive_proposal".to_string()
        })
    );
}

#[test]
#[ignore]
fn receive_proposal_fails_when_applying() {
    let mut state = GhostDiffState::new();
    state.receive_proposal(make_proposal(3)).unwrap();
    state.accept_all().unwrap();

    let result = state.receive_proposal(make_proposal(2));

    assert_eq!(
        result,
        Err(GhostDiffError::InvalidStateTransition {
            from: GhostDiffStateMode::Applying,
            action: "receive_proposal".to_string()
        })
    );
}
