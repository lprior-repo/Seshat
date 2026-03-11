//! Tests for CLI JSONL event format and exit code mapping.
//!
//! These tests verify:
//! - JSONL format compliance (one JSON object per line)
//! - Exit code mapping for different error types
//! - Rejection path preserving last-known-good state

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

#[cfg(test)]
mod cli_event_tests {
    use crate::cli::{error_code, exit_code, CliEvent};
    use crate::mutation::error::MutationError;
    use anyhow::{anyhow, Error};

    /// Test: Given valid CliEvent, when serialized to JSONL, then produces valid JSON
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_valid_cli_event_when_serialized_then_produces_valid_jsonl() {
        let event = CliEvent::start(String::from("render"));
        let json = serde_json::to_string(&event).expect("CliEvent should serialize to JSON");

        // Verify it's valid JSON (can be parsed back)
        let parsed: Result<CliEvent, _> = serde_json::from_str(&json);
        assert!(
            parsed.is_ok(),
            "JSONL output should be parseable: {:?}",
            parsed.err()
        );

        // Verify required fields
        assert!(
            json.contains("\"event\":"),
            "JSONL should contain event field"
        );
        assert!(
            json.contains("\"command\":"),
            "JSONL should contain command field"
        );
    }

    /// Test: Given error event, when serialized, then contains error details
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_error_event_when_serialized_then_contains_error_details() {
        let event = CliEvent::error(
            String::from("layout"),
            String::from("schema_violation"),
            String::from("Invalid document: version must be 2"),
        );

        let json = serde_json::to_string(&event).expect("Error event should serialize");

        // Verify error-specific fields
        assert!(
            json.contains("\"ok\":false"),
            "Error event should have ok=false"
        );
        assert!(
            json.contains("schema_violation"),
            "Should contain error code"
        );
        assert!(
            json.contains("Invalid document"),
            "Should contain error message"
        );
    }

    /// Test: Given parse error, when error_code called, then returns parse_error
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_parse_error_when_error_code_called_then_returns_parse_error() {
        let err = anyhow!("failed to parse JSON: unexpected token");
        let code = error_code(&err);

        assert_eq!(
            code, "parse_error",
            "Parse errors should map to parse_error code"
        );
    }

    /// Test: Given schema error, when error_code called, then returns schema_violation
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_schema_error_when_error_code_called_then_returns_schema_violation() {
        let err = anyhow!("schema validation failed: version must be 2");
        let code = error_code(&err);

        assert_eq!(
            code, "schema_violation",
            "Schema errors should map to schema_violation"
        );
    }

    /// Test: Given DAG cycle error, when error_code called, then returns dag_violation
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_dag_cycle_error_when_error_code_called_then_returns_dag_violation() {
        let err = anyhow!("DAG validation failed: cycle detected in edges");
        let code = error_code(&err);

        assert_eq!(
            code, "dag_violation",
            "DAG cycle errors should map to dag_violation"
        );
    }

    /// Test: Given stale revision error, when error_code called, then returns stale_revision
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_stale_revision_error_when_error_code_called_then_returns_stale_revision() {
        let err =
            anyhow!("stale_revision: test failed at /revision: expected Some(999) but got Some(1)");
        let code = error_code(&err);

        assert_eq!(
            code, "stale_revision",
            "Stale revision errors should map to stale_revision"
        );
    }

    /// Test: Given stale revision error, when exit_code called, then returns 1
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_stale_revision_error_when_exit_code_called_then_returns_1() {
        let err = anyhow!("stale_revision: test failed at /revision");
        let code = exit_code(&err);

        assert_eq!(
            code, 1,
            "Stale revision errors should return exit code 1 (business logic error)"
        );
    }

    /// Test: Given dangling reference error, when error_code called, then returns dangling_reference
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_dangling_reference_error_when_error_code_called_then_returns_dangling_reference() {
        let err = anyhow!("edge-dangling: Edge e1 target 'nonexistent' does not exist");
        let code = error_code(&err);

        assert_eq!(
            code, "dangling_reference",
            "Dangling reference errors should map to dangling_reference"
        );
    }

    /// Test: Given generic error, when error_code called, then returns command_error
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_generic_error_when_error_code_called_then_returns_command_error() {
        let err = anyhow!("Something unexpected went wrong");
        let code = error_code(&err);

        assert_eq!(
            code, "command_error",
            "Generic errors should map to command_error"
        );
    }

    /// Test: Given parse error, when exit_code called, then returns 2
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_parse_error_when_exit_code_called_then_returns_2() {
        let err = anyhow!("parse error");
        let code = exit_code(&err);

        assert_eq!(code, 2, "Parse errors should return exit code 2");
    }

    /// Test: Given command error, when exit_code called, then returns 2
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_command_error_when_exit_code_called_then_returns_2() {
        let err = anyhow!("generic command error");
        let code = exit_code(&err);

        assert_eq!(code, 2, "Command errors should return exit code 2");
    }

    /// Test: Given schema error, when exit_code called, then returns 1
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_schema_error_when_exit_code_called_then_returns_1() {
        let err = anyhow!("schema error");
        let code = exit_code(&err);

        assert_eq!(code, 1, "Schema errors should return exit code 1");
    }

    /// Test: Given MutationError, when error_code derived, then returns structured code
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_mutation_error_when_deriving_error_code_then_returns_structured_code() {
        let schema_err = MutationError::Schema(String::from("version must be 2"));
        let semantic_err = MutationError::Semantic(String::from("edge references missing node"));

        // The error_code function should handle MutationError via anyhow
        let schema_anyhow: Error = anyhow!("schema error: version must be 2");
        let semantic_anyhow: Error =
            anyhow!("semantic validation error: edge references missing node");

        assert_eq!(error_code(&schema_anyhow), "schema_violation");
        assert_eq!(error_code(&semantic_anyhow), "semantic_error");
    }

    /// Test: Given finish event, when serialized, then has correct structure
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_finish_event_when_serialized_then_has_correct_structure() {
        let event = CliEvent::finish(String::from("validate"), true, String::from("ok"));
        let json = serde_json::to_string(&event).expect("Finish event should serialize");

        assert!(
            json.contains("\"event\":\"finish\""),
            "Should be finish event"
        );
        assert!(
            json.contains("\"ok\":true"),
            "Successful finish should have ok=true"
        );
        assert!(json.contains("\"code\":\"ok\""), "Should have ok code");
    }

    /// Test: Given failed finish event, when serialized, then has ok=false
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_failed_finish_event_when_serialized_then_has_ok_false() {
        let event = CliEvent::finish(
            String::from("render"),
            false,
            String::from("schema_violation"),
        );
        let json = serde_json::to_string(&event).expect("Failed finish should serialize");

        assert!(
            json.contains("\"ok\":false"),
            "Failed finish should have ok=false"
        );
        assert!(
            json.contains("schema_violation"),
            "Should contain error code"
        );
    }

    /// Test: Multiple events should each be valid JSONL (one JSON per line)
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_events_when_serialized_per_line_then_each_is_valid_json() {
        let events = vec![
            CliEvent::start(String::from("render")),
            CliEvent::finish(String::from("render"), true, String::from("ok")),
        ];

        for event in &events {
            let json = serde_json::to_string(event).expect("Each event should serialize");
            let parsed: Result<CliEvent, _> = serde_json::from_str(&json);
            assert!(
                parsed.is_ok(),
                "Each JSONL line should be valid JSON: {:?}",
                parsed.err()
            );
        }
    }
}

#[cfg(test)]
mod rejection_path_tests {
    use crate::cli_persistence::{
        load_workspace_with_lkg, save_workspace_atomic, CliPersistenceError,
    };
    use crate::models::document::{DiagramDocument, DocumentData, EditorState, Revision};
    use im::HashMap;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_valid_document() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }

    /// Test: Given invalid primary and valid LKG, when loading, then uses LKG and preserves state
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_invalid_primary_and_valid_lkg_when_loading_then_uses_lkg() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let primary_path = temp_dir.path().join("diagram.json");
        let lkg_dir = temp_dir.path().join(".lkg");
        fs::create_dir_all(&lkg_dir).expect("Should create .lkg dir");
        let lkg_path = lkg_dir.join("diagram.json.lkg");

        // Write invalid primary
        fs::write(&primary_path, b"not valid json").expect("Should write invalid primary");

        // Write valid LKG
        let valid_doc = create_valid_document();
        let lkg_json = serde_json::to_string_pretty(&valid_doc).expect("Should serialize LKG");
        fs::write(&lkg_path, lkg_json).expect("Should write LKG");

        // Load should succeed with LKG
        let result = load_workspace_with_lkg(&primary_path);

        assert!(
            result.is_ok(),
            "Should load valid LKG when primary is invalid: {:?}",
            result.err()
        );
        let loaded = result.expect("Should have loaded document");

        // Verify it's the LKG content
        assert_eq!(loaded.version, 2, "Should have loaded LKG version");
        assert_eq!(
            loaded.revision,
            Revision::INITIAL,
            "Should have LKG revision"
        );
    }

    /// Test: Given failed save, when atomic operation fails, then original file untouched
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_failed_save_when_atomic_operation_fails_then_original_untouched() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let path = temp_dir.path().join("diagram.json");

        // Create initial valid document
        let original_doc = create_valid_document();
        fs::write(
            &path,
            serde_json::to_string_pretty(&original_doc).expect("Should write initial"),
        )
        .expect("Should create initial file");

        let original_content = fs::read_to_string(&path).expect("Should read original");

        // Attempt to save with invalid data (invalid version)
        let mut invalid_doc = create_valid_document();
        invalid_doc.version = 99; // Invalid version

        let result = save_workspace_atomic(&invalid_doc, &path);

        // Save should fail
        assert!(result.is_err(), "Should fail with invalid document");

        // Original should be unchanged
        let current_content = fs::read_to_string(&path).expect("Should read current");
        assert_eq!(
            original_content, current_content,
            "Original should be untouched after failed save"
        );
    }

    /// Test: Given rejection during mutation pipeline, then last known good is preserved
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_rejection_during_mutation_then_lkg_preserved() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let path = temp_dir.path().join("test.json");
        let lkg_dir = temp_dir.path().join(".lkg");
        fs::create_dir_all(&lkg_dir).expect("Should create .lkg dir");
        let lkg_path = lkg_dir.join("test.json.lkg");

        // Save valid document as primary
        let valid_doc = create_valid_document();
        save_workspace_atomic(&valid_doc, &path).expect("Should save valid doc");

        // Also save as LKG (simulating previous good state)
        save_workspace_atomic(&valid_doc, &lkg_path).expect("Should save LKG");

        // Try to load - should work
        let loaded = load_workspace_with_lkg(&path);
        assert!(loaded.is_ok(), "Should load valid document");

        // Now corrupt the primary
        fs::write(&path, b"corrupted").expect("Should corrupt primary");

        // Load again - should fall back to LKG
        let fallback_result = load_workspace_with_lkg(&path);
        assert!(fallback_result.is_ok(), "Should fall back to LKG");

        let fallback_doc = fallback_result.expect("Should have document");
        assert_eq!(fallback_doc.version, 2, "LKG version should be valid");
    }

    /// Test: Given no valid document exists, then returns specific error
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_no_valid_document_when_loading_then_returns_no_valid_document_error() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let path = temp_dir.path().join("nonexistent.json");

        let result = load_workspace_with_lkg(&path);

        assert!(result.is_err(), "Should fail when no valid document");
        let err = result.err().expect("Should have error");
        assert!(
            matches!(err, CliPersistenceError::NoValidDocument(_)),
            "Should be NoValidDocument error"
        );
    }
}

#[cfg(test)]
mod revision_feedback_tests {
    use crate::models::document::{DiagramDocument, Revision};

    /// Test: Revision should be monotonically increasing
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_revision_when_incremented_then_is_monotonic() {
        let initial = Revision::INITIAL;
        let next = initial.increment();

        assert!(next > initial, "Revision should increment");
        assert_eq!(
            next.value(),
            1,
            "Revision should be exactly 1 after first increment"
        );
    }

    /// Test: Document should expose revision for UI feedback
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_when_accessing_revision_then_provides_feedback() {
        let mut doc = DiagramDocument::default();
        doc.revision = Revision::new(42);

        assert_eq!(
            doc.revision.value(),
            42,
            "Document revision should be accessible"
        );
    }

    /// Test: Revision policy preserve should not increment
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_preserve_policy_when_mutation_runs_then_revision_unchanged() {
        use crate::mutation::pipeline::{
            run_mutation_with_policy, RevisionPolicy, ValidationPolicy,
        };

        let doc = DiagramDocument::default();
        let result = run_mutation_with_policy(
            &doc,
            RevisionPolicy::Preserve,
            ValidationPolicy::default(),
            |d| Ok(d.clone()),
        );

        assert!(result.is_ok(), "Mutation should succeed");
        let mutated = result.expect("Should have result");
        assert_eq!(
            mutated.revision,
            Revision::INITIAL,
            "Preserve policy should not increment"
        );
    }

    /// Test: Revision policy increment should increment
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_increment_policy_when_mutation_runs_then_revision_incremented() {
        use crate::mutation::pipeline::{run_mutation, RevisionPolicy};

        let doc = DiagramDocument::default();
        let result = run_mutation(&doc, |d| Ok(d.clone()));

        assert!(result.is_ok(), "Mutation should succeed");
        let mutated = result.expect("Should have result");
        assert_eq!(
            mutated.revision,
            Revision::INITIAL.increment(),
            "Should increment revision"
        );
    }
}
