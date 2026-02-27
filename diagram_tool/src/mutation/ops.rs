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
