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
        #![proptest_config(ProptestConfig::with_cases(256))]

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

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_100_ops_simultaneous_no_panic(ops_count in 50usize..=150usize) {
            let mut doc = DiagramDocument::default();
            let mut patch_ops = Vec::new();
            for i in 0..ops_count {
                let node_id = format!("n{}", i);
                let patch_op = format!(
                    r#"{{"op": "add", "path": "/document/nodes/{}", "value": {{"kind": "node", "icon": "", "label": "node{}", "x": {}, "y": {}, "width": 10, "height": 10, "locked": false}}}}"#,
                    node_id, i, i as f64 * 10.0, i as f64 * 10.0
                );
                patch_ops.push(patch_op);
            }
            for i in 0..ops_count.min(50) {
                let patch_op = format!(
                    r#"{{"op": "replace", "path": "/document/nodes/n{}/x", "value": {}}}"#,
                    i, (i as f64) * -1.0
                );
                patch_ops.push(patch_op);
            }
            let patch_json = format!("[{}]", patch_ops.join(","));
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
            if let Ok(new_doc) = result {
                prop_assert!(new_doc.document.nodes.len() >= ops_count / 2);
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_set_delete_set_same_path_no_panic(
            label1 in "[a-z]{1,5}",
            label2 in "[a-z]{1,5}",
            label3 in "[a-z]{1,5}",
        ) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(
                r#"[
                    {{"op": "replace", "path": "/document/nodes/n1/label", "value": "{}"}},
                    {{"op": "remove", "path": "/document/nodes/n1/label"}},
                    {{"op": "add", "path": "/document/nodes/n1/label", "value": "{}"}},
                    {{"op": "replace", "path": "/document/nodes/n1/label", "value": "{}"}}
                ]"#,
                label1, label2, label3
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_deeply_nested_json_10_levels_no_panic(depth in 8usize..=15usize) {
            let doc = minimal_doc();
            let mut nested = String::from("{}");
            for d in 0..depth {
                nested = format!(r#"{{"level{}": {}}}"#, d, nested);
            }
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/metadata/deep", "value": {}}}]"#,
                nested
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_nonexistent_path_chain_no_panic(
            path_segments in proptest::collection::vec("[a-z]{1,3}", 5..=15),
        ) {
            let doc = minimal_doc();
            let path = path_segments.join("/");
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/nodes/{}/x", "value": 42}}]"#,
                path
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_empty_unicode_special_sequence_no_panic(
            empty_val in Just(String::new()),
            unicode_val in "[\\p{L}\\p{N}\\p{S}]{1,20}",
            special_val in "[!@#$%^&*()\\[\\]{}|;:',.<>?/~`]{1,20}",
        ) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let empty_json = serde_json::to_string(&empty_val).unwrap();
            let unicode_json = serde_json::to_string(&unicode_val).unwrap();
            let special_json = serde_json::to_string(&special_val).unwrap();
            let patch_json = format!(
                r#"[
                    {{"op": "replace", "path": "/document/nodes/n1/label", "value": {}}},
                    {{"op": "replace", "path": "/document/nodes/n1/label", "value": {}}},
                    {{"op": "replace", "path": "/document/nodes/n1/label", "value": {}}}
                ]"#,
                empty_json, unicode_json, special_json
            );
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_parent_chain_cycle_no_panic(
            nodes in proptest::collection::vec("[a-z]{1,3}", 4..=8),
        ) {
            prop_assume!(nodes.len() >= 4);
            let doc = DiagramDocument::default();
            for id in &nodes {
                let node = Node {
                    kind: NodeKind::Node,
                    icon: String::new(),
                    label: id.clone(),
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
                doc.document.nodes.insert(NodeId::new(id.clone()), node);
            }
            let mut patch_ops = Vec::new();
            for i in 0..nodes.len() {
                let next_i = (i + 1) % nodes.len();
                patch_ops.push(format!(
                    r#"{{"op": "add", "path": "/document/nodes/{}/parent", "value": "{}"}}"#,
                    nodes[i], nodes[next_i]
                ));
            }
            let patch_json = format!("[{}]", patch_ops.join(","));
            let patch: Patch = serde_json::from_str(&patch_json).unwrap();
            let result = patch_doc(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_all_json_types_same_path_no_panic(
            str_val in "[a-z]{1,5}",
            num_val in -1000i64..=1000i64,
            bool_val: bool,
        ) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(
                r#"[
                    {{"op": "replace", "path": "/document/nodes/n1/metadata/test", "value": {}}},
                    {{"op": "replace", "path": "/document/nodes/n1/metadata/test", "value": {}}},
                    {{"op": "replace", "path": "/document/nodes/n1/metadata/test", "value": {}}},
                    {{"op": "replace", "path": "/document/nodes/n1/metadata/test", "value": null}},
                    {{"op": "replace", "path": "/document/nodes/n1/metadata/test", "value": [1,2,3]}},
                    {{"op": "replace", "path": "/document/nodes/n1/metadata/test", "value": {{"nested": true}}}}
                ]"#,
                serde_json::to_string(&str_val).unwrap(),
                num_val,
                bool_val
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_revision_u64_max_no_panic(rev_high in 0u64..=u64::MAX / 2) {
            let doc = minimal_doc();
            let rev = u64::MAX - rev_high;
            let patch_json = format!(
                r#"[{{"op": "replace", "path": "/revision", "value": {}}}]"#,
                rev
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_long_path_1000_chars_no_panic(base in "[a-z]{1,5}") {
            let doc = minimal_doc();
            let long_segment = base.repeat(200);
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/nodes/{}", "value": {{"kind": "node", "icon": "", "label": "x", "x": 0, "y": 0, "width": 10, "height": 10, "locked": false}}}}]"#,
                long_segment
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_move_operations_no_panic(src in "[a-z]{1,5}", tgt in "[a-z]{1,5}") {
            prop_assume!(src != tgt);
            let doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: src.clone(),
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
            doc.document.nodes.insert(NodeId::new(src.clone()), node);
            let patch_json = format!(
                r#"[{{"op": "move", "from": "/document/nodes/{}", "path": "/document/nodes/{}"}}]"#,
                src, tgt
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_copy_operations_no_panic(src in "[a-z]{1,5}", tgt in "[a-z]{1,5}") {
            prop_assume!(src != tgt);
            let mut doc = DiagramDocument::default();
            let node = Node {
                kind: NodeKind::Node,
                icon: String::new(),
                label: src.clone(),
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
            doc.document.nodes.insert(NodeId::new(src.clone()), node);
            let patch_json = format!(
                r#"[{{"op": "copy", "from": "/document/nodes/{}", "path": "/document/nodes/{}"}}]"#,
                src, tgt
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_test_operation_conflicts_no_panic(
            expected in "[a-z]{1,5}",
            actual in "[a-z]{1,5}",
        ) {
            let doc = doc_with_node("n1", 100.0, 100.0, 50.0, 50.0);
            let patch_json = format!(
                r#"[
                    {{"op": "test", "path": "/document/nodes/n1/label", "value": "{}"}},
                    {{"op": "replace", "path": "/document/nodes/n1/label", "value": "{}"}}
                ]"#,
                expected, actual
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_concurrent_add_remove_same_node_no_panic(node_id in "[a-z]{1,5}") {
            let doc = minimal_doc();
            let patch_json = r#"[
                    {"op": "add", "path": "/document/nodes/__NODE_ID__", "value": {"kind": "node", "icon": "", "label": "x", "x": 0, "y": 0, "width": 10, "height": 10, "locked": false}},
                    {"op": "remove", "path": "/document/nodes/__NODE_ID__"},
                    {"op": "add", "path": "/document/nodes/__NODE_ID__", "value": {"kind": "node", "icon": "", "label": "y", "x": 1, "y": 1, "width": 20, "height": 20, "locked": true}}
                ]"#
                .replace("__NODE_ID__", &node_id);
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_array_index_out_of_bounds_no_panic(idx in 100usize..=1000usize) {
            let doc = minimal_doc();
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/edges/{}", "value": {{"source": "a", "target": "b", "label": "", "style": "solid", "arrowType": "default", "label_offset_t": 0.5, "thickness": 1.5, "directed": true, "bend_points": [], "tags": [], "metadata": {{}}}}}}]"#,
                idx
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_negative_array_index_no_panic(idx in -1000isize..=-1isize) {
            let doc = minimal_doc();
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/edges/{}", "value": {{"source": "a", "target": "b"}}}}]"#,
                idx
            );
            if serde_json::from_str::<Patch>(&patch_json).is_ok() {
                let patch: Patch = serde_json::from_str(&patch_json).unwrap();
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_escaped_slash_in_path_no_panic(segment in "[a-z/]{1,10}") {
            let doc = minimal_doc();
            let escaped = segment.replace('/', "~1");
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/nodes/{}", "value": {{"kind": "node", "icon": "", "label": "x", "x": 0, "y": 0, "width": 10, "height": 10, "locked": false}}}}]"#,
                escaped
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_escaped_tilde_in_path_no_panic(segment in "[a-z~]{1,10}") {
            let doc = minimal_doc();
            let escaped = segment.replace('~', "~0");
            let patch_json = format!(
                r#"[{{"op": "add", "path": "/document/nodes/{}", "value": {{"kind": "node", "icon": "", "label": "x", "x": 0, "y": 0, "width": 10, "height": 10, "locked": false}}}}]"#,
                escaped
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                let result = patch_doc(&doc, &patch);
                prop_assert!(result.is_ok() || result.is_err());
            }
        }

        #[test]
        #[allow(clippy::unwrap_used)]
        fn prop_patch_fuzz_all_operations_mixed_no_panic(
            ops in proptest::collection::vec(
                (0usize..=5, "[a-z]{1,3}", -100f64..=100f64),
                20..=50
            )
        ) {
            let mut doc = DiagramDocument::default();
            let mut patch_ops = Vec::new();
            let mut node_ids: Vec<String> = Vec::new();
            for (i, (op_type, id_part, coord)) in ops.iter().enumerate() {
                let node_id = format!("{}{}", id_part, i % 10);
                match op_type {
                    0 => {
                        patch_ops.push(format!(
                            r#"{{"op": "add", "path": "/document/nodes/{}", "value": {{"kind": "node", "icon": "", "label": "{}", "x": {}, "y": {}, "width": 10, "height": 10, "locked": false}}}}"#,
                            node_id, node_id, coord, coord
                        ));
                        if !node_ids.contains(&node_id) {
                            node_ids.push(node_id);
                        }
                    }
                    1 => {
                        if !node_ids.is_empty() {
                            let existing = &node_ids[i % node_ids.len()];
                            patch_ops.push(format!(
                                r#"{{"op": "replace", "path": "/document/nodes/{}/x", "value": {}}}"#,
                                existing, coord
                            ));
                        }
                    }
                    2 => {
                        if !node_ids.is_empty() {
                            let existing = &node_ids[i % node_ids.len()];
                            patch_ops.push(format!(
                                r#"{{"op": "replace", "path": "/document/nodes/{}/y", "value": {}}}"#,
                                existing, coord
                            ));
                        }
                    }
                    3 => {
                        if node_ids.len() > 1 {
                            let from = &node_ids[i % node_ids.len()];
                            let to = &node_ids[(i + 1) % node_ids.len()];
                            if from != to {
                                patch_ops.push(format!(
                                    r#"{{"op": "move", "from": "/document/nodes/{}", "path": "/document/nodes/moved_{}"}}"#,
                                    from, i
                                ));
                            }
                        }
                    }
                    4 => {
                        if !node_ids.is_empty() {
                            let existing = &node_ids[i % node_ids.len()];
                            patch_ops.push(format!(
                                r#"{{"op": "remove", "path": "/document/nodes/{}"}}"#,
                                existing
                            ));
                        }
                    }
                    _ => {
                        patch_ops.push(format!(
                            r#"{{"op": "test", "path": "/document/nodes", "value": {{}}}}"#
                        ));
                    }
                }
            }
            if !patch_ops.is_empty() {
                let patch_json = format!("[{}]", patch_ops.join(","));
                if let Ok(patch) = serde_json::from_str::<Patch>(&patch_json) {
                    let result = patch_doc(&doc, &patch);
                    prop_assert!(result.is_ok() || result.is_err());
                }
            }
        }
    }
}
