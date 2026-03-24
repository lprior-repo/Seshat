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

use crate::document::types::Revision;
use crate::document::DiagramDocument;
use crate::proposed_changes::ProposedChanges;

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

// ===========================================================================
// Tests
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::types::{AuthorId, Revision, Timestamp};
    use proptest::{prop_assert, prop_assert_eq, prop_assume};

    // -- helpers ----------------------------------------------------------

    /// Build a `DiagramDocument` at the given revision.
    fn doc_at(rev: u64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        doc.revision = Revision::new(rev);
        doc
    }

    /// Build a `ProposedChanges` with the given `base_revision`.
    fn proposal_at(rev: u64) -> ProposedChanges {
        ProposedChanges {
            base_revision: Revision::new(rev),
            proposer: AuthorId::new("test-agent".into()),
            proposed_at: Timestamp::new(0),
            summary: String::new(),
        }
    }

    // =====================================================================
    // Happy Path Tests
    // =====================================================================

    #[test]
    fn test_returns_applied_when_revisions_match_initial() {
        let doc = doc_at(0);
        let proposal = proposal_at(0);
        assert_eq!(
            check_revision_mismatch(&doc, &proposal),
            ApplyResult::Applied
        );
    }

    #[test]
    fn test_returns_applied_when_revisions_match_at_high_revision() {
        let doc = doc_at(42);
        let proposal = proposal_at(42);
        assert_eq!(
            check_revision_mismatch(&doc, &proposal),
            ApplyResult::Applied
        );
    }

    #[test]
    fn test_returns_applied_when_both_revisions_are_identical_non_zero() {
        let doc = doc_at(1_000_000);
        let proposal = proposal_at(1_000_000);
        assert_eq!(
            check_revision_mismatch(&doc, &proposal),
            ApplyResult::Applied
        );
    }

    // =====================================================================
    // Error Path Tests
    // =====================================================================

    #[test]
    fn test_returns_stale_when_proposal_revision_is_behind_document() {
        let doc = doc_at(8);
        let proposal = proposal_at(5);
        let result = check_revision_mismatch(&doc, &proposal);
        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(5),
                current: Revision::new(8),
            })
        );
    }

    #[test]
    fn test_returns_stale_when_proposal_revision_is_ahead_of_document() {
        let doc = doc_at(3);
        let proposal = proposal_at(7);
        let result = check_revision_mismatch(&doc, &proposal);
        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(7),
                current: Revision::new(3),
            })
        );
    }

    #[test]
    fn test_stale_info_captures_expected_and_current_correctly() {
        let doc = doc_at(10);
        let proposal = proposal_at(4);
        let result = check_revision_mismatch(&doc, &proposal);
        match result {
            ApplyResult::Stale(info) => {
                assert_eq!(info.expected, Revision::new(4));
                assert_eq!(info.current, Revision::new(10));
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    #[test]
    fn test_stale_info_expected_equals_proposal_base_revision() {
        let proposal = proposal_at(99);
        let doc = doc_at(1);
        match check_revision_mismatch(&doc, &proposal) {
            ApplyResult::Stale(info) => {
                assert_eq!(info.expected, proposal.base_revision);
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    #[test]
    fn test_stale_info_current_equals_document_revision() {
        let proposal = proposal_at(1);
        let doc = doc_at(99);
        match check_revision_mismatch(&doc, &proposal) {
            ApplyResult::Stale(info) => {
                assert_eq!(info.current, doc.revision);
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    // =====================================================================
    // Edge Case Tests
    // =====================================================================

    #[test]
    fn test_stale_at_revision_boundary_zero_vs_one() {
        let doc = doc_at(0);
        let proposal = proposal_at(1);
        let result = check_revision_mismatch(&doc, &proposal);
        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(1),
                current: Revision::INITIAL,
            })
        );
    }

    #[test]
    fn test_stale_at_high_revision_values() {
        let doc = doc_at(u64::MAX - 1);
        let proposal = proposal_at(u64::MAX - 2);
        let result = check_revision_mismatch(&doc, &proposal);
        assert_eq!(
            result,
            ApplyResult::Stale(StaleInfo {
                expected: Revision::new(u64::MAX - 2),
                current: Revision::new(u64::MAX - 1),
            })
        );
    }

    #[test]
    fn test_matching_at_max_revision_boundary() {
        let doc = doc_at(u64::MAX);
        let proposal = proposal_at(u64::MAX);
        assert_eq!(
            check_revision_mismatch(&doc, &proposal),
            ApplyResult::Applied
        );
    }

    #[test]
    fn test_no_panic_on_any_revision_pair() {
        // Exhaustive boundary check — 0, 1, u64::MAX paired with each other.
        let boundaries = [0u64, 1, u64::MAX];
        for a in boundaries {
            for b in boundaries {
                let doc = doc_at(a);
                let proposal = proposal_at(b);
                let _ = check_revision_mismatch(&doc, &proposal);
            }
        }
    }

    // =====================================================================
    // Contract Verification Tests
    // =====================================================================

    #[test]
    fn test_precondition_stale_iff_revisions_differ() {
        // Test both directions of the bidirectional guarantee.
        // Differing → Stale
        assert!(matches!(
            check_revision_mismatch(&doc_at(5), &proposal_at(3)),
            ApplyResult::Stale(_)
        ));
        // Matching → NOT Stale
        assert!(!matches!(
            check_revision_mismatch(&doc_at(7), &proposal_at(7)),
            ApplyResult::Stale(_)
        ));
    }

    #[test]
    fn test_postcondition_document_unchanged_on_stale() {
        let doc = doc_at(3);
        let doc_before = doc.clone();
        let proposal = proposal_at(1);
        let _ = check_revision_mismatch(&doc, &proposal);
        assert_eq!(doc, doc_before);
    }

    #[test]
    fn test_invariant_stale_info_fields_are_faithful() {
        let expected_rev = 12u64;
        let current_rev = 20u64;
        let doc = doc_at(current_rev);
        let proposal = proposal_at(expected_rev);
        match check_revision_mismatch(&doc, &proposal) {
            ApplyResult::Stale(info) => {
                assert_eq!(info.expected, Revision::new(expected_rev));
                assert_eq!(info.current, Revision::new(current_rev));
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    #[test]
    fn test_invariant_function_is_pure_no_side_effects() {
        // Referential transparency: same inputs → same outputs, forever.
        let doc = doc_at(5);
        let proposal = proposal_at(3);
        let first = check_revision_mismatch(&doc, &proposal);
        let second = check_revision_mismatch(&doc, &proposal);
        assert_eq!(first, second);

        // Interleaved call with matching revisions still returns correct.
        let matching_proposal = proposal_at(5);
        let applied = check_revision_mismatch(&doc, &matching_proposal);
        assert_eq!(applied, ApplyResult::Applied);

        // Original mismatch still works.
        let third = check_revision_mismatch(&doc, &proposal);
        assert_eq!(first, third);
    }

    #[test]
    fn test_invariant_function_never_panics() {
        use std::panic::catch_unwind;
        use std::panic::AssertUnwindSafe;

        let result = catch_unwind(AssertUnwindSafe(|| {
            let doc = doc_at(u64::MAX);
            let proposal = proposal_at(0);
            check_revision_mismatch(&doc, &proposal)
        }));
        assert!(result.is_ok());
    }

    // =====================================================================
    // Property-Based Tests (proptest)
    // =====================================================================

    proptest::proptest! {
        #[test]
        fn proptest_revision_mismatch_detection_is_exhaustive(a in proptest::num::u64::ANY, b in proptest::num::u64::ANY) {
            let doc = doc_at(a);
            let proposal = proposal_at(b);
            let result = check_revision_mismatch(&doc, &proposal);
            if a == b {
                prop_assert_eq!(result, ApplyResult::Applied);
            } else {
                prop_assert!(matches!(result, ApplyResult::Stale(_)));
            }
        }

        #[test]
        fn proptest_stale_info_always_matches_inputs(expected in proptest::num::u64::ANY, current in proptest::num::u64::ANY) {
            // Only test the Stale path
            prop_assume!(expected != current);
            let doc = doc_at(current);
            let proposal = proposal_at(expected);
            match check_revision_mismatch(&doc, &proposal) {
                ApplyResult::Stale(info) => {
                    prop_assert_eq!(info.expected, Revision::new(expected));
                    prop_assert_eq!(info.current, Revision::new(current));
                }
                other => panic!("expected Stale for differing revisions, got {other:?}"),
            }
        }
    }
}
