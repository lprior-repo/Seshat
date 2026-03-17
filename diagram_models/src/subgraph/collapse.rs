//! Collapse/expand operations for subgraph containers
//!
//! Operations for toggling collapsed state of container nodes.

use crate::document::{Node, NodeId, NodeKind};

use super::types::CanvasState;
use super::types::Error;

/// Toggles the collapsed state of a container.
///
/// # Errors
/// Returns errors based on contract.
#[allow(clippy::needless_pass_by_value)]
pub fn toggle_collapse(canvas: &mut CanvasState, group_id: NodeId) -> Result<(), Error> {
    let group = canvas
        .nodes
        .get(&group_id)
        .ok_or_else(|| Error::NodeNotFound(group_id.clone()))?;

    if group.kind != NodeKind::Subgraph {
        return Err(Error::InvalidNodeType);
    }

    let is_collapsed = group.collapsed.unwrap_or(false);

    let updated_group = Node {
        collapsed: Some(!is_collapsed),
        ..group.clone()
    };

    canvas.nodes = canvas.nodes.update(group_id, updated_group);

    Ok(())
}
