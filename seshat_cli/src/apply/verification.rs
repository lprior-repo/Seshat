//! Kani verification harnesses for the `apply` module.

#![allow(unexpected_cfgs)]

use diagram_models::document::types::AuthorId;
use diagram_models::document::types::Revision;
use diagram_models::document::DiagramDocument;

use crate::apply::calc::*;
use crate::apply::types::*;

#[kani::proof]
fn kani_check_revision_match_human_priority_invariant() {
    let doc_rev: u64 = kani::any();
    let prop_rev: u64 = kani::any();
    let doc = DiagramDocument {
        revision: Revision::new(doc_rev),
        ..DiagramDocument::default()
    };
    let proposal = ApplyProposal {
        base_revision: Revision::new(prop_rev),
        proposer: AuthorId::new("test".to_string()),
        proposed_at: diagram_models::document::types::Timestamp::new(1),
        summary: String::new(),
        changes: vec![],
    };

    let result = check_revision_match(&doc, &proposal);

    if doc_rev != prop_rev {
        match result {
            Err(details) => {
                assert_eq!(details.expected_revision, Revision::new(prop_rev));
                assert_eq!(details.current_revision, Revision::new(doc_rev));
            }
            Ok(()) => assert!(false, "I1 VIOLATION: mismatch accepted"),
        }
    } else {
        assert_eq!(result, Ok(()));
    }
}

#[kani::proof]
fn kani_validate_and_build_status_atomic_evaluation() {
    let doc_rev: u64 = kani::any();
    let prop_rev: u64 = kani::any();

    let revision_result = if doc_rev == prop_rev {
        Ok(())
    } else {
        Err(ConflictDetails {
            expected_revision: Revision::new(prop_rev),
            current_revision: Revision::new(doc_rev),
        })
    };

    let outcome = match revision_result {
        Ok(()) => ApplyOutcome::Queued {
            proposal_id: "prop-kani".to_string(),
            change_count: 0,
            base_revision: doc_rev,
        },
        Err(ref details) => ApplyOutcome::Rejected {
            reason: RejectionReason::StaleRevision {
                expected: details.expected_revision,
                current: details.current_revision,
            },
        },
    };
    let status = build_apply_status(outcome);

    match status {
        ApplyStatus::Queued { .. } => assert_eq!(revision_result, Ok(())),
        ApplyStatus::Rejected { .. } => assert!(revision_result.is_err()),
    }
}

#[kani::proof]
fn kani_apply_calc_layer_never_panics() {
    let doc_rev: u64 = kani::any();
    let prop_rev: u64 = kani::any();
    let doc = DiagramDocument {
        revision: Revision::new(doc_rev),
        ..DiagramDocument::default()
    };
    let proposal = ApplyProposal {
        base_revision: Revision::new(prop_rev),
        proposer: AuthorId::new("test".to_string()),
        proposed_at: diagram_models::document::types::Timestamp::new(1),
        summary: String::new(),
        changes: vec![],
    };

    let _ = check_revision_match(&doc, &proposal);
    let _ = validate_proposal_schema(&proposal);
    let _ = build_apply_status(ApplyOutcome::Queued {
        proposal_id: "prop-kani2".to_string(),
        change_count: 0,
        base_revision: prop_rev,
    });
    let _ = build_apply_status(ApplyOutcome::Rejected {
        reason: RejectionReason::StaleRevision {
            expected: Revision::new(prop_rev),
            current: Revision::new(doc_rev),
        },
    });
}

#[kani::proof]
fn kani_check_revision_match_is_idempotent_read() {
    let doc_rev: u64 = kani::any();
    let prop_rev: u64 = kani::any();
    let doc = DiagramDocument {
        revision: Revision::new(doc_rev),
        ..DiagramDocument::default()
    };
    let proposal = ApplyProposal {
        base_revision: Revision::new(prop_rev),
        proposer: AuthorId::new("test".to_string()),
        proposed_at: diagram_models::document::types::Timestamp::new(1),
        summary: String::new(),
        changes: vec![],
    };

    let doc_before = doc.clone();
    let result1 = check_revision_match(&doc, &proposal);
    let result2 = check_revision_match(&doc, &proposal);

    assert_eq!(result1, result2, "I4: referential transparency violated");
    assert_eq!(doc, doc_before, "I4: document mutated on read");
}
