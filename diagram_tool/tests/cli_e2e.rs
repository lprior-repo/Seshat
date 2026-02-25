use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |d| d.as_nanos());
    let dir = std::env::temp_dir().join(format!("diagram-tool-{prefix}-{nanos}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn write_sample_doc(path: &PathBuf) -> std::io::Result<()> {
    let content = r#"{"version":2,"revision":1,"document":{"nodes":{"n1":{"kind":"node","icon":"aws/compute/ec2","label":"API","x":10.0,"y":20.0,"width":80.0,"height":60.0,"locked":true,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"},"n2":{"kind":"node","icon":"aws/database/rds","label":"DB","x":220.0,"y":40.0,"width":80.0,"height":60.0,"locked":true,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"n2","label":"calls","style":"solid","arrowhead":"arrow","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":200.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#;

    fs::write(path, content)
}

fn run_diagram_tool(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_diagram_tool"))
        .args(args)
        .output()
}

fn parse_jsonl_events(stdout: Vec<u8>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let parsed = String::from_utf8(stdout)?
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect::<Vec<_>>();
    Ok(parsed)
}

#[test]
fn given_valid_document_when_validate_command_runs_then_it_succeeds(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("validate");
    let input = dir.join("input.json");
    write_sample_doc(&input)?;

    let output = run_diagram_tool(&["validate", "--input", input.to_string_lossy().as_ref()])?;

    assert!(output.status.success(), "validate should succeed");
    Ok(())
}

#[test]
fn given_valid_patch_when_patch_command_runs_then_it_writes_updated_document(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("patch");
    let input = dir.join("input.json");
    let patch = dir.join("patch.json");
    let output_path = dir.join("patched.json");
    write_sample_doc(&input)?;

    let patch_content = r#"[
  {"op":"test","path":"/revision","value":1},
  {"op":"replace","path":"/document/nodes/n1/label","value":"Gateway"}
]"#;
    fs::write(&patch, patch_content)?;

    let output = run_diagram_tool(&[
        "patch",
        "--input",
        input.to_string_lossy().as_ref(),
        "--patch",
        patch.to_string_lossy().as_ref(),
        "--output",
        output_path.to_string_lossy().as_ref(),
    ])?;

    assert!(output.status.success(), "patch command should succeed");

    let patched = fs::read_to_string(output_path)?;
    assert!(
        patched.contains("Gateway"),
        "patched label should be written"
    );
    Ok(())
}

#[test]
fn given_valid_document_when_layout_command_runs_then_output_contains_nodes_and_edges(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("layout");
    let input = dir.join("input.json");
    let output_path = dir.join("layout.json");
    write_sample_doc(&input)?;

    let output = run_diagram_tool(&[
        "layout",
        "--input",
        input.to_string_lossy().as_ref(),
        "--output",
        output_path.to_string_lossy().as_ref(),
    ])?;

    assert!(output.status.success(), "layout command should succeed");

    let laid_out = fs::read_to_string(output_path)?;
    assert!(
        laid_out.contains("\"nodes\""),
        "output should contain nodes"
    );
    assert!(
        laid_out.contains("\"edges\""),
        "output should contain edges"
    );
    Ok(())
}

#[test]
fn given_valid_document_when_render_svg_command_runs_then_svg_file_is_generated(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("render-svg");
    let input = dir.join("input.json");
    let output_path = dir.join("diagram.svg");
    write_sample_doc(&input)?;

    let output = run_diagram_tool(&[
        "render",
        "--input",
        input.to_string_lossy().as_ref(),
        "--output",
        output_path.to_string_lossy().as_ref(),
    ])?;

    assert!(output.status.success(), "render svg command should succeed");

    let svg = fs::read_to_string(output_path)?;
    assert!(svg.contains("<svg"), "svg output should contain svg root");
    Ok(())
}

#[test]
fn given_validate_command_when_run_then_it_outputs_jsonl_start_and_finish_events(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("validate-jsonl");
    let input = dir.join("input.json");
    write_sample_doc(&input)?;

    let output = run_diagram_tool(&["validate", "--input", input.to_string_lossy().as_ref()])?;

    assert!(output.status.success(), "validate should succeed");

    let lines = parse_jsonl_events(output.stdout)?;

    assert!(
        lines
            .iter()
            .any(|v| v.get("event") == Some(&Value::String(String::from("start")))),
        "JSONL output must contain start event"
    );
    assert!(
        lines
            .iter()
            .any(|v| v.get("event") == Some(&Value::String(String::from("finish")))),
        "JSONL output must contain finish event"
    );
    Ok(())
}

#[test]
fn given_invalid_v2_document_when_validate_runs_then_schema_violation_is_emitted(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("validate-schema-error");
    let input = dir.join("bad-version.json");
    let bad_doc = r#"{"version":1,"revision":0,"document":{"nodes":{},"edges":{}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":200.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#;
    fs::write(&input, bad_doc)?;

    let output = run_diagram_tool(&["validate", "--input", input.to_string_lossy().as_ref()])?;
    assert!(
        !output.status.success(),
        "validate should fail on non-v2 schema"
    );

    let events = parse_jsonl_events(output.stdout)?;
    assert!(
        events.iter().any(|v| {
            v.get("event") == Some(&Value::String(String::from("error")))
                && v.get("code") == Some(&Value::String(String::from("schema_violation")))
        }),
        "JSONL output must contain schema_violation error event"
    );
    Ok(())
}

#[test]
fn given_legacy_edge_alias_when_validate_runs_then_parse_error_is_emitted(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("validate-legacy-alias");
    let input = dir.join("legacy-alias.json");
    let legacy_alias_doc = r#"{"version":2,"revision":0,"document":{"nodes":{"n1":{"kind":"node","icon":"","label":"API","x":0.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"},"n2":{"kind":"node","icon":"","label":"DB","x":100.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"n2","label":"calls","style":"solid","arrow_type":"diamond","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":200.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#;
    fs::write(&input, legacy_alias_doc)?;

    let output = run_diagram_tool(&["validate", "--input", input.to_string_lossy().as_ref()])?;
    assert!(
        !output.status.success(),
        "validate should fail on legacy alias field"
    );

    let events = parse_jsonl_events(output.stdout)?;
    assert!(
        events.iter().any(|v| {
            v.get("event") == Some(&Value::String(String::from("error")))
                && v.get("code") == Some(&Value::String(String::from("parse_error")))
        }),
        "JSONL output must contain parse_error event for legacy alias input"
    );
    Ok(())
}

#[test]
fn given_patch_without_first_revision_test_when_patch_runs_then_fail_closed_error_events_are_emitted(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = unique_test_dir("patch-no-revision-test");
    let input = dir.join("input.json");
    let patch = dir.join("patch.json");
    let output_path = dir.join("patched.json");
    write_sample_doc(&input)?;
    fs::write(
        &patch,
        r#"[{"op":"replace","path":"/document/nodes/n1/label","value":"Gateway"}]"#,
    )?;

    let output = run_diagram_tool(&[
        "patch",
        "--input",
        input.to_string_lossy().as_ref(),
        "--patch",
        patch.to_string_lossy().as_ref(),
        "--output",
        output_path.to_string_lossy().as_ref(),
    ])?;

    assert!(
        !output.status.success(),
        "patch should fail closed without revision test op"
    );
    assert!(
        !output_path.exists(),
        "failed patch should not write output document"
    );

    let events = parse_jsonl_events(output.stdout)?;
    assert!(
        events.iter().any(|v| {
            v.get("event") == Some(&Value::String(String::from("error")))
                && v.get("code") == Some(&Value::String(String::from("command_error")))
        }),
        "JSONL output must include command_error error event"
    );
    assert!(
        events.iter().any(|v| {
            v.get("event") == Some(&Value::String(String::from("finish")))
                && v.get("ok") == Some(&Value::Bool(false))
        }),
        "JSONL output must include non-ok finish event"
    );
    Ok(())
}
