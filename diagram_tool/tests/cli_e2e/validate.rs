use crate::common::E2eTest;

#[cfg_attr(kani, kani::proof)]
#[test]
fn valid_document_succeeds() {
    let ctx = E2eTest::setup("validate");
    ctx.write_sample();

    let res = ctx.validate();
    assert!(res.success(), "validate should succeed");
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn validate_outputs_start_and_finish_events() {
    let ctx = E2eTest::setup("validate-jsonl");
    ctx.write_sample();

    let res = ctx.validate();
    assert!(res.success(), "validate should succeed");
    assert!(res.has_event("start"), "must contain start event");
    assert!(res.has_event("finish"), "must contain finish event");
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn invalid_v2_document_emits_schema_violation() {
    let ctx = E2eTest::setup("validate-schema-error");
    ctx.write_doc(r#"{"version":1,"revision":0,"document":{"nodes":{},"edges":{}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":200.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#);

    let res = ctx.validate();
    assert!(!res.success(), "validate should fail on non-v2 schema");
    assert!(
        res.has_error_event("schema_violation"),
        "must contain schema_violation error"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn legacy_edge_alias_emits_parse_error() {
    let ctx = E2eTest::setup("validate-legacy-alias");
    ctx.write_doc(r#"{"version":2,"revision":0,"document":{"nodes":{"n1":{"kind":"node","icon":"","label":"API","x":0.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"},"n2":{"kind":"node","icon":"","label":"DB","x":100.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"n2","label":"calls","style":"solid","arrow_type":"diamond","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":200.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#);

    let res = ctx.validate();
    assert!(!res.success(), "validate should fail on legacy alias field");
    assert!(
        res.has_error_event("parse_error"),
        "must contain parse_error event"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn dag_cycle_fails_with_dag_error() {
    let ctx = E2eTest::setup("dag-cycle");
    ctx.write_doc(r#"{"version":2,"revision":1,"document":{"nodes":{"n1":{"kind":"node","icon":"","label":"A","x":0.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"},"n2":{"kind":"node","icon":"","label":"B","x":100.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"n2","label":"","style":"solid","arrowType":"default","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}},"e2":{"source":"n2","target":"n1","label":"","style":"solid","arrowType":"default","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#);

    let res = ctx.validate();
    assert!(!res.success(), "validate should fail on DAG cycle");
    assert!(
        res.has_error_event("dag_violation"),
        "must contain dag_violation error"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn dangling_edge_fails_with_dangling_error() {
    let ctx = E2eTest::setup("dangling-edge");
    ctx.write_doc(r#"{"version":2,"revision":1,"document":{"nodes":{"n1":{"kind":"node","icon":"","label":"A","x":0.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"nonexistent","label":"","style":"solid","arrowType":"default","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#);

    let res = ctx.validate();
    assert!(!res.success(), "validate should fail on dangling edge");
    assert!(
        res.has_error_event("dangling_reference"),
        "must contain dangling_reference error"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn self_loop_edge_fails_with_dag_error() {
    let ctx = E2eTest::setup("self-loop");
    ctx.write_doc(r#"{"version":2,"revision":1,"document":{"nodes":{"n1":{"kind":"node","icon":"","label":"Loop","x":0.0,"y":0.0,"width":80.0,"height":60.0,"locked":false,"parent":null,"dag_rank":null,"tags":[],"metadata":{},"z_index":0,"style":"box"}},"edges":{"e1":{"source":"n1","target":"n1","label":"self","style":"solid","arrowType":"default","label_offset_t":0.5,"color":null,"thickness":1.5,"directed":true,"bend_points":[],"tags":[],"metadata":{}}}},"editor_state":{"camera_x":0.0,"camera_y":0.0,"zoom":1.0,"grid_size":20.0,"snap_to_grid":true,"selected_items":[],"editing_edge_id":null,"theme":"system","show_grid":true,"minimap_visible":false}}"#);

    let res = ctx.validate();
    assert!(!res.success(), "validate should fail on self-loop edge");
    assert!(
        res.has_error_event("dag_violation"),
        "must contain dag_violation error"
    );
}
