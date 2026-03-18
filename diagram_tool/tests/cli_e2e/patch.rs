use crate::common::E2eTest;
use std::fs;

#[cfg_attr(kani, kani::proof)]
#[test]
fn valid_patch_writes_updated_document() {
    let ctx = E2eTest::setup("patch");
    ctx.write_sample();

    let patch = ctx.write_file(
        "patch.json",
        r#"[
  {"op":"test","path":"/revision","value":1},
  {"op":"replace","path":"/document/nodes/n1/label","value":"Gateway"}
]"#,
    );
    let output_path = ctx.dir.join("patched.json");

    let res = ctx.patch(&patch, &output_path);
    assert!(res.success(), "patch command should succeed");

    let patched = fs::read_to_string(output_path).unwrap();
    assert!(
        patched.contains("Gateway"),
        "patched label should be written"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn patch_without_revision_test_fails_closed() {
    let ctx = E2eTest::setup("patch-no-revision-test");
    ctx.write_sample();

    let patch = ctx.write_file(
        "patch.json",
        r#"[{"op":"replace","path":"/document/nodes/n1/label","value":"Gateway"}]"#,
    );
    let output_path = ctx.dir.join("patched.json");

    let res = ctx.patch(&patch, &output_path);
    assert!(
        !res.success(),
        "patch should fail closed without revision test op"
    );
    assert!(
        !output_path.exists(),
        "failed patch should not write output document"
    );

    assert!(
        res.has_error_event("command_error"),
        "must include command_error event"
    );
    assert!(
        res.has_finish_event_ok(false),
        "must include non-ok finish event"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn stale_revision_document_fails() {
    let ctx = E2eTest::setup("stale-revision");
    ctx.write_sample();

    let patch = ctx.write_file(
        "patch.json",
        r#"[
  {"op":"test","path":"/revision","value":999},
  {"op":"replace","path":"/document/nodes/n1/label","value":"Stale"}
]"#,
    );
    let output_path = ctx.dir.join("patched.json");

    let res = ctx.patch(&patch, &output_path);
    assert!(!res.success(), "patch should fail when revision is stale");
    assert!(
        res.has_error_event("stale_revision"),
        "must contain stale_revision error"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn failed_patch_preserves_last_known_good() {
    let ctx = E2eTest::setup("lkg-preservation");
    ctx.write_doc(r#"{"version":2,"revision":1,"document":{"nodes":{"n1":{"kind":"node","icon":"aws/compute/ec2","label":"API","x":10.0,"y":20.0,"width":80.0,"height":60.0,"locked":true,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#);

    let patch = ctx.write_file(
        "patch.json",
        r#"[
  {"op":"test","path":"/revision","value":999},
  {"op":"replace","path":"/document/nodes/n1/label","value":"StalePatch"}
]"#,
    );
    let output_path = ctx.dir.join("patched.json");

    let res = ctx.patch(&patch, &output_path);
    assert!(!res.success(), "patch should fail");

    let lkg_path = ctx.dir.join(".lkg").join("input.json.lkg");
    assert!(
        lkg_path.exists(),
        "last known good should exist after failed patch"
    );

    let lkg_content = fs::read_to_string(&lkg_path).unwrap();
    assert!(
        lkg_content.contains("API"),
        "lkg should contain original label"
    );
}
