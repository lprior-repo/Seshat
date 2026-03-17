//! Selection evaluation operations
//!
//! Operations for hit-testing and evaluating node selections.

use crate::document::{NodeId, NodeKind, Point};

use super::types::CanvasState;
use super::types::Error;

/// Modifiers for selection actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionModifiers {
    pub ctrl: bool,
}

/// The result of a selection evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionResult {
    NodeSelected(NodeId),
}

/// Evaluates a selection click, considering modifiers like Ctrl to bypass containers.
///
/// # Errors
/// Returns `Error::EmptySelection` if no node was hit.
#[allow(clippy::needless_pass_by_value)]
pub fn evaluate_selection(
    canvas: &CanvasState,
    click_pos: Point,
    modifiers: SelectionModifiers,
) -> Result<SelectionResult, Error> {
    use itertools::Itertools;

    let px = click_pos.x.0;
    let py = click_pos.y.0;

    let hit = canvas
        .nodes
        .iter()
        .sorted_by_key(|(_, n)| -n.z_index)
        .find(|(_id, n)| {
            let nx = n.x.0;
            let ny = n.y.0;
            let nw = n.width.0;
            let nh = n.height.0;

            let intersects = px >= nx && px <= nx + nw && py >= ny && py <= ny + nh;

            if !intersects {
                return false;
            }

            // If we have ctrl modifier, bypass containers
            if modifiers.ctrl && n.kind == NodeKind::Subgraph {
                return false;
            }

            // If a node is in a collapsed parent, it shouldn't be hit-testable
            if let Some(parent_id) = &n.parent {
                if let Some(parent) = canvas.nodes.get(parent_id) {
                    if parent.collapsed.unwrap_or(false) {
                        return false;
                    }
                }
            }

            true
        });

    if let Some((id, _)) = hit {
        Ok(SelectionResult::NodeSelected(id.clone()))
    } else {
        Err(Error::EmptySelection)
    }
}
