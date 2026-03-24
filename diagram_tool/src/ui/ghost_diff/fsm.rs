//! Ghost Diff Finite State Machine
//!
//! Models explicit state transitions for AI proposal review workflow.
//! States: IDLE -> REVIEWING -> APPLYING -> IDLE

use super::state::PendingProposal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewState {
    Idle,
    Reviewing { proposal: PendingProposal },
    Applying { proposal: PendingProposal },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewEvent {
    ProposalReceived(PendingProposal),
    Accept,
    Reject,
    Cancel,
    ApplyComplete,
    ApplyFailed(String),
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum ReviewError {
    #[error("Invalid transition from {from:?} with event {event:?}")]
    InvalidTransition {
        from: ReviewState,
        event: ReviewEvent,
    },
    #[error("No proposal to accept")]
    NoProposal,
    #[error("Apply already in progress")]
    ApplyInProgress,
}

pub fn calculate_transition(
    current: &ReviewState,
    event: ReviewEvent,
) -> Result<ReviewState, ReviewError> {
    match (current, &event) {
        (ReviewState::Idle, ReviewEvent::ProposalReceived(proposal)) => {
            Ok(ReviewState::Reviewing {
                proposal: proposal.clone(),
            })
        }
        (ReviewState::Reviewing { proposal }, ReviewEvent::Accept) => Ok(ReviewState::Applying {
            proposal: proposal.clone(),
        }),
        (ReviewState::Reviewing { .. }, ReviewEvent::Reject) => Ok(ReviewState::Idle),
        (ReviewState::Reviewing { .. }, ReviewEvent::Cancel) => Ok(ReviewState::Idle),
        (ReviewState::Applying { .. }, ReviewEvent::ApplyComplete) => Ok(ReviewState::Idle),
        (ReviewState::Applying { .. }, ReviewEvent::ApplyFailed(_)) => Ok(ReviewState::Idle),
        (ReviewState::Applying { .. }, ReviewEvent::Cancel) => Ok(ReviewState::Idle),
        (ReviewState::Idle, ReviewEvent::Cancel) => Ok(ReviewState::Idle),
        _ => Err(ReviewError::InvalidTransition {
            from: current.clone(),
            event,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proposal(n: usize) -> PendingProposal {
        PendingProposal {
            change_count: n,
            summary: format!("{} changes", n),
        }
    }

    #[test]
    fn idle_to_reviewing_on_proposal_received() {
        let proposal = make_proposal(3);
        let result = calculate_transition(
            &ReviewState::Idle,
            ReviewEvent::ProposalReceived(proposal.clone()),
        );
        assert_eq!(result, Ok(ReviewState::Reviewing { proposal }));
    }

    #[test]
    fn reviewing_to_applying_on_accept() {
        let proposal = make_proposal(5);
        let result = calculate_transition(
            &ReviewState::Reviewing {
                proposal: proposal.clone(),
            },
            ReviewEvent::Accept,
        );
        assert_eq!(result, Ok(ReviewState::Applying { proposal }));
    }

    #[test]
    fn reviewing_to_idle_on_reject() {
        let proposal = make_proposal(2);
        let result =
            calculate_transition(&ReviewState::Reviewing { proposal }, ReviewEvent::Reject);
        assert_eq!(result, Ok(ReviewState::Idle));
    }

    #[test]
    fn reviewing_to_idle_on_cancel() {
        let proposal = make_proposal(1);
        let result =
            calculate_transition(&ReviewState::Reviewing { proposal }, ReviewEvent::Cancel);
        assert_eq!(result, Ok(ReviewState::Idle));
    }

    #[test]
    fn applying_to_idle_on_complete() {
        let proposal = make_proposal(4);
        let result = calculate_transition(
            &ReviewState::Applying { proposal },
            ReviewEvent::ApplyComplete,
        );
        assert_eq!(result, Ok(ReviewState::Idle));
    }

    #[test]
    fn applying_to_idle_on_failed() {
        let proposal = make_proposal(4);
        let result = calculate_transition(
            &ReviewState::Applying { proposal },
            ReviewEvent::ApplyFailed("error".to_string()),
        );
        assert_eq!(result, Ok(ReviewState::Idle));
    }

    #[test]
    fn applying_to_idle_on_cancel() {
        let proposal = make_proposal(3);
        let result = calculate_transition(&ReviewState::Applying { proposal }, ReviewEvent::Cancel);
        assert_eq!(result, Ok(ReviewState::Idle));
    }

    #[test]
    fn idle_stays_idle_on_cancel() {
        let result = calculate_transition(&ReviewState::Idle, ReviewEvent::Cancel);
        assert_eq!(result, Ok(ReviewState::Idle));
    }

    #[test]
    fn invalid_transition_idle_accept() {
        let result = calculate_transition(&ReviewState::Idle, ReviewEvent::Accept);
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_idle_reject() {
        let result = calculate_transition(&ReviewState::Idle, ReviewEvent::Reject);
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_idle_apply_complete() {
        let result = calculate_transition(&ReviewState::Idle, ReviewEvent::ApplyComplete);
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_reviewing_proposal_received() {
        let proposal = make_proposal(1);
        let new_proposal = make_proposal(2);
        let result = calculate_transition(
            &ReviewState::Reviewing { proposal },
            ReviewEvent::ProposalReceived(new_proposal),
        );
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_reviewing_apply_complete() {
        let proposal = make_proposal(1);
        let result = calculate_transition(
            &ReviewState::Reviewing { proposal },
            ReviewEvent::ApplyComplete,
        );
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_applying_accept() {
        let proposal = make_proposal(1);
        let result = calculate_transition(&ReviewState::Applying { proposal }, ReviewEvent::Accept);
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn invalid_transition_applying_proposal_received() {
        let proposal = make_proposal(1);
        let new_proposal = make_proposal(2);
        let result = calculate_transition(
            &ReviewState::Applying { proposal },
            ReviewEvent::ProposalReceived(new_proposal),
        );
        assert!(matches!(result, Err(ReviewError::InvalidTransition { .. })));
    }

    #[test]
    fn proposal_preserved_through_reviewing_to_applying() {
        let proposal = make_proposal(7);
        let reviewing = calculate_transition(
            &ReviewState::Idle,
            ReviewEvent::ProposalReceived(proposal.clone()),
        )
        .expect("transition should succeed");
        let applying = calculate_transition(&reviewing, ReviewEvent::Accept)
            .expect("transition should succeed");
        match applying {
            ReviewState::Applying { proposal: p } => {
                assert_eq!(p, proposal);
            }
            _ => panic!("Expected Applying state"),
        }
    }

    #[test]
    fn full_cycle_idle_reviewing_applying_idle() {
        let proposal = make_proposal(3);
        let s1 = calculate_transition(
            &ReviewState::Idle,
            ReviewEvent::ProposalReceived(proposal.clone()),
        )
        .expect("t1");
        let s2 = calculate_transition(&s1, ReviewEvent::Accept).expect("t2");
        let s3 = calculate_transition(&s2, ReviewEvent::ApplyComplete).expect("t3");
        assert_eq!(s3, ReviewState::Idle);
    }
}
