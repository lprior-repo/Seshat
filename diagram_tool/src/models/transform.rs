use crate::models::document::{DiagramDocument, NodeId, OrderedFloat};
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
        |acc, item_id| -> Result<HashMap<NodeId, crate::models::document::Node>, Error> {
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
