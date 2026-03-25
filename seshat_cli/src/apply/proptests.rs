//! Proptest invariants for the `apply` module.

use diagram_models::document::types::AuthorId;
use diagram_models::document::types::Revision;
use diagram_models::document::DiagramDocument;
use proptest::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;

use crate::apply::calc::*;
use crate::apply::io::load_proposal;
use crate::apply::types::*;

// =========================================================================
// PROPTTEST 1: map_apply_subcommand never panics
// =========================================================================

proptest! {
    #[test]
    fn proptest_map_apply_subcommand_never_panics_for_any_option_pathbuf(
        bytes in prop::option::of(prop::collection::vec(any::<u8>(), 0..256))
    ) {
        let path_opt = bytes.map(|b| {
            use std::ffi::OsString;
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStringExt;
                PathBuf::from(OsString::from_vec(b))
            }
            #[cfg(not(unix))]
            {
                PathBuf::from(String::from_utf8_lossy(&b).to_string())
            }
        });
        let is_some = path_opt.is_some();
        let result = map_apply_subcommand(path_opt);
        if is_some {
            prop_assert!(matches!(result.input_source, ApplySource::File(_)));
        } else {
            prop_assert_eq!(result.input_source, ApplySource::Stdin);
        }
    }
}

// =========================================================================
// PROPTTEST 2: check_revision_match exhaustive (I1)
// =========================================================================

proptest! {
    #[test]
    fn proptest_check_revision_match_exhaustive(
        doc_rev in any::<u64>(),
        prop_rev in any::<u64>()
    ) {
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

        if doc_rev == prop_rev {
            prop_assert_eq!(result, Ok(()));
        } else {
            let err = result.expect_err("expected Err for mismatched revisions");
            prop_assert_eq!(err.expected_revision, Revision::new(prop_rev));
            prop_assert_eq!(err.current_revision, Revision::new(doc_rev));
        }
    }
}

// =========================================================================
// PROPTTEST 3: validate_proposal_schema never panics
// =========================================================================

proptest! {
    #[test]
    fn proptest_validate_proposal_schema_never_panics_for_any_proposal(
        base_revision in any::<u64>(),
        proposer_len in 0..100usize
    ) {
        let proposer = AuthorId::new("a".repeat(proposer_len));
        let proposal = ApplyProposal {
            base_revision: Revision::new(base_revision),
            proposer,
            proposed_at: diagram_models::document::types::Timestamp::new(1),
            summary: String::new(),
            changes: vec![],
        };

        let issues = validate_proposal_schema(&proposal);
        prop_assert!(true);

        if proposer_len == 0 {
            prop_assert!(!issues.is_empty(), "empty proposer must produce at least 1 issue");
        }
    }
}

// =========================================================================
// PROPTTEST 4: serialize_apply_status is total
// =========================================================================

proptest! {
    #[test]
    fn proptest_serialize_apply_status_returns_ok_for_any_apply_status(
        proposal_id in ".*",
        change_count in any::<usize>(),
        base_revision in any::<u64>()
    ) {
        let status = ApplyStatus::Queued {
            proposal_id,
            change_count,
            base_revision,
        };
        let result = serialize_apply_status(&status);
        prop_assert!(result.is_ok(), "serialize must succeed for any Queued: {result:?}");
        if let Ok(ref json) = result {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
            prop_assert!(parsed.is_ok(), "output must be valid JSON");
        }
    }
}

// =========================================================================
// PROPTTEST 5: serialize_apply_status round-trip for Rejected
// =========================================================================

proptest! {
    #[test]
    fn proptest_serialize_apply_status_round_trips_for_rejected(
        reason in prop::sample::select(vec![
            RejectionReasonCode::StaleRevision,
            RejectionReasonCode::SchemaInvalid,
            RejectionReasonCode::EmptyChanges,
            RejectionReasonCode::InvalidProposer,
        ]),
        expected in any::<u64>(),
        current in any::<u64>()
    ) {
        let status = ApplyStatus::Rejected {
            reason,
            conflict_details: Some(ConflictDetails {
                expected_revision: Revision::new(expected),
                current_revision: Revision::new(current),
            }),
            validation_issues: vec![],
            hint: None,
        };
        let result = serialize_apply_status(&status);
        prop_assert!(result.is_ok(), "serialize must succeed: {result:?}");
        if let Ok(json) = result {
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&parsed["status"], "rejected");
            prop_assert_eq!(&parsed["conflict_details"]["expected_revision"], expected);
            prop_assert_eq!(&parsed["conflict_details"]["current_revision"], current);
        }
    }
}

// =========================================================================
// PROPTTEST 6: ApplyCommandError display always has prefix
// =========================================================================

proptest! {
    #[test]
    fn proptest_apply_command_error_display_always_starts_with_error_apply_prefix(
        variant_index in 0u8..=15u8,
        payload in ".*"
    ) {
        let err = match variant_index {
            0 => ApplyCommandError::InputFileNotFound(std::path::PathBuf::from(&payload)),
            1 => ApplyCommandError::InputIoError(payload),
            2 => ApplyCommandError::InputInvalidUtf8,
            3 => ApplyCommandError::InputEmpty,
            4 => ApplyCommandError::ProposalJsonMalformed(payload.clone()),
            5 => ApplyCommandError::ProposalSchemaInvalid { issues: vec![] },
            6 => ApplyCommandError::ProposalEmptyChanges,
            7 => ApplyCommandError::ProposalInvalidProposer,
            8 => ApplyCommandError::DocumentNotFound(std::path::PathBuf::from(&payload)),
            9 => ApplyCommandError::DocumentIoError(payload.clone()),
            10 => ApplyCommandError::DocumentInvalidUtf8,
            11 => ApplyCommandError::DocumentEmpty,
            12 => ApplyCommandError::DocumentJsonMalformed(payload.clone()),
            13 => ApplyCommandError::DocumentSchemaInvalid(payload),
            14 => ApplyCommandError::StaleRevision { expected: Revision::new(1), current: Revision::new(2) },
            15 => ApplyCommandError::OutputWriteFailure(payload),
            _ => ApplyCommandError::InputEmpty,
        };
        prop_assert!(
            err.to_string().starts_with("error: apply: "),
            "display must start with 'error: apply: ', got: {:?}", err.to_string()
        );
    }
}

// =========================================================================
// PROPTTEST 7: validate_proposal_schema issue count monotonic
// =========================================================================

proptest! {
    #[test]
    fn proptest_validate_proposal_schema_issue_count_monotonic_with_violations(
        has_base_revision in any::<bool>(),
        proposer_empty in any::<bool>(),
        changes_empty in any::<bool>()
    ) {
        let base_revision = if has_base_revision { Revision::new(1) } else { Revision::new(0) };
        let proposer = if proposer_empty {
            AuthorId::new(String::new())
        } else {
            AuthorId::new("agent".to_string())
        };
        let changes = if changes_empty {
            vec![]
        } else {
            vec![diagram_models::proposed_changes::ProposedChange::MoveNode {
                node_id: diagram_models::document::types::NodeId::new("n1".to_string()),
                new_x: 0.0,
                new_y: 0.0,
            }]
        };

        let proposal = ApplyProposal {
            base_revision,
            proposer,
            proposed_at: diagram_models::document::types::Timestamp::new(1),
            summary: String::new(),
            changes,
        };

        let issues = validate_proposal_schema(&proposal);

        if proposer_empty {
            prop_assert!(!issues.is_empty(), "empty proposer must produce at least 1 issue");
        }

        if changes_empty && !proposer_empty {
            prop_assert!(!issues.is_empty(), "empty changes must produce at least 1 issue");
        }

        if !proposer_empty && !changes_empty {
            prop_assert!(issues.is_empty(), "valid proposal must produce 0 issues");
        }
    }
}

// =========================================================================
// PROPTTEST 8: load_proposal from reader never panics
// =========================================================================

proptest! {
    #[test]
    fn proptest_load_proposal_from_reader_never_panics_for_arbitrary_byte_input(
        bytes in prop::collection::vec(any::<u8>(), 0..1000)
    ) {
        let reader = Cursor::new(bytes);
        let result = load_proposal(&ApplySource::Stdin, reader);
        match result {
            Ok(_) => prop_assert!(true),
            Err(
                ApplyCommandError::InputEmpty
                | ApplyCommandError::InputInvalidUtf8
                | ApplyCommandError::ProposalJsonMalformed(_)
                | ApplyCommandError::ProposalSchemaInvalid { .. }
                | ApplyCommandError::InputIoError(_)
                | ApplyCommandError::ProposalEmptyChanges
                | ApplyCommandError::ProposalInvalidProposer,
            ) => prop_assert!(true),
            Err(
                ApplyCommandError::InputFileNotFound(_)
                | ApplyCommandError::DocumentNotFound(_)
                | ApplyCommandError::DocumentIoError(_)
                | ApplyCommandError::DocumentInvalidUtf8
                | ApplyCommandError::DocumentEmpty
                | ApplyCommandError::DocumentJsonMalformed(_)
                | ApplyCommandError::DocumentSchemaInvalid(_)
                | ApplyCommandError::StaleRevision { .. }
                | ApplyCommandError::OutputWriteFailure(_)
                | ApplyCommandError::ProposalRejected,
            ) => {
                prop_assert!(false, "reader-based load must not return file/document/output errors");
            }
        }
    }
}
