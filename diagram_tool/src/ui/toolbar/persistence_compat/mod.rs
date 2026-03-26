#![allow(dead_code)]
use diagram_models::document::DiagramDocument;

fn remap_key(obj: &mut serde_json::Map<String, serde_json::Value>, from: &str, to: &str) {
    if obj.contains_key(to) {
        let _ = obj.remove(from);
    } else if let Some(value) = obj.remove(from) {
        let _ = obj.insert(to.to_string(), value);
    }
}

fn normalize_collection(
    document: &mut serde_json::Map<String, serde_json::Value>,
    collection_key: &str,
    mut f: impl FnMut(&mut serde_json::Map<String, serde_json::Value>),
) {
    if let Some(collection) = document
        .get_mut(collection_key)
        .and_then(serde_json::Value::as_object_mut)
    {
        for item in collection.values_mut() {
            if let Some(item_obj) = item.as_object_mut() {
                let _ = item_obj.remove("id");
                f(item_obj);
            }
        }
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

    normalize_collection(document, "nodes", |node_obj| {
        remap_key(node_obj, "font_size", "fontSize");
        remap_key(node_obj, "fontWeight", "font_weight");
        remap_key(node_obj, "dagRank", "dag_rank");
        // Migrate icon_data_url → icon_url, but transform base64 data-URL values
        // into proper HTTP URL paths. Old format: "data:image/png;base64,..."
        // New format: "/assets/resources/{icon_key}"
        // Handle both top-level and metadata-nested icon_data_url
        let icon_key = node_obj
            .get("icon")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        if let Some(meta_obj) = node_obj
            .get_mut("metadata")
            .and_then(serde_json::Value::as_object_mut)
        {
            if let Some(icon_data_url) = meta_obj.remove("icon_data_url") {
                if icon_data_url
                    .as_str()
                    .is_some_and(|s| s.starts_with("data:"))
                {
                    if let Some(ref icon) = icon_key {
                        if !meta_obj.contains_key("icon_url") {
                            let _ = meta_obj.insert(
                                "icon_url".to_string(),
                                serde_json::Value::String(format!("/assets/resources/{icon}")),
                            );
                        }
                    }
                } else if !meta_obj.contains_key("icon_url") {
                    let _ = meta_obj.insert("icon_url".to_string(), icon_data_url);
                }
            }
        } else if let Some(icon_data_url) = node_obj.remove("icon_data_url") {
            if icon_data_url
                .as_str()
                .is_some_and(|s| s.starts_with("data:"))
            {
                if let Some(ref icon) = icon_key {
                    if !node_obj.contains_key("icon_url") {
                        let _ = node_obj.insert(
                            "icon_url".to_string(),
                            serde_json::Value::String(format!("/assets/resources/{icon}")),
                        );
                    }
                }
            } else if !node_obj.contains_key("icon_url") {
                let _ = node_obj.insert("icon_url".to_string(), icon_data_url);
            }
        }
    });

    normalize_collection(document, "edges", |edge_obj| {
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
    });
}

pub fn parse_diagram_document_with_compat(contents: &str) -> Result<DiagramDocument, String> {
    let mut value =
        serde_json::from_str::<serde_json::Value>(contents).map_err(|err| err.to_string())?;
    normalize_compat_shape(&mut value);
    serde_json::from_value::<DiagramDocument>(value).map_err(|err| err.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
