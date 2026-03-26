use assert_cmd::Command;
use diagram_models::document::DiagramDocument;
use std::io::Write;

#[test]
fn e2e_apply_exits_zero_and_returns_queued_on_success() {
    let doc = DiagramDocument::default();
    let doc_json = serde_json::to_string(&doc).expect("serialize document");

    let proposal_json = serde_json::json!({
        "base_revision": doc.revision.value(),
        "proposer": "agent1",
        "proposed_at": 0,
        "summary": "Test"
    })
    .to_string();

    let mut tmp_file = tempfile::NamedTempFile::new().expect("tempfile creation");
    tmp_file
        .write_all(proposal_json.as_bytes())
        .expect("write to temp file");

    let mut cmd = Command::cargo_bin("seshat").expect("cargo binary lookup");
    cmd.arg("apply")
        .arg("--proposal")
        .arg(tmp_file.path())
        .write_stdin(doc_json)
        .assert()
        .success()
        .stdout(predicates::str::contains("queued"));
}

#[test]
fn e2e_apply_exits_zero_and_returns_rejected_on_mismatch() {
    let doc = DiagramDocument::default();
    let doc_json = serde_json::to_string(&doc).expect("serialize document");

    let proposal_json = serde_json::json!({
        "base_revision": 999,
        "proposer": "agent1",
        "proposed_at": 0,
        "summary": "Test"
    })
    .to_string();

    let mut tmp_file = tempfile::NamedTempFile::new().expect("tempfile creation");
    tmp_file
        .write_all(proposal_json.as_bytes())
        .expect("write to temp file");

    let mut cmd = Command::cargo_bin("seshat").expect("cargo binary lookup");
    cmd.arg("apply")
        .arg("--proposal")
        .arg(tmp_file.path())
        .write_stdin(doc_json)
        .assert()
        .success()
        .stdout(predicates::str::contains("rejected"))
        .stdout(predicates::str::contains("Human Priority Block"));
}

#[test]
fn e2e_apply_exits_non_zero_on_invalid_proposal() {
    let doc = DiagramDocument::default();
    let doc_json = serde_json::to_string(&doc).expect("serialize document");

    let proposal_json = serde_json::json!({
        "proposer": "agent1",
        "summary": "Missing base_revision"
    })
    .to_string();

    let mut tmp_file = tempfile::NamedTempFile::new().expect("tempfile creation");
    tmp_file
        .write_all(proposal_json.as_bytes())
        .expect("write to temp file");

    let mut cmd = Command::cargo_bin("seshat").expect("cargo binary lookup");
    cmd.arg("apply")
        .arg("--proposal")
        .arg(tmp_file.path())
        .write_stdin(doc_json)
        .assert()
        .failure()
        .stderr(predicates::str::contains("error: apply: invalid proposal"));
}
