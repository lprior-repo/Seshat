//! Selection evaluation operations

use crate::document::{NodeId, NodeKind};
use crate::geometry::Point;
use crate::subgraph::types::CanvasState;
use crate::subgraph::types::Error;
use itertools::Itertools;

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
/// Uses `itertools::sorted_by_key` on `im::HashMap` for hit testing — faster than
/// building a full spatial index for single-point queries at typical document sizes.
///
/// # Errors
/// Returns `Error::EmptySelection` if no node was hit.
#[allow(clippy::needless_pass_by_value)]
pub fn evaluate_selection(
    canvas: &CanvasState,
    click_pos: Point,
    modifiers: SelectionModifiers,
) -> Result<SelectionResult, Error> {
    let hit = canvas
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            if modifiers.ctrl && node.kind == NodeKind::Subgraph {
                return None;
            }

            if let Some(parent_id) = &node.parent {
                if let Some(parent) = canvas.nodes.get(parent_id) {
                    if parent.collapsed == Some(true) {
                        return None;
                    }
                }
            }

            let nx = node.x.0;
            let ny = node.y.0;
            let nw = node.width.0;
            let nh = node.height.0;
            if click_pos.x < nx
                || click_pos.x > nx + nw
                || click_pos.y < ny
                || click_pos.y > ny + nh
            {
                return None;
            }

            Some((node.z_index, id.clone()))
        })
        .sorted_by_key(|(z_index, _)| std::cmp::Reverse(*z_index))
        .next()
        .map(|(_, id)| id);

    hit.map(SelectionResult::NodeSelected)
        .ok_or(Error::EmptySelection)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
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
            z_index: 1,
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
        let click_pos = Point::new(80.0, 80.0);
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
        let click_pos = Point::new(80.0, 80.0);
        let modifiers = SelectionModifiers { ctrl: true };

        let result = evaluate_selection(&canvas, click_pos, modifiers);
        assert_eq!(result, Err(Error::EmptySelection));
    }

    #[test]
    fn test_evaluate_selection_collapsed_parent_hides_child() {
        let mut canvas = setup_canvas();
        let parent = canvas
            .nodes
            .get_mut(&NodeId::new("p1".to_string()))
            .unwrap();
        parent.collapsed = Some(true);

        let click_pos = Point::new(20.0, 20.0);
        let modifiers = SelectionModifiers { ctrl: false };

        let result = evaluate_selection(&canvas, click_pos, modifiers).unwrap();
        assert_eq!(
            result,
            SelectionResult::NodeSelected(NodeId::new("p1".to_string()))
        );
    }
}
