use diagram_models::document::DiagramDocument;
use seshat_cli::apply::{execute_apply, map_apply_subcommand};
use seshat_cli::domain::{ApplyCommand, ApplySource};
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn map_apply_subcommand_maps_correctly_with_stdin() {
    let proposal = PathBuf::from("proposal.json");
    let cmd = map_apply_subcommand(None, proposal.clone());

    assert_eq!(cmd.document, ApplySource::Stdin);
    assert_eq!(cmd.proposal, proposal);
}

#[test]
fn map_apply_subcommand_maps_correctly_with_files() {
    let doc = PathBuf::from("doc.json");
    let proposal = PathBuf::from("proposal.json");
    let cmd = map_apply_subcommand(Some(doc.clone()), proposal.clone());

    assert_eq!(cmd.document, ApplySource::File(doc));
    assert_eq!(cmd.proposal, proposal);
}

#[test]
fn execute_apply_succeeds_when_revisions_match() {
    let doc = DiagramDocument::default();
    let doc_json = serde_json::to_string(&doc).unwrap();

    let proposal_json = serde_json::json!({
        "base_revision": doc.revision.value(),
        "proposer": "agent1",
        "proposed_at": 0,
        "summary": "Test"
    })
    .to_string();

    let tmp_dir = tempfile::tempdir().unwrap();
    let proposal_path = tmp_dir.path().join("proposal.json");
    std::fs::write(&proposal_path, proposal_json).unwrap();

    let cmd = ApplyCommand {
        document: ApplySource::Stdin,
        proposal: proposal_path,
    };

    let stdin = Cursor::new(doc_json);
    let mut stdout = Vec::new();

    let result = execute_apply(&cmd, stdin, &mut stdout);
    assert!(result.is_ok());

    let output: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(output["status"], "queued");
    assert_eq!(output["base_revision"], doc.revision.value());
}

#[test]
fn execute_apply_fails_when_revisions_mismatch() {
    let doc = DiagramDocument::default();
    let doc_json = serde_json::to_string(&doc).unwrap();

    let proposal_json = serde_json::json!({
        "base_revision": 999,
        "proposer": "agent1",
        "proposed_at": 0,
        "summary": "Test"
    })
    .to_string();

    let tmp_dir = tempfile::tempdir().unwrap();
    let proposal_path = tmp_dir.path().join("proposal.json");
    std::fs::write(&proposal_path, proposal_json).unwrap();

    let cmd = ApplyCommand {
        document: ApplySource::Stdin,
        proposal: proposal_path,
    };

    let stdin = Cursor::new(doc_json);
    let mut stdout = Vec::new();

    let result = execute_apply(&cmd, stdin, &mut stdout);
    assert!(result.is_ok()); // The execution succeeds, but output is "rejected"

    let output: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(output["status"], "rejected");
    assert_eq!(output["conflict_context"]["expected_revision"], 999);
    assert_eq!(
        output["conflict_context"]["actual_revision"],
        doc.revision.value()
    );
}

#[test]
fn execute_apply_returns_error_on_invalid_proposal() {
    let doc = DiagramDocument::default();
    let doc_json = serde_json::to_string(&doc).unwrap();

    // Missing base_revision
    let proposal_json = serde_json::json!({
        "proposer": "agent1",
    })
    .to_string();

    let tmp_dir = tempfile::tempdir().unwrap();
    let proposal_path = tmp_dir.path().join("proposal.json");
    std::fs::write(&proposal_path, proposal_json).unwrap();

    let cmd = ApplyCommand {
        document: ApplySource::Stdin,
        proposal: proposal_path,
    };

    let stdin = Cursor::new(doc_json);
    let mut stdout = Vec::new();

    let result = execute_apply(&cmd, stdin, &mut stdout);
    assert!(matches!(
        result,
        Err(seshat_cli::error::ApplyError::InvalidProposal(_))
    ));
}
