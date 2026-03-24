#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod fsm;
pub mod state;

pub use fsm::{calculate_transition, ReviewError, ReviewEvent, ReviewState};
pub use state::{GhostDiffState, PendingProposal};
