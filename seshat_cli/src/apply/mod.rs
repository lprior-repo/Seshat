//! Implementation of the `seshat apply` subcommand.
//!
//! # Architecture: Data → Calculations → Actions
//!
//! - **Data** (`types.rs`): `ApplyCommand`, `ApplySource`, `ApplyStatus`, `ConflictDetails`,
//!   `ApplyOutcome`, `RejectionReason`, `RejectionReasonCode`, `ApplyProposal`, `ApplyCommandError`.
//! - **Calculations** (`calc.rs`): `map_apply_subcommand`, `validate_proposal_schema`,
//!   `check_revision_match`, `build_apply_status`, `serialize_apply_status` — pure functions.
//! - **Actions** (`io.rs`): `load_proposal`, `load_current_document`, `execute_apply` — I/O boundary.

mod calc;
mod io;
pub mod types;

#[cfg(test)]
mod proptests;
#[cfg(test)]
mod tests;
#[cfg(kani)]
mod verification;

// Re-export all public types and functions to maintain the existing public API.
pub(crate) use calc::map_apply_subcommand;
pub use calc::{
    build_apply_status, check_revision_match, serialize_apply_status, validate_proposal_schema,
};
pub use io::{execute_apply, load_current_document, load_proposal};
pub use types::{
    ApplyCommand, ApplyCommandError, ApplyOutcome, ApplyProposal, ApplySource, ApplyStatus,
    ConflictDetails, RejectionReason, RejectionReasonCode,
};
