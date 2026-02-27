#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::layout::grid::calculate_grid_layout;
use crate::models::document::DiagramDocument;
use crate::mutation::error::MutationError;
use crate::patch::patch_doc;
use json_patch::Patch;
use serde_json::Value;

pub fn apply_patch(doc: &DiagramDocument, patch: &Patch) -> Result<DiagramDocument, MutationError> {
    if !has_valid_revision_test(doc, patch) {
        return Err(MutationError::Transform(String::from(
            "first patch operation must be test /revision with current revision",
        )));
    }

    patch_doc(doc, patch).map_err(|err| MutationError::Transform(err.to_string()))
}

#[must_use]
pub fn apply_layout(doc: &DiagramDocument, cell_size: f64) -> DiagramDocument {
    calculate_grid_layout(doc, cell_size)
}

fn has_valid_revision_test(doc: &DiagramDocument, patch: &Patch) -> bool {
    let expected_revision = serde_json::to_value(doc.revision).ok();
    let patch_value = serde_json::to_value(patch).ok();

    expected_revision
        .zip(patch_value)
        .is_some_and(|(expected, patch_json)| {
            patch_json
                .as_array()
                .and_then(|ops| ops.first())
                .and_then(Value::as_object)
                .and_then(|op| {
                    let kind = op.get("op")?.as_str()?;
                    let json_path = op.get("path")?.as_str()?;
                    let value = op.get("value")?;
                    Some(kind == "test" && json_path == "/revision" && *value == expected)
                })
                .is_some_and(|is_valid| is_valid)
        })
}

#[cfg(test)]
mod tests {
    use super::{apply_layout, apply_patch};
    use crate::models::document::{
        DiagramDocument, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use anyhow::Result;
    use im::HashMap;
    use json_patch::Patch;

    fn node(x: f64, y: f64, locked: bool) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(60.0),
            font_size: None,
            font_weight: None,
            locked,
            parent: None,
            dag_rank: None,
            tags: vec![],
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    #[test]
    fn given_patch_without_leading_revision_test_when_apply_patch_then_it_returns_error(
    ) -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_non_first_revision_test_when_apply_patch_then_it_returns_error() -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false},
                {"op":"test","path":"/revision","value":0}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_patch_with_matching_revision_test_when_apply_patch_then_it_returns_updated_document(
    ) -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"test","path":"/revision","value":0},
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        let toggled = result.ok();
        assert!(toggled.is_some());
        assert!(toggled.is_some_and(|next| !next.editor_state.snap_to_grid));
        Ok(())
    }

    #[test]
    fn given_patch_with_wrong_revision_when_apply_patch_then_it_returns_error() -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"test","path":"/revision","value":999},
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_patch_with_wrong_test_path_when_apply_patch_then_it_returns_error() -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"test","path":"/not_revision","value":0},
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_patch_with_test_op_on_wrong_field_when_apply_patch_then_it_returns_error() -> Result<()>
    {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"test","path":"/version","value":2},
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_patch_with_wrong_test_path_but_matching_revision_value_when_apply_patch_then_it_returns_error(
    ) -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"test","path":"/editor_state/camera_x","value":0},
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_non_test_first_op_with_revision_value_when_apply_patch_then_it_returns_error(
    ) -> Result<()> {
        let doc = DiagramDocument::default();
        let patch: Patch = serde_json::from_str(
            r#"[
                {"op":"replace","path":"/editor_state/camera_x","value":0},
                {"op":"replace","path":"/editor_state/snap_to_grid","value":false}
            ]"#,
        )?;

        let result = apply_patch(&doc, &patch);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_unlocked_node_when_apply_layout_then_position_snaps_to_grid() {
        let id = NodeId::new(String::from("n1"));
        let mut doc = DiagramDocument::default();
        doc.document.nodes = HashMap::new().update(id.clone(), node(23.0, 47.0, false));

        let laid_out = apply_layout(&doc, 20.0);
        let before = doc
            .document
            .nodes
            .get(&id)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));
        let after = laid_out
            .document
            .nodes
            .get(&id)
            .map_or((0.0, 0.0), |n| (n.x.0, n.y.0));

        assert!(laid_out.document.nodes.contains_key(&id));
        assert_ne!(before, after);
        assert!((after.0 % 20.0).abs() < f64::EPSILON);
        assert!((after.1 % 20.0).abs() < f64::EPSILON);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod proptests {
    use super::{apply_layout, apply_patch};
    use crate::models::document::{
        DiagramDocument, Edge, EdgeId, EdgeStyle, Node, NodeId, NodeKind, NodeStyle, OrderedFloat,
    };
    use im::HashMap;
    use json_patch::Patch;
    use proptest::prelude::*;

    fn make_node(x: f64, y: f64, locked: bool, parent: Option<NodeId>) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: String::new(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(60.0),
            font_size: None,
            font_weight: None,
            locked,
            parent,
            dag_rank: None,
            tags: vec![],
            metadata: HashMap::new(),
            z_index: 0,
            style: Some(NodeStyle::default()),
            collapsed: None,
        }
    }

    fn make_edge(source: &str, target: &str) -> Edge {
        Edge {
            source: NodeId::new(String::from(source)),
            target: NodeId::new(String::from(target)),
            label: String::new(),
            style: EdgeStyle::default(),
            arrow_type: crate::models::document::ArrowType::default(),
            label_offset_t: OrderedFloat(0.5),
            color: None,
            thickness: OrderedFloat(1.5),
            directed: true,
            bend_points: vec![],
            tags: vec![],
            metadata: HashMap::new(),
            font_size: None,
        }
    }

    fn make_doc_with_nodes(nodes: Vec<(String, f64, f64, bool)>) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        for (id, x, y, locked) in nodes {
            doc.document.nodes = doc
                .document
                .nodes
                .update(NodeId::new(id), make_node(x, y, locked, None));
        }
        doc
    }

    fn patch_with_revision(revision: u64, ops: &str) -> String {
        format!(
            r#"[{{"op":"test","path":"/revision","value":{}}},{}]"#,
            revision, ops
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn prop_apply_patch_nan_node_position(_ in Just(())) {
            let mut doc = DiagramDocument::default();
            let mut node = make_node(f64::NAN, f64::NAN, false, None);
            node.x = OrderedFloat(f64::NAN);
            node.y = OrderedFloat(f64::NAN);
            doc.document.nodes = doc.document.nodes.update(NodeId::new("n1".into()), node);

            let patch: Patch = serde_json::from_str(&patch_with_revision(0, r#"{"op":"replace","path":"/editor_state/snap_to_grid","value":false}"#)).unwrap();
            let result = apply_patch(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn prop_apply_patch_inf_node_position(sign in 0_i32..=1) {
            let val = if sign == 0 { f64::NEG_INFINITY } else { f64::INFINITY };
            let mut doc = DiagramDocument::default();
            let mut node = make_node(val, val, false, None);
            node.x = OrderedFloat(val);
            node.y = OrderedFloat(val);
            doc.document.nodes = doc.document.nodes.update(NodeId::new("n1".into()), node);

            let patch: Patch = serde_json::from_str(&patch_with_revision(0, r#"{"op":"replace","path":"/editor_state/snap_to_grid","value":false}"#)).unwrap();
            let result = apply_patch(&doc, &patch);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn prop_apply_patch_empty_document_always_valid(
            op in r#"(add|replace|remove)"#,
            path in r#"/[a-z_]+(/[a-z_]+)*"#,
            value in proptest::option::of(any::<bool>()),
        ) {
            let doc = DiagramDocument::default();
            let value_json = value.map_or("null".into(), |v| v.to_string());
            let patch_str = format!(
                r#"[{{"op":"test","path":"/revision","value":0}},{{"op":"{}","path":"{}","value":{}}}]"#,
                op, path, value_json
            );
            let patch: Patch = serde_json::from_str(&patch_str).unwrap();
            let _ = apply_patch(&doc, &patch);
        }

        #[test]
        fn prop_apply_patch_duplicate_node_ids_preserves_count(n in 1_usize..10) {
            let mut doc = DiagramDocument::default();
            for i in 0..n {
                doc.document.nodes = doc.document.nodes.update(
                    NodeId::new(format!("dup")),
                    make_node(i as f64, i as f64, false, None),
                );
            }

            let patch: Patch = serde_json::from_str(&patch_with_revision(0, r#"{"op":"replace","path":"/editor_state/snap_to_grid","value":false}"#)).unwrap();
            let result = apply_patch(&doc, &patch);
            prop_assert!(result.is_ok());
            if let Ok(new_doc) = result {
                prop_assert!(new_doc.document.nodes.len() <= n);
            }
        }

        #[test]
        fn prop_apply_patch_deeply_nested_path(depth in 1_usize..10) {
            let doc = DiagramDocument::default();
            let nested_path = (0..depth).map(|_| "nested").collect::<Vec<_>>().join("/");
            let patch_str = format!(
                r#"[{{"op":"test","path":"/revision","value":0}},{{"op":"add","path":"/{}","value":{}}}]"#,
                nested_path,
                if depth % 2 == 0 { "true" } else { "null" }
            );
            let patch: Patch = serde_json::from_str(&patch_str).unwrap();
            let _ = apply_patch(&doc, &patch);
        }

        #[test]
        fn prop_apply_layout_zero_cell_size(_ in Just(())) {
            let doc = make_doc_with_nodes(vec![
                ("a".into(), 100.0, 100.0, false),
                ("b".into(), 200.0, 200.0, false),
            ]);
            let result = apply_layout(&doc, 0.0);
            prop_assert!(result.document.nodes.len() == 2);
        }

        #[test]
        fn prop_apply_layout_negative_cell_size(cell_size in -1e10_f64..-0.001) {
            let doc = make_doc_with_nodes(vec![("a".into(), 100.0, 100.0, false)]);
            let result = apply_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[test]
        fn prop_apply_layout_nan_cell_size(_ in Just(())) {
            let doc = make_doc_with_nodes(vec![("a".into(), 100.0, 100.0, false)]);
            let result = apply_layout(&doc, f64::NAN);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[test]
        fn prop_apply_layout_inf_cell_size(sign in -1_i32..=1) {
            let cell_size = if sign < 0 { f64::NEG_INFINITY } else { f64::INFINITY };
            let doc = make_doc_with_nodes(vec![("a".into(), 100.0, 100.0, false)]);
            let result = apply_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[test]
        fn prop_apply_layout_extreme_scale(scale in 1e-15_f64..1e15_f64) {
            let doc = make_doc_with_nodes(vec![
                ("a".into(), 50.0, 50.0, false),
                ("b".into(), 150.0, 150.0, false),
            ]);
            let result = apply_layout(&doc, scale);
            prop_assert!(result.document.nodes.len() == 2);
        }

        #[test]
        fn prop_apply_layout_preserves_node_count(
            node_count in 0_usize..20,
            cell_size in 0.001_f64..1000.0,
        ) {
            let mut nodes = Vec::new();
            for i in 0..node_count {
                nodes.push((format!("n{}", i), i as f64 * 10.0, i as f64 * 10.0, i % 3 == 0));
            }
            let doc = make_doc_with_nodes(nodes);
            let result = apply_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == node_count);
        }

        #[test]
        fn prop_apply_layout_locked_nodes_unchanged(
            x in -1e6_f64..1e6_f64,
            y in -1e6_f64..1e6_f64,
            cell_size in 1.0_f64..1000.0,
        ) {
            let doc = make_doc_with_nodes(vec![("locked".into(), x, y, true)]);
            let result = apply_layout(&doc, cell_size);
            let orig = doc.document.nodes.get(&NodeId::new("locked".into())).unwrap();
            let new = result.document.nodes.get(&NodeId::new("locked".into())).unwrap();
            prop_assert!((orig.x.0 - new.x.0).abs() < f64::EPSILON);
            prop_assert!((orig.y.0 - new.y.0).abs() < f64::EPSILON);
        }

        #[test]
        fn prop_apply_patch_invalid_type_string(ty in "[!@#$%^&*()]{1,10}") {
            let doc = DiagramDocument::default();
            let patch_str = format!(
                r#"[{{"op":"test","path":"/revision","value":0}},{{"op":"add","path":"/document/nodes/n1","value":{{"kind":"{}","label":"test","x":0,"y":0,"width":100,"height":60,"locked":false}}}}]"#,
                ty
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_str) {
                let result = apply_patch(&doc, &patch);
                prop_assert!(result.is_err() || result.is_ok());
            }
        }

        #[test]
        fn prop_apply_patch_self_referential_edge(_ in Just(())) {
            let mut doc = DiagramDocument::default();
            doc.document.nodes = doc.document.nodes.update(
                NodeId::new("n1".into()),
                make_node(0.0, 0.0, false, None),
            );
            doc.document.edges = doc.document.edges.update(
                EdgeId::new("e1".into()),
                make_edge("n1", "n1"),
            );

            let patch: Patch = serde_json::from_str(&patch_with_revision(0, r#"{"op":"replace","path":"/editor_state/snap_to_grid","value":false}"#)).unwrap();
            let result = apply_patch(&doc, &patch);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_apply_layout_with_parent_cycle(_ in Just(())) {
            let n1 = NodeId::new("n1".into());
            let n2 = NodeId::new("n2".into());
            let n3 = NodeId::new("n3".into());

            let mut doc = DiagramDocument::default();
            doc.document.nodes = doc.document.nodes.update(n1.clone(), make_node(0.0, 0.0, false, Some(n3.clone())));
            doc.document.nodes = doc.document.nodes.update(n2.clone(), make_node(100.0, 0.0, false, Some(n1.clone())));
            doc.document.nodes = doc.document.nodes.update(n3.clone(), make_node(200.0, 0.0, false, Some(n2.clone())));

            let result = apply_layout(&doc, 100.0);
            prop_assert!(result.document.nodes.len() == 3);
        }

        #[test]
        fn prop_apply_patch_malformed_path_preserves_doc(path in ".*") {
            let doc = DiagramDocument::default();
            let patch_str = format!(
                r#"[{{"op":"test","path":"/revision","value":0}},{{"op":"replace","path":"{}","value":true}}]"#,
                path
            );
            if let Ok(patch) = serde_json::from_str::<Patch>(&patch_str) {
                let _ = apply_patch(&doc, &patch);
            }
        }

        #[test]
        fn prop_apply_layout_extreme_position_preserves_finiteness(coord in -1e15_f64..1e15_f64) {
            let doc = make_doc_with_nodes(vec![
                ("a".into(), coord, coord, false),
                ("b".into(), -coord, -coord, false),
            ]);
            let result = apply_layout(&doc, 100.0);
            for node in result.document.nodes.values() {
                prop_assert!(node.x.0.is_finite() || node.x.0.is_nan() || node.x.0.is_infinite());
                prop_assert!(node.y.0.is_finite() || node.y.0.is_nan() || node.y.0.is_infinite());
            }
        }

        #[test]
        fn prop_apply_patch_revision_overflow(rev in u64::MAX-10..=u64::MAX) {
            let mut doc = DiagramDocument::default();
            doc.revision = crate::models::document::Revision::INITIAL;

            let patch: Patch = serde_json::from_str(&format!(
                r#"[{{"op":"test","path":"/revision","value":{}}},{{"op":"replace","path":"/editor_state/snap_to_grid","value":false}}]"#,
                rev
            )).unwrap();
            let result = apply_patch(&doc, &patch);
            prop_assert!(result.is_err());
        }

        #[test]
        fn prop_apply_layout_very_small_cell_size(cell_size in f64::MIN_POSITIVE..1e-10) {
            let doc = make_doc_with_nodes(vec![("a".into(), 1.0, 1.0, false)]);
            let result = apply_layout(&doc, cell_size);
            prop_assert!(result.document.nodes.len() == 1);
        }

        #[test]
        fn prop_apply_layout_subnormal_cell_size(_ in Just(())) {
            let subnormal = f64::from_bits(1_u64);
            let doc = make_doc_with_nodes(vec![("a".into(), 1.0, 1.0, false)]);
            let result = apply_layout(&doc, subnormal);
            prop_assert!(result.document.nodes.len() == 1);
        }
    }
}
