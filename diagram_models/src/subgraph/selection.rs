//! Selection evaluation operations
//!
//! Operations for hit-testing and evaluating node selections.

use crate::document::{NodeId, NodeKind};
use crate::geometry::Point;

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

    let px = click_pos.x;
    let py = click_pos.y;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{LockState, Node, OrderedFloat};
    use im::HashMap;

    fn setup_canvas() -> CanvasState {
        let mut nodes = HashMap::new();

        let p1_node = Node {
            kind: NodeKind::Subgraph,
            icon: String::new(),
            label: "p1".to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: Some(false),
        };

        let c1_node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "c1".to_string(),
            x: OrderedFloat(10.0),
            y: OrderedFloat(10.0),
            width: OrderedFloat(50.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: Some(NodeId::new("p1".to_string())),
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: HashMap::new(),
            z_index: 1, // higher z-index
            style: None,
            collapsed: None,
        };

        nodes.insert(NodeId::new("p1".to_string()), p1_node);
        nodes.insert(NodeId::new("c1".to_string()), c1_node);

        CanvasState {
            nodes,
            edges: HashMap::new(),
        }
    }

    #[test]
    fn test_evaluate_selection_hits_child() {
        let canvas = setup_canvas();
        let click_pos = Point::new(20.0, 20.0);
        let modifiers = SelectionModifiers { ctrl: false };

        let result = evaluate_selection(&canvas, click_pos, modifiers).unwrap();
        assert_eq!(
            result,
            SelectionResult::NodeSelected(NodeId::new("c1".to_string()))
        );
    }

    #[test]
    fn test_evaluate_selection_hits_parent() {
        let canvas = setup_canvas();
        let click_pos = Point::new(80.0, 80.0); // Inside parent, outside child
        let modifiers = SelectionModifiers { ctrl: false };

        let result = evaluate_selection(&canvas, click_pos, modifiers).unwrap();
        assert_eq!(
            result,
            SelectionResult::NodeSelected(NodeId::new("p1".to_string()))
        );
    }

    #[test]
    fn test_evaluate_selection_misses() {
        let canvas = setup_canvas();
        let click_pos = Point::new(200.0, 200.0);
        let modifiers = SelectionModifiers { ctrl: false };

        let result = evaluate_selection(&canvas, click_pos, modifiers);
        assert_eq!(result, Err(Error::EmptySelection));
    }

    #[test]
    fn test_evaluate_selection_ctrl_ignores_parent() {
        let canvas = setup_canvas();
        let click_pos = Point::new(80.0, 80.0); // Inside parent
        let modifiers = SelectionModifiers { ctrl: true };

        let result = evaluate_selection(&canvas, click_pos, modifiers);
        assert_eq!(result, Err(Error::EmptySelection)); // parent ignored
    }

    #[test]
    fn test_evaluate_selection_collapsed_parent_hides_child() {
        let mut canvas = setup_canvas();
        let parent = canvas
            .nodes
            .get_mut(&NodeId::new("p1".to_string()))
            .unwrap();
        parent.collapsed = Some(true); // collapse the parent

        let click_pos = Point::new(20.0, 20.0); // Inside child
        let modifiers = SelectionModifiers { ctrl: false };

        let result = evaluate_selection(&canvas, click_pos, modifiers).unwrap();
        // Since child is hidden inside collapsed parent, the parent itself should be hit instead
        assert_eq!(
            result,
            SelectionResult::NodeSelected(NodeId::new("p1".to_string()))
        );
    }
}
