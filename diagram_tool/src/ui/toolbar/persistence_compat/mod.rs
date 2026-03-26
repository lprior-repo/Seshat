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

/// Compute the replacement `icon_url` value from a `icon_data_url` payload.
/// Returns `None` if no migration is needed (existing `icon_url` wins, or no icon key
/// for base64 conversion).
fn compute_migrated_url(
    icon_data_url: &serde_json::Value,
    icon_key: Option<&str>,
    has_existing_icon_url: bool,
) -> Option<serde_json::Value> {
    if has_existing_icon_url {
        return None;
    }
    if icon_data_url
        .as_str()
        .is_some_and(|s| s.starts_with("data:"))
    {
        icon_key.map(|icon| serde_json::Value::String(format!("/assets/resources/{icon}")))
    } else {
        Some(icon_data_url.clone())
    }
}

/// Migrate `icon_data_url` → `icon_url` in a node object.
/// Handles both metadata-level and top-level node keys.
fn migrate_icon_data_url(
    node_obj: &mut serde_json::Map<String, serde_json::Value>,
    icon_key: Option<&str>,
) {
    // Data: extract whether metadata exists (pure read)
    let has_metadata = node_obj
        .get("metadata")
        .is_some_and(serde_json::Value::is_object);

    // Data: extract icon_data_url from metadata if present, else from node level
    let (icon_data_url, target_has_existing) = if has_metadata {
        let meta = node_obj
            .get("metadata")
            .and_then(|v| v.as_object())
            .is_some_and(|m| m.contains_key("icon_url"));
        let data = node_obj
            .get("metadata")
            .and_then(|v| v.get("icon_data_url"))
            .cloned();
        (data, meta)
    } else {
        let data = node_obj.get("icon_data_url").cloned();
        let has = node_obj.contains_key("icon_url");
        (data, has)
    };

    let Some(url_value) = icon_data_url else {
        return;
    };

    // Calculation: compute replacement (pure)
    let replacement = compute_migrated_url(&url_value, icon_key, target_has_existing);
    let Some(replacement) = replacement else {
        // Existing icon_url wins — just remove old key
        if has_metadata {
            if let Some(meta) = node_obj
                .get_mut("metadata")
                .and_then(serde_json::Value::as_object_mut)
            {
                meta.remove("icon_data_url");
            }
        } else {
            node_obj.remove("icon_data_url");
        }
        return;
    };

    // Action: apply mutation
    if has_metadata {
        if let Some(meta) = node_obj
            .get_mut("metadata")
            .and_then(serde_json::Value::as_object_mut)
        {
            meta.remove("icon_data_url");
            meta.insert("icon_url".to_string(), replacement);
        }
    } else {
        node_obj.remove("icon_data_url");
        node_obj.insert("icon_url".to_string(), replacement);
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
        let icon_key = node_obj
            .get("icon")
            .and_then(|v| v.as_str())
            .map(String::from);
        migrate_icon_data_url(node_obj, icon_key.as_deref());
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
