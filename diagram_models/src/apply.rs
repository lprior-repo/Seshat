//! Revision-mismatch detection gate for `apply_proposal()`.
//!
//! This module provides [`check_revision_mismatch`], a **pure function** (Calc
//! layer) that compares a proposal's `base_revision` against the current
//! document revision.  If they differ it returns [`ApplyResult::Stale`] with
//! enough metadata for the caller to recover; otherwise it returns
//! [`ApplyResult::Applied`] as a pass-through for downstream processing.
//!
//! # Invariants
//!
//! * **I1**: `Stale` ⟺ `proposal.base_revision != doc.revision`.
//! * **I2**: `StaleInfo` fields faithfully mirror the compared values.
//! * **I3**: The function never panics.
//! * **I4**: Zero side-effects — purely referentially transparent.

use crate::document::types::{EdgeId, NodeId, Revision};
use crate::document::{DiagramDocument, DocumentError, Edge};
use crate::proposed_changes::{ApplyError, DeleteNodeResult, ProposedChange, ProposedChanges};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Metadata carried when a proposal is rejected for revision mismatch.
///
/// Provides the caller with enough information to recover or regenerate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaleInfo {
    /// The revision the proposal was built against.
    pub expected: Revision,
    /// The actual current revision of the document.
    pub current: Revision,
}

/// Result of attempting to apply an AI proposal.
///
/// This bead only implements the `Stale` gate; `Applied` is the pass-through
/// and `PartialConflict` is a placeholder for a downstream bead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyResult {
    /// All proposed changes were applied successfully.
    Applied,
    /// Proposal was based on an outdated document revision.
    Stale(StaleInfo),
    /// Some changes could not be applied due to conflicts (placeholder).
    PartialConflict {
        applied_count: usize,
        skipped_count: usize,
        reasons: Vec<String>,
    },
}

// ---------------------------------------------------------------------------
// Pure calculation
// ---------------------------------------------------------------------------

/// Pure function: check if proposal revision matches document revision.
///
/// Returns [`ApplyResult::Stale`] if revisions differ.
/// Returns [`ApplyResult::Applied`] if revisions match (pass-through for
/// downstream beads).
///
/// # Invariants
///
/// * No mutation of `doc` or `proposal`.
/// * Never panics.
/// * O(1) time complexity.
#[must_use]
pub fn check_revision_mismatch(doc: &DiagramDocument, proposal: &ProposedChanges) -> ApplyResult {
    if proposal.base_revision == doc.revision {
        ApplyResult::Applied
    } else {
        ApplyResult::Stale(StaleInfo {
            expected: proposal.base_revision,
            current: doc.revision,
        })
    }
}

// ---------------------------------------------------------------------------
// DeleteNode apply logic
// ---------------------------------------------------------------------------

/// Apply a `DeleteNode` proposed change to the document.
///
/// Validates snapshot consistency, confirms the node exists, removes the node,
/// cascades deletion to all connected edges, and increments the revision.
///
/// # Errors
///
/// * `ApplyError::SnapshotIdMismatch` if `was_node_id != node_id`
/// * `ApplyError::NodeNotFound` if `node_id` is not in the document
/// * `ApplyError::DocumentError` if the underlying mutation fails
pub fn apply_delete_node(
    doc: &mut DiagramDocument,
    change: &ProposedChange,
) -> Result<DeleteNodeResult, ApplyError> {
    apply_delete_node_inner(doc, change, DiagramDocument::remove_node)
}

/// Test-only seam: inject a failing `remove_fn` to exercise the
/// `ApplyError::DocumentError` wrapping path.
#[cfg(test)]
pub(crate) fn apply_delete_node_with_remove(
    doc: &mut DiagramDocument,
    change: &ProposedChange,
    remove_fn: impl FnOnce(&mut DiagramDocument, &NodeId) -> Result<(), DocumentError>,
) -> Result<DeleteNodeResult, ApplyError> {
    apply_delete_node_inner(doc, change, remove_fn)
}

/// Shared implementation for `apply_delete_node` and its test seam.
///
/// Follows the Mutation Safety Protocol (invariant I4):
/// 1. Extract & validate — verify `was_node_id == node_id` (P1)
/// 2. Existence check — verify `node_id` in `doc.document.nodes` (P2)
/// 3. Collect cascade targets — scan edges referencing `node_id`
/// 4. Mutate — call `remove_fn`, increment revision
fn apply_delete_node_inner(
    doc: &mut DiagramDocument,
    change: &ProposedChange,
    remove_fn: impl FnOnce(&mut DiagramDocument, &NodeId) -> Result<(), DocumentError>,
) -> Result<DeleteNodeResult, ApplyError> {
    let (node_id, was_node_id) = extract_delete_node_ids(change)?;
    validate_delete_node_ids(doc, node_id, was_node_id)?;
    let cascade_deleted_edge_ids = collect_cascade_edge_ids(&doc.document.edges, node_id);
    mutate_delete_node(doc, node_id, remove_fn)?;
    Ok(DeleteNodeResult {
        deleted_node_id: node_id.clone(),
        cascade_deleted_edge_ids,
    })
}

/// Extract `node_id` and `was_node_id` from the change variant.
/// Returns `Err(UnsupportedChangeVariant)` if not a `DeleteNode`.
const fn extract_delete_node_ids(
    change: &ProposedChange,
) -> Result<(&NodeId, &NodeId), ApplyError> {
    let ProposedChange::DeleteNode {
        node_id,
        was_node_id,
        ..
    } = change
    else {
        return Err(ApplyError::UnsupportedChangeVariant);
    };
    Ok((node_id, was_node_id))
}

/// Validate snapshot ID consistency (P1) and node existence (P2).
fn validate_delete_node_ids(
    doc: &DiagramDocument,
    node_id: &NodeId,
    was_node_id: &NodeId,
) -> Result<(), ApplyError> {
    if was_node_id != node_id {
        return Err(ApplyError::SnapshotIdMismatch {
            declared: node_id.clone(),
            snapshot: was_node_id.clone(),
        });
    }
    if !doc.document.nodes.contains_key(node_id) {
        return Err(ApplyError::NodeNotFound(node_id.clone()));
    }
    Ok(())
}

/// Mutate: call `remove_fn` with rollback on failure, then increment revision.
fn mutate_delete_node(
    doc: &mut DiagramDocument,
    node_id: &NodeId,
    remove_fn: impl FnOnce(&mut DiagramDocument, &NodeId) -> Result<(), DocumentError>,
) -> Result<(), ApplyError> {
    let nodes_backup = doc.document.nodes.clone();
    let edges_backup = doc.document.edges.clone();
    remove_fn(doc, node_id).map_err(|e| {
        doc.document.nodes = nodes_backup;
        doc.document.edges = edges_backup;
        ApplyError::DocumentError(e)
    })?;
    doc.revision = Revision::new(doc.revision.value().wrapping_add(1));
    Ok(())
}

/// Collect all edge IDs where `source == node_id` or `target == node_id`.
///
/// Pure read-only scan. Includes self-loops (I5). Returns empty vec if no
/// edges reference the node (I6).
fn collect_cascade_edge_ids(edges: &im::HashMap<EdgeId, Edge>, node_id: &NodeId) -> Vec<EdgeId> {
    edges
        .iter()
        .filter(|(_, edge)| edge.source == *node_id || edge.target == *node_id)
        .map(|(id, _)| id.clone())
        .collect()
}

/// Collect all edge IDs that would be cascade-deleted if the given node were removed.
///
/// Pure read-only query. Does NOT modify the document. Used by the ghost diff
/// rendering layer to show which edges will disappear.
///
/// # Returns
///
/// `Some(Vec<EdgeId>)` for every edge where `source == node_id` or `target == node_id`.
/// Returns `None` if the node does not exist.
#[must_use]
pub fn cascade_edges_for_node(doc: &DiagramDocument, node_id: &NodeId) -> Option<Vec<EdgeId>> {
    doc.document
        .nodes
        .contains_key(node_id)
        .then(|| collect_cascade_edge_ids(&doc.document.edges, node_id))
}

/// Validate and deduplicate accepted indices against the changes slice.
///
/// Returns a vec of unique, in-bounds indices sorted ascending.
/// Out-of-bounds and duplicate indices are silently excluded.
#[must_use]
fn validate_and_dedup_indices(indices: &[usize], max: usize) -> Vec<usize> {
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<usize> = indices
        .iter()
        .filter(|&&idx| idx < max && seen.insert(idx))
        .copied()
        .collect();
    let mut sorted = unique;
    sorted.sort_unstable();
    sorted
}

/// Format an [`ApplyError`] into a human-readable reason string per the
/// contract's error taxonomy.
fn format_error_reason(idx: usize, error: &ApplyError) -> String {
    match error {
        ApplyError::NodeNotFound(id) => {
            format!("change [{idx}]: node not found: {id}")
        }
        ApplyError::SnapshotIdMismatch { declared, snapshot } => {
            format!("change [{idx}]: snapshot mismatch: declared {declared}, snapshot {snapshot}")
        }
        ApplyError::DocumentError(inner) => {
            format!("change [{idx}]: document error: {inner}")
        }
        ApplyError::UnsupportedChangeVariant => {
            format!("change [{idx}]: unsupported change variant")
        }
    }
}

/// Apply a subset of proposed changes to the document.
///
/// Revision mismatch is checked first (delegates to `check_revision_mismatch`).
/// Then iterates over `accepted_indices`, applies each change. Out-of-bounds
/// and duplicate indices are silently ignored. If any change fails, all
/// previously applied changes are rolled back atomically and
/// `PartialConflict` is returned.
///
/// Revision is incremented exactly once if all changes apply successfully.
#[must_use]
pub fn apply_proposal(
    doc: &mut DiagramDocument,
    proposal: &ProposedChanges,
    changes: &[ProposedChange],
    accepted_indices: &[usize],
) -> ApplyResult {
    // Step 1: Revision gate — pure check, no mutation
    let gate = check_revision_mismatch(doc, proposal);
    if let ApplyResult::Stale(_) = gate {
        return gate;
    }

    // Step 2: Validate and dedup indices
    let valid_indices = validate_and_dedup_indices(accepted_indices, changes.len());

    // Step 3: Snapshot document state (im::HashMap structural sharing — O(1))
    let nodes_snapshot = doc.document.nodes.clone();
    let edges_snapshot = doc.document.edges.clone();
    let pre_revision = doc.revision;

    // Step 4: Apply each change; on first error, rollback and return
    let apply_result = valid_indices.iter().try_fold((), |(), &idx| {
        let change = &changes[idx];
        match change {
            ProposedChange::DeleteNode { .. } => {
                apply_delete_node(doc, change).map_err(|e| (idx, e))
            }
            _ => {
                return Err((idx, ApplyError::UnsupportedChangeVariant));
            }
        }?;
        Ok(())
    });

    match apply_result {
        Ok(()) => {
            // Step 5: All succeeded — correct revision to exactly pre+1
            doc.revision = Revision::new(pre_revision.value().wrapping_add(1));
            ApplyResult::Applied
        }
        Err((fail_idx, fail_err)) => {
            // Step 6: Rollback to snapshot
            doc.document.nodes = nodes_snapshot;
            doc.document.edges = edges_snapshot;
            doc.revision = pre_revision;

            // Build reasons: error for failing index + "not attempted" for rest
            let fail_pos = valid_indices
                .iter()
                .position(|&i| i == fail_idx)
                .map_or(0, |p| p);
            let reasons = valid_indices
                .iter()
                .enumerate()
                .map(|(pos, &idx)| match pos.cmp(&fail_pos) {
                    std::cmp::Ordering::Equal => format_error_reason(idx, &fail_err),
                    std::cmp::Ordering::Greater => {
                        format!("change [{idx}]: not attempted due to prior failure")
                    }
                    std::cmp::Ordering::Less => {
                        format!("change [{idx}]: rolled back due to subsequent failure")
                    }
                })
                .collect();

            // All already-rolled-back changes count as skipped
            let skipped = valid_indices.len();
            ApplyResult::PartialConflict {
                applied_count: 0,
                skipped_count: skipped,
                reasons,
            }
        }
    }
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod apply_tests;
