use crate::models::document::DiagramDocument;

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
    use crate::models::document::DiagramDocument;

    #[test]
    fn given_document_when_serialized_then_round_trips() {
        let doc = DiagramDocument::default();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let loaded: DiagramDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc.revision, loaded.revision);
    }

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
}
