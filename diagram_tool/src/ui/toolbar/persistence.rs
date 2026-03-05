use crate::history::History;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::canonical_json::to_canonical_pretty_json;
use crate::models::document::{ArrowType, DiagramDocument, EdgeStyle, Revision};
use crate::mutation::pipeline::{run_mutation_with_policy, RevisionPolicy};
use crate::ui::editor::ToolMode;
use crate::ui::toast::{ToastApi, ToastHandle, ToastIntent, ToastOptions, ToastQueue, ToastUpdate};
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[derive(Debug)]
enum ImportTransitionError {
    Parse(String),
    Validation(String),
}

fn prepare_import_transition(
    current: &DiagramDocument,
    contents: &str,
) -> Result<(DiagramDocument, History), ImportTransitionError> {
    let mut loaded_doc = super::persistence_compat::parse_diagram_document_with_compat(contents)
        .map_err(ImportTransitionError::Parse)?;
    loaded_doc.revision = Revision::INITIAL;

    run_mutation_with_policy(current, RevisionPolicy::Preserve, ValidationPolicy::default(), |_| Ok(loaded_doc))
        .map(|next_doc| (next_doc, History::new().push(current.clone())))
        .map_err(|err| {
            ImportTransitionError::Validation(super::mutation_error_code(&err).to_string())
        })
}

fn apply_import_contents(
    doc: &mut DiagramDocument,
    history: &mut History,
    contents: &str,
) -> Result<(), ImportTransitionError> {
    let current = doc.clone();
    match prepare_import_transition(&current, contents) {
        Ok((next_doc, next_history)) => {
            *doc = next_doc;
            *history = next_history;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

pub fn save_workspace(
    doc_signal: Signal<DiagramDocument>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    toasts: Signal<ToastQueue>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Saving workspace").with_detail("Preparing data..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let _ = doc_signal;
        let _ = toast_handle.dismiss();
        let toast_api = ToastApi::from_signal(toasts);
        let _ = toast_api.toast(
            ToastOptions::new(ToastIntent::Error, "Save not available")
                .with_detail("Backend has been decommissioned"),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&tool_signal, &edge_style_signal, &arrow_type_signal);
        let doc_snapshot = doc_signal.read().clone();
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .set_file_name("diagram.json")
                .save_file();
            match path {
                None => {
                    let _ = toast_handle.dismiss();
                }
                Some(p) => match to_canonical_pretty_json(&doc_snapshot) {
                    Ok(json_str) => match fs::write(&p, json_str.as_bytes()) {
                        Ok(()) => {
                            update_load_save_success(
                                toast_handle,
                                "Workspace saved",
                                format!("Saved to {}", p.display()),
                            );
                        }
                        Err(e) => update_load_save_error(
                            toast_handle,
                            "Save failed",
                            format!("Save error: {e}"),
                        ),
                    },
                    Err(e) => update_load_save_error(
                        toast_handle,
                        "Save failed",
                        format!("Serialize error: {e}"),
                    ),
                },
            }
        });
    }
}
#[allow(clippy::too_many_lines)]
pub fn open_workspace(
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
    tool_signal: Signal<ToolMode>,
    edge_style_signal: Signal<EdgeStyle>,
    arrow_type_signal: Signal<ArrowType>,
    toasts: Signal<ToastQueue>,
) {
    let toast_api = ToastApi::from_signal(toasts);
    let toast_handle = toast_api.toast(
        ToastOptions::new(ToastIntent::Info, "Loading workspace")
            .with_detail("Reading persisted document..."),
    );
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (&tool_signal, &edge_style_signal, &arrow_type_signal);
        spawn(async move {
            let mut eval = document::eval(
                r#"
                (function() {
                    if (window.__SESHAT_E2E_IMPORT_JSON) {
                        const contents = window.__SESHAT_E2E_IMPORT_JSON;
                        delete window.__SESHAT_E2E_IMPORT_JSON;
                        dioxus.send({ ok: true, contents });
                        return;
                    }
                    const input = document.createElement('input');
                    input.type = 'file';
                    input.accept = '.json,application/json';
                    input.style.display = 'none';
                    let settled = false;
                    const finish = (payload) => {
                        if (settled) return;
                        settled = true;
                        window.removeEventListener('focus', onFocus, true);
                        dioxus.send(payload);
                    };
                    const onFocus = () => {
                        setTimeout(() => {
                            finish({ ok: false, cancelled: true });
                        }, 150);
                    };
                    window.addEventListener('focus', onFocus, true);
                    input.addEventListener('change', () => {
                        const file = input.files && input.files[0];
                        if (!file) {
                            finish({ ok: false, cancelled: true });
                            return;
                        }

                        const reader = new FileReader();
                        reader.onload = () => {
                            finish({ ok: true, contents: String(reader.result || '') });
                        };
                        reader.onerror = () => {
                            finish({ ok: false, cancelled: false, error: 'read-failed' });
                        };
                        reader.readAsText(file);
                    });
                    input.click();
                })();
                "#,
            );

            match eval.recv::<serde_json::Value>().await {
                Ok(msg) => {
                    if msg["cancelled"].as_bool().is_some_and(|v| v) {
                        let _ = toast_handle.dismiss();
                        return;
                    }

                    if msg["ok"].as_bool() != Some(true) {
                        let detail = msg["error"].as_str().map_or_else(
                            || String::from("Browser file import failed"),
                            String::from,
                        );
                        update_load_save_error(toast_handle, "Load failed", detail);
                        return;
                    }

                    let contents = msg["contents"].as_str().map_or("", |v| v);
                    let mut next_doc = doc_signal.read().clone();
                    let mut next_history = history_signal.read().clone();
                    match apply_import_contents(&mut next_doc, &mut next_history, contents) {
                        Ok(()) => {
                            *doc_signal.write() = next_doc;
                            *history_signal.write() = next_history;
                            update_load_save_success(
                                toast_handle,
                                "Workspace loaded",
                                String::from("Loaded from local JSON"),
                            );
                        }
                        Err(ImportTransitionError::Parse(err)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {err}"),
                            );
                        }
                        Err(ImportTransitionError::Validation(code)) => {
                            update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Load validation error: {code}"),
                            );
                        }
                    }
                }
                Err(err) => update_load_save_error(
                    toast_handle,
                    "Load failed",
                    format!("Import bridge error: {err}"),
                ),
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (&tool_signal, &edge_style_signal, &arrow_type_signal);
        spawn(async move {
            let path = FileDialog::new()
                .add_filter("Seshat Diagram", &["json"])
                .pick_file();
            match path {
                None => {
                    let _ = toast_handle.dismiss();
                }
                Some(p) => match fs::read_to_string(&p) {
                    Err(e) => update_load_save_error(
                        toast_handle,
                        "Load failed",
                        format!("Read error: {e}"),
                    ),
                    Ok(contents) => {
                        let mut next_doc = doc_signal.read().clone();
                        let mut next_history = history_signal.read().clone();
                        match apply_import_contents(&mut next_doc, &mut next_history, &contents) {
                            Ok(()) => {
                                *doc_signal.write() = next_doc;
                                *history_signal.write() = next_history;
                                update_load_save_success(
                                    toast_handle,
                                    "Workspace loaded",
                                    format!("Loaded from {}", p.display()),
                                );
                            }
                            Err(ImportTransitionError::Parse(e)) => update_load_save_error(
                                toast_handle,
                                "Load failed",
                                format!("Parse error: {e}"),
                            ),
                            Err(ImportTransitionError::Validation(code)) => {
                                update_load_save_error(
                                    toast_handle,
                                    "Load failed",
                                    format!("Load validation error: {code}"),
                                );
                            }
                        }
                    }
                },
            }
        });
    }
}
fn update_load_save_success(toast_handle: ToastHandle, title: &str, detail: String) {
    let _ = toast_handle.update(ToastUpdate {
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Success),
        action: None,
    });
}
fn update_load_save_error(toast_handle: ToastHandle, title: &str, detail: String) {
    let _ = toast_handle.update(ToastUpdate {
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Error),
        action: None,
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{apply_import_contents, prepare_import_transition, ImportTransitionError};
    use crate::history::History;
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::{HashMap, HashSet};

    fn sample_doc_with_node(id: &str, x: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        let _ = doc.document.nodes.insert(
            NodeId::new(id.to_string()),
            Node {
                kind: NodeKind::Text,
                icon: String::new(),
                label: String::from("Text"),
                x: OrderedFloat(x),
                y: OrderedFloat(120.0),
                width: OrderedFloat(100.0),
                height: OrderedFloat(24.0),
                font_size: None,
                font_weight: None,
                locked: true,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: Some(NodeStyle::default()),
                collapsed: None,
            },
        );
        doc
    }

    #[test]
    fn given_malformed_import_when_preparing_transition_then_returns_parse_error() {
        let current = sample_doc_with_node("n-current", 40.0);
        let result = prepare_import_transition(&current, "{this-is-not-json");
        assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
    }

    #[test]
    fn given_semantically_invalid_import_when_preparing_transition_then_returns_validation_error() {
        let current = sample_doc_with_node("n-current", 40.0);
        let invalid = r#"{
            "version": 2,
            "revision": 0,
            "document": {
                "nodes": {},
                "edges": {
                    "e1": {
                        "source": "missing-a",
                        "target": "missing-b"
                    }
                }
            }
        }"#;

        let result = prepare_import_transition(&current, invalid);
        assert!(matches!(result, Err(ImportTransitionError::Validation(_))));
    }

    #[test]
    fn given_valid_import_when_preparing_transition_then_new_doc_and_history_are_atomic() {
        let current = sample_doc_with_node("n-current", 40.0);
        let valid = serde_json::to_string_pretty(&sample_doc_with_node("n-import", 260.0)).unwrap();

        let (next_doc, next_history) = prepare_import_transition(&current, &valid)
            .expect("valid import should produce a transition");
        assert!(next_doc
            .document
            .nodes
            .contains_key(&NodeId::new(String::from("n-import"))));
        assert!(!next_doc
            .document
            .nodes
            .contains_key(&NodeId::new(String::from("n-current"))));

        let undone = next_history.undo(next_doc.clone());
        assert!(
            undone.is_some(),
            "history should include pre-import snapshot"
        );
        let (restored, _) = undone.expect("undo should restore prior state");
        assert!(restored
            .document
            .nodes
            .contains_key(&NodeId::new(String::from("n-current"))));
        assert!(!restored
            .document
            .nodes
            .contains_key(&NodeId::new(String::from("n-import"))));

        let fresh_history = History::new();
        assert!(fresh_history.undo(current).is_none());
    }

    #[test]
    fn given_import_error_when_applying_contents_then_doc_and_history_remain_unchanged() {
        let mut doc = sample_doc_with_node("n-current", 40.0);
        let previous = sample_doc_with_node("n-prev", 12.0);
        let mut history = History::new().push(previous.clone());

        let doc_before = doc.clone();
        let undo_before = history
            .clone()
            .undo(doc.clone())
            .map(|(snapshot, _)| snapshot);

        let result = apply_import_contents(&mut doc, &mut history, "{not-valid-json");
        assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
        assert_eq!(doc, doc_before);

        let undo_after = history.undo(doc.clone()).map(|(snapshot, _)| snapshot);
        assert_eq!(undo_after, undo_before);
        assert_eq!(undo_after, Some(previous));
    }

    #[test]
    fn given_validation_error_when_applying_contents_then_doc_and_history_remain_unchanged() {
        let mut doc = sample_doc_with_node("n-current", 40.0);
        let previous = sample_doc_with_node("n-prev", 12.0);
        let mut history = History::new().push(previous.clone());

        let invalid = r#"{
            "version": 2,
            "revision": 0,
            "document": {
                "nodes": {},
                "edges": {
                    "e1": {
                        "source": "missing-a",
                        "target": "missing-b"
                    }
                }
            }
        }"#;

        let doc_before = doc.clone();
        let undo_before = history
            .clone()
            .undo(doc.clone())
            .map(|(snapshot, _)| snapshot);

        let result = apply_import_contents(&mut doc, &mut history, invalid);
        assert!(matches!(result, Err(ImportTransitionError::Validation(_))));
        assert_eq!(doc, doc_before);

        let undo_after = history.undo(doc.clone()).map(|(snapshot, _)| snapshot);
        assert_eq!(undo_after, undo_before);
        assert_eq!(undo_after, Some(previous));
    }

    #[test]
    fn given_import_error_when_selection_exists_then_selection_is_preserved() {
        let mut doc = sample_doc_with_node("n-current", 40.0);
        doc.editor_state.selected_items = HashSet::new().update(String::from("n-current"));
        let mut history = History::new();

        let selected_before = doc.editor_state.selected_items.clone();
        let result = apply_import_contents(&mut doc, &mut history, "{not-valid-json");

        assert!(matches!(result, Err(ImportTransitionError::Parse(_))));
        assert_eq!(doc.editor_state.selected_items, selected_before);
    }

    /// IO-TEST-3: Save/Reopen Exact Geometry (bd-1u1)
    /// Given: A document with nodes at precise fractional coordinates
    /// When: Saving to JSON and reopening
    /// Then: All geometry values are exactly preserved
    #[test]
    fn given_document_with_fractional_coords_when_round_trip_then_geometry_preserved() {
        use crate::models::canonical_json::to_canonical_pretty_json;
        use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;

        // Given - document with precise fractional coordinates
        let mut doc = DiagramDocument::default();
        let precise_x = 123.456789;
        let precise_y = 987.654321;
        let precise_width = 45.125;
        let precise_height = 67.875;

        let _ = doc.document.nodes.insert(
            NodeId::new("precise-node".to_string()),
            Node {
                kind: NodeKind::Text,
                icon: String::new(),
                label: String::from("Precise"),
                x: OrderedFloat(precise_x),
                y: OrderedFloat(precise_y),
                width: OrderedFloat(precise_width),
                height: OrderedFloat(precise_height),
                font_size: None,
                font_weight: None,
                locked: false,
                parent: None,
                dag_rank: None,
                tags: Vec::new(),
                metadata: HashMap::new(),
                z_index: 0,
                style: None,
                collapsed: None,
            },
        );

        // When - serialize to JSON and reload
        let json = to_canonical_pretty_json(&doc).expect("serialization should succeed");
        let reloaded: DiagramDocument =
            parse_diagram_document_with_compat(&json).expect("parsing should succeed");

        // Then - geometry should be exactly preserved
        let reloaded_node = reloaded
            .document
            .nodes
            .get(&NodeId::new("precise-node".to_string()))
            .expect("node should exist");

        assert_eq!(
            reloaded_node.x.0, precise_x,
            "x coordinate should be exactly preserved"
        );
        assert_eq!(
            reloaded_node.y.0, precise_y,
            "y coordinate should be exactly preserved"
        );
        assert_eq!(
            reloaded_node.width.0, precise_width,
            "width should be exactly preserved"
        );
        assert_eq!(
            reloaded_node.height.0, precise_height,
            "height should be exactly preserved"
        );
    }

    /// IO-TEST-3b: Multiple nodes with various precision levels
    #[test]
    fn given_document_with_various_precision_coords_when_round_trip_then_all_preserved() {
        use crate::models::canonical_json::to_canonical_pretty_json;
        use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;

        // Given
        let mut doc = DiagramDocument::default();

        // Test various precision levels
        let test_cases: [(&str, f64, f64, f64, f64); 5] = [
            ("integer", 100.0, 200.0, 50.0, 30.0),
            ("one_decimal", 100.5, 200.5, 50.5, 30.5),
            ("two_decimals", 100.25, 200.75, 50.25, 30.75),
            (
                "many_decimals",
                123.456789012,
                987.654321098,
                45.123456789,
                67.987654321,
            ),
            ("small_values", 0.001, 0.002, 0.5, 0.25),
        ];

        for (name, x, y, w, h) in test_cases {
            let _ = doc.document.nodes.insert(
                NodeId::new(name.to_string()),
                Node {
                    kind: NodeKind::Text,
                    icon: String::new(),
                    label: name.to_string(),
                    x: OrderedFloat(x),
                    y: OrderedFloat(y),
                    width: OrderedFloat(w),
                    height: OrderedFloat(h),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: None,
                    collapsed: None,
                },
            );
        }

        // When
        let json = to_canonical_pretty_json(&doc).expect("serialization should succeed");
        let reloaded: DiagramDocument =
            parse_diagram_document_with_compat(&json).expect("parsing should succeed");

        // Then
        for (name, x, y, w, h) in test_cases {
            let node = reloaded
                .document
                .nodes
                .get(&NodeId::new(name.to_string()))
                .expect("node should exist");
            assert_eq!(node.x.0, x, "{name}: x should be preserved");
            assert_eq!(node.y.0, y, "{name}: y should be preserved");
            assert_eq!(node.width.0, w, "{name}: width should be preserved");
            assert_eq!(node.height.0, h, "{name}: height should be preserved");
        }
    }

    /// IO-TEST-4: Import Large Coordinates No Float Crash (bd-1u1)
    /// Given: A JSON document with very large coordinate values
    /// When: Importing the document
    /// Then: Import succeeds without floating-point overflow/crash
    #[test]
    fn given_document_with_large_coordinates_when_import_then_succeeds() {
        use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;

        // Given - JSON with very large but finite coordinates
        let json = r#"{
            "version": 2,
            "revision": 0,
            "document": {
                "nodes": {
                    "large_coord": {
                        "kind": "text",
                        "icon": "",
                        "label": "Large",
                        "x": 1e15,
                        "y": 1e15,
                        "width": 1000000000000.0,
                        "height": 500000000000.0,
                        "locked": false,
                        "parent": null,
                        "tags": [],
                        "metadata": {}
                    }
                },
                "edges": {}
            },
            "editor_state": {
                "camera_x": 0,
                "camera_y": 0,
                "zoom": 1,
                "grid_size": 20,
                "snap_to_grid": true,
                "selected_items": []
            }
        }"#;

        // When
        let result = parse_diagram_document_with_compat(json);

        // Then - should parse without crash
        assert!(
            result.is_ok(),
            "Large coordinates should parse without crash: {:?}",
            result.err()
        );
        let doc = result.expect("should have document");
        let node = doc
            .document
            .nodes
            .get(&NodeId::new("large_coord".to_string()))
            .expect("node should exist");

        // Verify the large values are preserved
        assert!(node.x.0.is_finite(), "x should be finite");
        assert!(node.y.0.is_finite(), "y should be finite");
        assert!(node.x.0 > 1e14, "x should be very large");
    }

    /// IO-TEST-4b: Extreme but finite values
    #[test]
    fn given_document_with_extreme_finite_coords_when_import_then_succeeds() {
        use crate::ui::toolbar::persistence_compat::parse_diagram_document_with_compat;

        // Given - JSON with values near f64::MAX
        let large_value = 1e300_f64;
        let json = format!(
            r#"{{
            "version": 2,
            "revision": 0,
            "document": {{
                "nodes": {{
                    "extreme": {{
                        "kind": "text",
                        "icon": "",
                        "label": "Extreme",
                        "x": {large_value:e},
                        "y": {large_value:e},
                        "width": 100.0,
                        "height": 50.0,
                        "locked": false,
                        "parent": null,
                        "tags": [],
                        "metadata": {{}}
                    }}
                }},
                "edges": {{}}
            }},
            "editor_state": {{
                "camera_x": 0,
                "camera_y": 0,
                "zoom": 1,
                "grid_size": 20,
                "snap_to_grid": true,
                "selected_items": []
            }}
        }}"#
        );

        // When
        let result = parse_diagram_document_with_compat(&json);

        // Then - should parse without crash
        assert!(
            result.is_ok(),
            "Extreme coordinates should parse: {:?}",
            result.err()
        );
        let doc = result.expect("should have document");
        let node = doc
            .document
            .nodes
            .get(&NodeId::new("extreme".to_string()))
            .expect("node should exist");

        assert!(node.x.0.is_finite(), "x should be finite");
        assert!(node.y.0.is_finite(), "y should be finite");
    }
}
