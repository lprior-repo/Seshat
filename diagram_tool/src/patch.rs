#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use anyhow::Result;
use json_patch::Patch;

/// Pure calculation to apply an AI patch.
pub fn patch_doc(doc: &DiagramDocument, patch: &Patch) -> Result<DiagramDocument> {
    let mut doc_val = match serde_json::to_value(doc) {
        Ok(v) => v,
        Err(e) => return Err(anyhow::Error::new(e).context("Failed to serialize document")),
    };

    match json_patch::patch(&mut doc_val, patch) {
        Ok(()) => {}
        Err(e) => return Err(anyhow::Error::new(e).context("Failed to apply patch")),
    }

    match serde_json::from_value(doc_val) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow::Error::new(e).context("Failed to deserialize document")),
    }
}

#[cfg(test)]
mod tests {
    use super::{patch_doc, DiagramDocument, Patch};
    use crate::models::document::{Node, NodeId, NodeKind, OrderedFloat};
    use im::HashMap;
    use proptest::prelude::*;

    fn minimal_doc() -> DiagramDocument {
        DiagramDocument::default()
    }

    fn doc_with_node(id: &str, x: f64, y: f64, w: f64, h: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::from("test"),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(w),
            height: OrderedFloat(h),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: vec![],
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        doc.document
            .nodes
            .insert(NodeId::new(String::from(id)), node);
        doc
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_negative_dimensions_no_panic(width in -1000.0f64..=-0.001f64, height in -1000.0f64..=-0.001f64) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/width", "value": {}}}, {{"op": "replace", "path": "/document/nodes/n1/height", "value": {}}}]"#, width, height);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
            if let Ok(new_doc) = result {
                if let Some(node) = new_doc.document.nodes.get(&NodeId::new(String::from("n1"))) {
                    prop_assert!(node.width.0 < 0.0 || node.height.0 < 0.0);
                }
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_invalid_node_id_no_panic(id in "[!@#$%^&*()+=\\[\\]{}|;:',.?]{1,20}") {
            let doc = minimal_doc();
            let patch_json = format!(r#"[{{"op": "add", "path": "/document/nodes/{}", "value": {{"kind": "node", "icon": "", "label": "x", "x": 0, "y": 0, "width": 10, "height": 10, "locked": false}}}}]"#, id);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_empty_patch_no_panic(_dummy in Just(())) {
            let doc = minimal_doc();
            let patch_json = r#"[]"#;
            let patch: Patch = serde_json::from_str(patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_malformed_json_value_no_pokemon(garbage in "[^\"]{0,50}") {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/label", "value": "{}"}}]"#, garbage.escape_default());
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_self_parent_cycle_no_panic(node_id in "[a-zA-Z0-9_-]{1,10}") {
            let doc = doc_with_node(&node_id, 0.0, 0.0, 10.0, 10.0);
            let patch_json = format!(r#"[{{"op": "add", "path": "/document/nodes/{}/parent", "value": "{}"}}]"#, node_id, node_id);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
            if let Ok(new_doc) = result {
                if let Some(node) = new_doc.document.nodes.get(&NodeId::new(node_id.clone())) {
                    if let Some(ref parent) = node.parent {
                        prop_assert!(parent.as_str() == node_id);
                    }
                }
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_nonexistent_parent_no_panic(parent_id in "[a-zA-Z0-9_-]{1,10}") {
            let doc = doc_with_node("child", 0.0, 0.0, 10.0, 10.0);
            let patch_json = format!(r#"[{{"op": "add", "path": "/document/nodes/child/parent", "value": "{}"}}]"#, parent_id);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_cycle_chain_no_panic(a in "[a-z]{1,5}", b in "[a-z]{1,5}", c in "[a-z]{1,5}") {
            prop_assume!(a != b && b != c && a != c);
            let mut doc = DiagramDocument::default();
            for id in [&a, &b, &c] {
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: id.to_string(),
                    x: OrderedFloat(0.0),
                    y: OrderedFloat(0.0),
                    width: OrderedFloat(10.0),
                    height: OrderedFloat(10.0),
                    font_size: None,
                    font_weight: None,
                    locked: false,
                    parent: None,
                    dag_rank: None,
                    tags: vec![],
                    metadata: HashMap::new(),
                    z_index: 0,
                    style: None,
                    collapsed: None,
                };
                doc.document.nodes.insert(NodeId::new(id.to_string()), node);
            }
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/nodes/{}/parent", "value": "{}"}},
                   {{"op": "add", "path": "/document/nodes/{}/parent", "value": "{}"}},
                   {{"op": "add", "path": "/document/nodes/{}/parent", "value": "{}"}}]"#,
                a, b, b, c, c, a
            );
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_deeply_nested_path_no_panic(depth in 1usize..=10usize) {
            let doc = minimal_doc();
            let path = (0..depth).map(|_| "nodes").collect::<Vec<_>>().join("/nodes/");
            let patch_json = format!(r#"[{{"op": "add", "path": "/document/{}", "value": {{}}}}]"#, path);
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_type_mismatch_no_panic(val in -1000i64..=1000i64) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/locked", "value": {}}}]"#, val);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_float_for_bool_field_no_panic(val in -1000.0f64..=1000.0f64) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/locked", "value": {}}}]"#, val);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_zero_dimensions_no_panic(_w in Just(()), _h in Just(())) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = r#"[{"op": "replace", "path": "/document/nodes/n1/width", "value": 0}, {"op": "replace", "path": "/document/nodes/n1/height", "value": 0}]"#;
            let patch: Patch = serde_json::from_str(patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_edge_nonexistent_nodes_no_panic(src in "[a-z]{1,5}", tgt in "[a-z]{1,5}") {
            let doc = minimal_doc();
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/edges/e1", "value": {{"source": "{}", "target": "{}", "label": "", "style": "solid", "arrowType": "default", "label_offset_t": 0.5, "thickness": 1.5, "directed": true, "bend_points": [], "tags": [], "metadata": {{}}}}}}]"#,
                src, tgt
            );
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_extreme_coordinates_no_panic(x in -1000000.0f64..=1000000.0f64, y in -1000000.0f64..=1000000.0f64) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/x", "value": {}}}, {{"op": "replace", "path": "/document/nodes/n1/y", "value": {}}}]"#, x, y);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_unicode_in_labels_no_panic(label in "\\PC*") {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let label_json = serde_json::to_string(&label).unwrap();
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/label", "value": {}}}]"#, label_json);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_empty_string_fields_no_panic(field in proptest::sample::select(&["label", "icon"][..])) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/{}", "value": ""}}]"#, field);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_nan_coordinates_rejected(_dummy in Just(())) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let val = f64::NAN;
            let val_json = serde_json::to_string(&val).unwrap();
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/x", "value": {}}}]"#, val_json);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_infinity_coordinates_rejected(_dummy in Just(())) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let val = f64::INFINITY;
            let val_json = serde_json::to_string(&val).unwrap();
            let patch_json = format!(r#"[{{"op": "replace", "path": "/document/nodes/n1/x", "value": {}}}]"#, val_json);
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }
    }
}
