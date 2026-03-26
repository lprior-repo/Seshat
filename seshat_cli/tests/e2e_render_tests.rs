//! End-to-end tests for the `seshat` binary — `render` subcommand.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::redundant_clone)]

use std::io::Write;
use std::process::Command;

use diagram_models::document::DiagramDocument;

fn seshat_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seshat"))
}

fn write_to_temp_file(contents: &[u8]) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("temp file creation must succeed");
    f.write_all(contents)
        .expect("write to temp file must succeed");
    f
}

#[test]
fn e2e_render_exits_zero_and_creates_svg_when_valid_file_provided() {
    let doc = DiagramDocument::default();
    let json = serde_json::to_string(&doc).expect("must serialize");
    let input = write_to_temp_file(json.as_bytes());

    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("output.svg");

    let output = seshat_bin()
        .arg("render")
        .arg("-i")
        .arg(input.path())
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(0),
        "exit code must be 0 for valid file; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "SVG output file must be created");
    let svg_content = std::fs::read_to_string(&output_path).unwrap();
    assert!(svg_content.contains("<svg"), "Output must contain <svg tag");
}

#[test]
fn e2e_render_exits_zero_and_creates_png_when_valid_file_provided() {
    let doc = DiagramDocument::default();
    let json = serde_json::to_string(&doc).expect("must serialize");
    let input = write_to_temp_file(json.as_bytes());

    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("output.png");

    let output = seshat_bin()
        .arg("render")
        .arg("-i")
        .arg(input.path())
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("seshat binary must be spawnable");

    assert_eq!(
        output.status.code(),
        Some(0),
        "exit code must be 0 for valid file; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "PNG output file must be created");
    let png_content = std::fs::read(&output_path).unwrap();
    assert!(
        png_content.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]),
        "Output must have PNG signature"
    );
}

#[test]
fn e2e_render_exits_non_zero_for_unsupported_format() {
    let doc = DiagramDocument::default();
    let json = serde_json::to_string(&doc).expect("must serialize");
    let input = write_to_temp_file(json.as_bytes());

    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("output.txt");

    let output = seshat_bin()
        .arg("render")
        .arg("-i")
        .arg(input.path())
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("seshat binary must be spawnable");

    assert_ne!(
        output.status.code(),
        Some(0),
        "exit code must be non-zero for unsupported format"
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("unsupported output format: txt"),
        "stderr must indicate unsupported format, got: {stderr_str:?}"
    );
}

#[test]
fn e2e_render_exits_non_zero_for_invalid_input() {
    let input = write_to_temp_file(b"invalid json");

    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("output.svg");

    let output = seshat_bin()
        .arg("render")
        .arg("-i")
        .arg(input.path())
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("seshat binary must be spawnable");

    assert_ne!(
        output.status.code(),
        Some(0),
        "exit code must be non-zero for invalid JSON input"
    );

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr_str.contains("JSON parse error"),
        "stderr must indicate JSON parse error, got: {stderr_str:?}"
    );
}
