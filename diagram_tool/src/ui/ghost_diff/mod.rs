#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

pub mod fsm;
pub mod state;

pub use fsm::{calculate_transition, ReviewError, ReviewEvent, ReviewState};
pub use state::{GhostDiffState, PendingProposal};
