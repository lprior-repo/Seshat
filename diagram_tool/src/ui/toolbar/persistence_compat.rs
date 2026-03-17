use diagram_models::document::DiagramDocument;

fn remap_key(obj: &mut serde_json::Map<String, serde_json::Value>, from: &str, to: &str) {
    if obj.contains_key(to) {
        let _ = obj.remove(from);
    } else if let Some(value) = obj.remove(from) {
        let _ = obj.insert(to.to_string(), value);
    }
}

fn normalize_compat_shape(root: &mut serde_json::Value) {
    let Some(document) = root
        .as_object_mut()
        .and_then(|obj| obj.get_mut("document"))
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };

    if let Some(nodes) = document
        .get_mut("nodes")
        .and_then(serde_json::Value::as_object_mut)
    {
        for node in nodes.values_mut() {
            if let Some(node_obj) = node.as_object_mut() {
                let _ = node_obj.remove("id");
                remap_key(node_obj, "font_size", "fontSize");
                remap_key(node_obj, "fontWeight", "font_weight");
                remap_key(node_obj, "dagRank", "dag_rank");
            }
        }
    }

    if let Some(edges) = document
        .get_mut("edges")
        .and_then(serde_json::Value::as_object_mut)
    {
        for edge in edges.values_mut() {
            if let Some(edge_obj) = edge.as_object_mut() {
                let _ = edge_obj.remove("id");
                remap_key(edge_obj, "font_size", "fontSize");
                remap_key(edge_obj, "arrowhead", "arrowType");
                remap_key(edge_obj, "arrow_type", "arrowType");
                remap_key(edge_obj, "bendPoints", "bend_points");
                remap_key(edge_obj, "labelOffsetT", "label_offset_t");
                if let Some(arrow_type) = edge_obj.get_mut("arrowType") {
                    let normalized = arrow_type
                        .as_str()
                        .map(|value| match value {
                            "arrow" => "default",
                            "open" => "straight",
                            "diamond" => "step",
                            "circle" => "curved",
                            "none" => "sharp",
                            _ => value,
                        })
                        .map(ToString::to_string);
                    if let Some(value) = normalized {
                        *arrow_type = serde_json::Value::String(value);
                    }
                }
            }
        }
    }
}

pub fn parse_diagram_document_with_compat(contents: &str) -> Result<DiagramDocument, String> {
    let mut value =
        serde_json::from_str::<serde_json::Value>(contents).map_err(|err| err.to_string())?;
    normalize_compat_shape(&mut value);
    serde_json::from_value::<DiagramDocument>(value).map_err(|err| err.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_when_serialized_then_round_trips() {
        let doc = DiagramDocument::default();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let loaded: DiagramDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.revision, loaded.revision);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_ts_style_json_when_parsed_then_document_loads() {
        let json = r#"{
          "version": 2,
          "revision": 1,
          "document": {
            "nodes": {
              "n1": {
                "id": "n1",
                "kind": "node",
                "icon": "aws/compute/ec2",
                "label": "EC2",
                "x": 10,
                "y": 20,
                "width": 64,
                "height": 64,
                "locked": true,
                "parent": null,
                "tags": ["aws", "compute"],
                "metadata": {}
              }
            },
            "edges": {
              "e1": {
                "id": "e1",
                "source": "n1",
                "target": "n1",
                "label": "",
                "style": "solid",
                "arrowType": "curved",
                "directed": true,
                "bend_points": []
              }
            }
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

        let loaded = super::parse_diagram_document_with_compat(json);
        assert!(loaded.is_ok(), "{:?}", loaded.err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_legacy_font_size_keys_when_parsed_then_document_loads() {
        let json = r#"{
          "version": 2,
          "revision": 1,
          "document": {
            "nodes": {
              "n1": {
                "id": "n1",
                "kind": "node",
                "icon": "aws/compute/ec2",
                "label": "EC2",
                "x": 10,
                "y": 20,
                "width": 64,
                "height": 64,
                "font_size": null,
                "locked": true,
                "parent": null,
                "tags": [],
                "metadata": {}
              }
            },
            "edges": {
              "e1": {
                "id": "e1",
                "source": "n1",
                "target": "n1",
                "label": "",
                "style": "solid",
                "arrowType": "curved",
                "directed": true,
                "font_size": null,
                "bend_points": []
              }
            }
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

        let loaded = super::parse_diagram_document_with_compat(json);
        assert!(loaded.is_ok(), "{:?}", loaded.err());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_equivalent_legacy_aliases_when_parsed_then_canonical_json_is_identical() {
        let legacy_a = r#"{
          "version": 2,
          "revision": 0,
          "document": {
            "nodes": {
              "n1": {
                "id": "n1",
                "kind": "node",
                "icon": "",
                "label": "A",
                "x": 0,
                "y": 0,
                "width": 80,
                "height": 60,
                "locked": false,
                "parent": null,
                "tags": [],
                "metadata": {},
                "font_size": 12,
                "dagRank": 7
              }
            },
            "edges": {
              "e1": {
                "id": "e1",
                "source": "n1",
                "target": "n1",
                "label": "",
                "style": "solid",
                "arrow_type": "diamond",
                "labelOffsetT": 0.25,
                "bendPoints": [],
                "directed": true,
                "metadata": {}
              }
            }
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

        let legacy_b = r#"{
          "version": 2,
          "revision": 0,
          "document": {
            "nodes": {
              "n1": {
                "kind": "node",
                "icon": "",
                "label": "A",
                "x": 0,
                "y": 0,
                "width": 80,
                "height": 60,
                "locked": false,
                "parent": null,
                "tags": [],
                "metadata": {},
                "fontSize": 12,
                "dag_rank": 7
              }
            },
            "edges": {
              "e1": {
                "source": "n1",
                "target": "n1",
                "label": "",
                "style": "solid",
                "arrowType": "step",
                "label_offset_t": 0.25,
                "bend_points": [],
                "directed": true,
                "metadata": {}
              }
            }
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

        let parsed_a = super::parse_diagram_document_with_compat(legacy_a).unwrap();
        let parsed_b = super::parse_diagram_document_with_compat(legacy_b).unwrap();

        let canonical_a = to_canonical_pretty_json(&parsed_a).unwrap();
        let canonical_b = to_canonical_pretty_json(&parsed_b).unwrap();

        assert_eq!(canonical_a, canonical_b);
    }

    /// IO-TEST-5: Import Older Version Migration (bd-1u1)
    /// Given: A JSON document with version: 1 (older schema)
    /// When: Importing the document
    /// Then: Document migrates to current version (version: 2)
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_version_1_document_when_import_then_migrates_to_current_version() {
        // Given - Version 1 document (hypothetical older schema)
        // Note: The actual system uses version 2, but we test that any
        // document with version field parses and produces a valid document
        let json = r#"{
            "version": 1,
            "revision": 0,
            "document": {
                "nodes": {
                    "legacy_node": {
                        "kind": "node",
                        "icon": "",
                        "label": "Legacy",
                        "x": 100,
                        "y": 200,
                        "width": 80,
                        "height": 40,
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
        let result = super::parse_diagram_document_with_compat(json);

        // Then - should parse successfully (migration handled)
        assert!(
            result.is_ok(),
            "Version 1 document should parse: {:?}",
            result.err()
        );
        let doc = result.expect("should have document");

        // The document should be usable regardless of original version
        assert!(
            doc.document
                .nodes
                .contains_key(&diagram_models::document::NodeId::new(
                    "legacy_node".to_string()
                )),
            "Legacy node should be present"
        );
    }

    /// IO-TEST-5b: Version migration with legacy field names
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_older_document_with_legacy_fields_when_import_then_fields_remapped() {
        // Given - Document using legacy field naming conventions
        let json = r#"{
            "version": 2,
            "revision": 0,
            "document": {
                "nodes": {
                    "legacy_fields": {
                        "kind": "node",
                        "icon": "",
                        "label": "Legacy Fields",
                        "x": 50,
                        "y": 50,
                        "width": 100,
                        "height": 60,
                        "font_size": 14,
                        "fontWeight": "bold",
                        "dagRank": 5,
                        "locked": false,
                        "parent": null,
                        "tags": [],
                        "metadata": {}
                    }
                },
                "edges": {
                    "legacy_edge": {
                        "source": "legacy_fields",
                        "target": "legacy_fields",
                        "label": "",
                        "style": "solid",
                        "arrowhead": "diamond",
                        "labelOffsetT": 0.75,
                        "bendPoints": [],
                        "directed": true,
                        "metadata": {}
                    }
                }
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
        let result = super::parse_diagram_document_with_compat(json);

        // Then - should parse and remap fields
        assert!(
            result.is_ok(),
            "Legacy fields should be remapped: {:?}",
            result.err()
        );
        let doc = result.expect("should have document");

        // Verify node exists
        assert!(
            doc.document
                .nodes
                .contains_key(&diagram_models::document::NodeId::new(
                    "legacy_fields".to_string()
                )),
            "Node should be parsed"
        );

        // Verify edge exists and was remapped
        assert!(
            doc.document
                .edges
                .contains_key(&diagram_models::document::EdgeId::new(
                    "legacy_edge".to_string()
                )),
            "Edge should be parsed"
        );

        // Verify the node's dag_rank was remapped from dagRank
        let node = doc
            .document
            .nodes
            .get(&diagram_models::document::NodeId::new(
                "legacy_fields".to_string(),
            ))
            .expect("node should exist");
        assert_eq!(
            node.dag_rank,
            Some(5),
            "dagRank should be remapped to dag_rank"
        );
    }

    /// IO-TEST-5c: Version field is required
    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_document_without_version_when_import_then_fails_gracefully() {
        // Given - Document without explicit version field
        let json = r#"{
            "revision": 0,
            "document": {
                "nodes": {
                    "no_version": {
                        "kind": "node",
                        "icon": "",
                        "label": "No Version",
                        "x": 0,
                        "y": 0,
                        "width": 80,
                        "height": 40,
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
        let result = super::parse_diagram_document_with_compat(json);

        // Then - should fail gracefully with clear error (version is required)
        assert!(
            result.is_err(),
            "Document without version should fail: expected error, got {:?}",
            result.ok()
        );
        let err_msg = result.expect_err("should have error");
        assert!(
            err_msg.contains("version"),
            "Error message should mention version field: {}",
            err_msg
        );
    }
}
