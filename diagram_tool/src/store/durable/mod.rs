//! Durable Workflow Store - Restate-like durable execution for diagram operations
//!
//! This module provides:
//! - Operation tracking for multi-step AI workflows
//! - Step journal for retry/resume capability
//! - Outbox for reliable side-effect delivery
//! - Conflict diff on conditional append rejection
//! - Cursor-based pagination for incremental sync

pub mod bootstrap;
pub mod conflict;
pub mod cursor;
pub mod error;
pub mod operation;
pub mod outbox;
pub mod step_journal;
pub mod workflow;

pub use bootstrap::*;
pub use conflict::*;
pub use cursor::*;
pub use error::*;
pub use operation::*;
pub use outbox::*;
pub use step_journal::*;
pub use workflow::*;

#[cfg(test)]
mod conflict_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod cursor_tests;
#[cfg(test)]
mod edge_cases_tests;
#[cfg(test)]
mod error_paths_tests;
#[cfg(test)]
mod operation_tests;
#[cfg(test)]
mod outbox_tests;
#[cfg(test)]
mod scenarios_tests;
#[cfg(test)]
mod step_journal_tests;
#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod workflow_tests;
