//! `ProposedChanges` — the container for an AI agent's proposed modifications.
//!
//! This module defines the minimal shape needed by the revision-mismatch gate
//! in [`crate::apply::check_revision_mismatch`].  A sibling bead will flesh out
//! the full `ProposedChange` enum and additional fields.

use crate::document::types::{AuthorId, Revision, Timestamp};

/// A complete proposal submitted by an AI agent.
///
/// Only `base_revision` is consumed by the revision-mismatch gate; the
/// remaining fields are placeholders for downstream beads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposedChanges {
    /// The document revision this proposal was built against.
    /// Must match `DiagramDocument::revision` at apply time.
    pub base_revision: Revision,
    /// Identifier of the proposing AI agent.
    pub proposer: AuthorId,
    /// Wall-clock time when the proposal was generated.
    pub proposed_at: Timestamp,
    /// Human-readable summary for UI display.
    pub summary: String,
}
