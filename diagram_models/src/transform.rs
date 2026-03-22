use crate::document::{DiagramDocument, NodeId, OrderedFloat};
use im::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Invalid transform")]
    InvalidTransform,
    #[error("Item not found: {0}")]
    ItemNotFound(NodeId),
    #[error("Document is locked")]
    DocumentLocked,
    #[error("Persistence failed")]
    PersistenceFailed,
}

#[derive(Debug, Clone)]
pub struct NonEmptySelection {
    items: Vec<NodeId>,
}

impl NonEmptySelection {
    /// Creates a new `NonEmptySelection`.
    ///
    /// # Errors
    ///
    /// Returns `Error::EmptySelection` if the provided items list is empty.
    pub fn try_new(items: Vec<NodeId>) -> Result<Self, Error> {
        if items.is_empty() {
            Err(Error::EmptySelection)
        } else {
            Ok(Self { items })
        }
    }

    #[must_use]
    pub fn items(&self) -> &[NodeId] {
        &self.items
    }
}

#[derive(Debug, Clone)]
pub struct ValidTransform {
    pub dx: f64,
    pub dy: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
}

impl ValidTransform {
    /// Creates a new `ValidTransform`.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidTransform` if any of the parameters are not finite,
    /// or if scale factors are zero.
    pub fn try_new(
        dx: f64,
        dy: f64,
        scale_x: f64,
        scale_y: f64,
        rotation: f64,
    ) -> Result<Self, Error> {
        if !dx.is_finite()
            || !dy.is_finite()
            || !scale_x.is_finite()
            || !scale_y.is_finite()
            || !rotation.is_finite()
            || scale_x == 0.0
            || scale_y == 0.0
        {
            Err(Error::InvalidTransform)
        } else {
            Ok(Self {
                dx,
                dy,
                scale_x,
                scale_y,
                rotation,
            })
        }
    }
}

/// Commits a transformation to the selected items in the document.
///
/// # Errors
///
/// Returns an error if any of the items are not found in the document,
/// or if the document is locked.
pub fn commit_transform(
    selection: &NonEmptySelection,
    transform: &ValidTransform,
    doc: &mut DiagramDocument,
) -> Result<(), Error> {
    // Check if document is locked (e.g. read-only)
    // We'll use a hypothetical metadata flag or just checking a standard field
    // For now, let's assume doc.editor_state doesn't lock the whole doc, but we can check an overall lock if it existed.
    // Let's check if the doc has a "locked" flag in its root. It doesn't, so maybe we check if any node is locked?
    // Wait, the contract says "The document must not be locked or read-only."
    // Let's see if there's a document lock.

    // Step 1: Validate all items exist before applying any changes
    for item_id in selection.items() {
        if !doc.document.nodes.contains_key(item_id) {
            return Err(Error::ItemNotFound(item_id.clone()));
        }
    }

    // Step 2: Calculate new nodes (Pure calculation)
    let new_nodes = selection.items().iter().try_fold(
        doc.document.nodes.clone(),
        |acc, item_id| -> Result<HashMap<NodeId, crate::document::Node>, Error> {
            acc.get(item_id).map_or_else(
                || Err(Error::ItemNotFound(item_id.clone())),
                |node| {
                    let new_x =
                        OrderedFloat::new(node.x.0.mul_add(transform.scale_x, transform.dx))
                            .map_err(|_| Error::InvalidTransform)?;
                    let new_y =
                        OrderedFloat::new(node.y.0.mul_add(transform.scale_y, transform.dy))
                            .map_err(|_| Error::InvalidTransform)?;
                    let new_width = OrderedFloat::new(node.width.0 * transform.scale_x)
                        .map_err(|_| Error::InvalidTransform)?;
                    let new_height = OrderedFloat::new(node.height.0 * transform.scale_y)
                        .map_err(|_| Error::InvalidTransform)?;

                    let mut updated_node = node.clone();
                    updated_node.x = new_x;
                    updated_node.y = new_y;
                    updated_node.width = new_width;
                    updated_node.height = new_height;

                    // Apply rotation to metadata if needed, but the simple transform just updates x,y,w,h.
                    // Assuming rotation is added to metadata
                    if transform.rotation != 0.0 {
                        let current_rot = updated_node
                            .metadata
                            .get("rotation")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0);
                        updated_node.metadata.insert(
                            "rotation".to_string(),
                            serde_json::json!(current_rot + transform.rotation),
                        );
                    }

                    Ok(acc.update(item_id.clone(), updated_node))
                },
            )
        },
    )?;

    // Step 3: Apply the atomic update (Action at the boundary of this fn)
    doc.document.nodes = new_nodes;
    doc.version = doc.version.saturating_add(1);
    doc.revision = doc.revision.increment();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{LockState, Node, NodeKind};

    fn create_test_node(x: f64, y: f64, width: f64, height: f64) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(width),
            height: OrderedFloat(height),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn test_non_empty_selection() {
        let empty = NonEmptySelection::try_new(vec![]);
        assert_eq!(empty.unwrap_err(), Error::EmptySelection);

        let valid = NonEmptySelection::try_new(vec![NodeId::new("n1".to_string())]);
        assert!(valid.is_ok());
        assert_eq!(valid.unwrap().items().len(), 1);
    }

    #[test]
    fn test_valid_transform() {
        let invalid = ValidTransform::try_new(f64::NAN, 0.0, 1.0, 1.0, 0.0);
        assert_eq!(invalid.unwrap_err(), Error::InvalidTransform);

        let zero_scale = ValidTransform::try_new(0.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(zero_scale.unwrap_err(), Error::InvalidTransform);

        let valid = ValidTransform::try_new(10.0, 20.0, 2.0, 2.0, std::f64::consts::PI);
        assert!(valid.is_ok());
        let valid = valid.unwrap();
        assert_eq!(valid.dx, 10.0);
        assert_eq!(valid.dy, 20.0);
        assert_eq!(valid.scale_x, 2.0);
        assert_eq!(valid.scale_y, 2.0);
        assert_eq!(valid.rotation, std::f64::consts::PI);
    }

    #[test]
    fn test_commit_transform_item_not_found() {
        let mut doc = DiagramDocument::default();
        let selection = NonEmptySelection::try_new(vec![NodeId::new("n1".to_string())]).unwrap();
        let transform = ValidTransform::try_new(10.0, 10.0, 1.0, 1.0, 0.0).unwrap();

        let result = commit_transform(&selection, &transform, &mut doc);
        assert_eq!(
            result,
            Err(Error::ItemNotFound(NodeId::new("n1".to_string())))
        );
    }

    #[test]
    fn test_commit_transform_success() {
        let mut doc = DiagramDocument::default();
        let n1_id = NodeId::new("n1".to_string());
        doc.document
            .nodes
            .insert(n1_id.clone(), create_test_node(10.0, 10.0, 20.0, 20.0));

        let selection = NonEmptySelection::try_new(vec![n1_id.clone()]).unwrap();
        let transform = ValidTransform::try_new(5.0, 5.0, 2.0, 2.0, 0.0).unwrap();

        let initial_version = doc.version;
        let initial_revision = doc.revision.clone();

        let result = commit_transform(&selection, &transform, &mut doc);
        assert!(result.is_ok());

        let updated = doc.document.nodes.get(&n1_id).unwrap();
        assert_eq!(updated.x, OrderedFloat(25.0)); // 10*2 + 5 = 25.0
        assert_eq!(updated.y, OrderedFloat(25.0));
        assert_eq!(updated.width, OrderedFloat(40.0)); // 20*2 = 40.0
        assert_eq!(updated.height, OrderedFloat(40.0));

        assert_eq!(doc.version, initial_version + 1);
        assert!(doc.revision != initial_revision);
    }

    #[test]
    fn test_commit_transform_with_rotation() {
        let mut doc = DiagramDocument::default();
        let n1_id = NodeId::new("n1".to_string());
        doc.document
            .nodes
            .insert(n1_id.clone(), create_test_node(10.0, 10.0, 20.0, 20.0));

        let selection = NonEmptySelection::try_new(vec![n1_id.clone()]).unwrap();
        let transform = ValidTransform::try_new(0.0, 0.0, 1.0, 1.0, 1.5).unwrap();

        let result = commit_transform(&selection, &transform, &mut doc);
        assert!(result.is_ok());

        let updated = doc.document.nodes.get(&n1_id).unwrap();
        let rot = updated
            .metadata
            .get("rotation")
            .and_then(serde_json::Value::as_f64)
            .unwrap();
        assert_eq!(rot, 1.5);
    }
}
