#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_variables,
    unused_imports
)]

use crate::document::{LockState, Node, NodeId, NodeKind, OrderedFloat};
use im::{HashMap, HashSet};

fn test_node(id: &str, parent: Option<&str>) -> (NodeId, Node) {
    let node_id = NodeId::new(id.to_string());
    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: String::new(),
        x: OrderedFloat::new_unchecked(0.0),
        y: OrderedFloat::new_unchecked(0.0),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(100.0),
        font_size: None,
        font_weight: None,
        lock_state: LockState::Unlocked,
        parent: parent.map(|s| NodeId::new(s.to_string())),
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: im::HashMap::new(),
        z_index: 0,
        style: None,
        collapsed: None,
    };
    (node_id, node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grouping::validation::{
        count_nesting_depth, validate_coordinates, validate_selection, GroupingError,
        ValidatedSelection, MAX_SUBGRAPH_NESTING_DEPTH,
    };

    // ── validate_coordinates tests ──────────────────────────────────────────

    #[test]
    fn given_nan_min_x_when_validate_coordinates_then_returns_false() {
        assert_eq!(validate_coordinates(f64::NAN, 0.0, 10.0, 10.0), false);
    }

    #[test]
    fn given_inf_width_when_validate_coordinates_then_returns_false() {
        assert_eq!(validate_coordinates(0.0, 0.0, f64::INFINITY, 10.0), false);
    }

    #[test]
    fn given_neg_inf_height_when_validate_coordinates_then_returns_false() {
        assert_eq!(
            validate_coordinates(0.0, 0.0, 10.0, f64::NEG_INFINITY),
            false
        );
    }

    #[test]
    fn given_zero_width_when_validate_coordinates_then_returns_false() {
        assert_eq!(validate_coordinates(0.0, 0.0, 0.0, 10.0), false);
    }

    #[test]
    fn given_zero_height_when_validate_coordinates_then_returns_false() {
        assert_eq!(validate_coordinates(0.0, 0.0, 10.0, 0.0), false);
    }

    #[test]
    fn given_negative_width_when_validate_coordinates_then_returns_false() {
        assert_eq!(validate_coordinates(0.0, 0.0, -1.0, 10.0), false);
    }

    #[test]
    fn given_valid_inputs_when_validate_coordinates_then_returns_true() {
        assert_eq!(validate_coordinates(10.0, 20.0, 100.0, 50.0), true);
    }

    #[test]
    fn given_neg_inf_min_x_when_validate_coordinates_then_returns_false() {
        assert_eq!(
            validate_coordinates(f64::NEG_INFINITY, 0.0, 10.0, 10.0),
            false
        );
    }

    // ── count_nesting_depth tests ───────────────────────────────────────────

    #[test]
    fn given_no_parent_when_count_nesting_depth_then_returns_0() {
        let nodes: HashMap<NodeId, Node> = HashMap::new();
        let result = count_nesting_depth(&nodes, None);
        assert_eq!(result, 0);
    }

    #[test]
    fn given_root_node_when_count_nesting_depth_then_returns_1() {
        // A root node (no parent) passed as the starting node counts as depth 1
        let (root_id, root_node) = test_node("root", None);
        let nodes = HashMap::unit(root_id.clone(), root_node);
        let result = count_nesting_depth(&nodes, Some(&root_id));
        assert_eq!(result, 1);
    }

    #[test]
    fn given_single_nesting_when_count_nesting_depth_then_returns_2() {
        // child → root: depth = 1 (child) + 1 (root) = 2
        let (root_id, root_node) = test_node("root", None);
        let (child_id, child_node) = test_node("child", Some("root"));
        let nodes = HashMap::from_iter(vec![
            (root_id.clone(), root_node),
            (child_id.clone(), child_node),
        ]);
        let result = count_nesting_depth(&nodes, Some(&child_id));
        assert_eq!(result, 2);
    }

    #[test]
    fn given_chain_of_3_when_count_nesting_depth_then_returns_3() {
        // deep → mid → root: depth = 1 (deep) + 1 (mid) + 1 (root) = 3
        let (root_id, root_node) = test_node("root", None);
        let (mid_id, mid_node) = test_node("mid", Some("root"));
        let (deep_id, deep_node) = test_node("deep", Some("mid"));
        let nodes = HashMap::from_iter(vec![
            (root_id.clone(), root_node),
            (mid_id.clone(), mid_node),
            (deep_id.clone(), deep_node),
        ]);
        let result = count_nesting_depth(&nodes, Some(&deep_id));
        assert_eq!(result, 3);
    }

    // ── validate_selection tests ────────────────────────────────────────────

    #[test]
    fn given_empty_selection_when_validate_selection_then_returns_empty_selection_error() {
        let nodes: HashMap<NodeId, Node> = HashMap::new();
        let selected: HashSet<String> = HashSet::new();
        let result = validate_selection(&nodes, &selected);
        assert_eq!(result, Err(GroupingError::EmptySelection));
    }

    #[test]
    fn given_nonexistent_node_when_validate_selection_then_returns_node_not_found() {
        let nodes: HashMap<NodeId, Node> = HashMap::new();
        let selected: HashSet<String> = HashSet::unit("ghost".to_string());
        let result = validate_selection(&nodes, &selected);
        assert_eq!(
            result,
            Err(GroupingError::NodeNotFound(NodeId::new(
                "ghost".to_string()
            )))
        );
    }

    #[test]
    fn given_valid_selection_when_validate_selection_then_returns_ok() {
        let (n1_id, n1) = test_node("n1", None);
        let (n2_id, n2) = test_node("n2", None);
        let nodes = HashMap::from_iter(vec![(n1_id.clone(), n1), (n2_id.clone(), n2)]);
        let selected: HashSet<String> =
            HashSet::from_iter(vec!["n1".to_string(), "n2".to_string()]);
        let result = validate_selection(&nodes, &selected);
        assert_eq!(result, Ok(()));
    }

    // ── ValidatedSelection::try_new tests ───────────────────────────────────

    #[test]
    fn given_empty_ids_when_try_new_then_returns_empty_selection_error() {
        let nodes: HashMap<NodeId, Node> = HashMap::new();
        let selected: HashSet<NodeId> = HashSet::new();
        let result = ValidatedSelection::try_new(&nodes, &selected);
        assert!(matches!(result, Err(GroupingError::EmptySelection)));
    }

    #[test]
    fn given_locked_node_when_try_new_then_returns_locked_node_error() {
        let (n1_id, mut n1) = test_node("n1", None);
        n1.lock_state = LockState::Locked;
        let nodes = HashMap::unit(n1_id.clone(), n1);
        let selected: HashSet<NodeId> = HashSet::unit(n1_id.clone());
        let result = ValidatedSelection::try_new(&nodes, &selected);
        assert!(matches!(result, Err(GroupingError::LockedNode(_))));
    }

    #[test]
    fn given_nesting_exceeds_limit_when_try_new_then_returns_nested_limit_error() {
        // Build a chain of depth MAX_SUBGRAPH_NESTING_DEPTH (5) so the deepest
        // child's parent chain depth equals the limit, causing rejection.
        let mut pairs: Vec<(NodeId, Node)> = Vec::new();
        for i in 0..=MAX_SUBGRAPH_NESTING_DEPTH {
            let parent = if i == 0 {
                None
            } else {
                Some(format!("n{}", i - 1))
            };
            pairs.push(test_node(&format!("n{}", i), parent.as_deref()));
        }
        let deepest_id = pairs.last().map(|(id, _)| id.clone()).unwrap();
        let nodes = HashMap::from_iter(pairs);
        let selected: HashSet<NodeId> = HashSet::unit(deepest_id.clone());
        let result = ValidatedSelection::try_new(&nodes, &selected);
        assert!(matches!(
            result,
            Err(GroupingError::NestedSubgraphLimitExceeded(
                MAX_SUBGRAPH_NESTING_DEPTH
            ))
        ));
    }

    #[test]
    fn given_valid_nodes_when_try_new_then_returns_ok() {
        let (n1_id, n1) = test_node("n1", None);
        let nodes = HashMap::unit(n1_id.clone(), n1);
        let selected: HashSet<NodeId> = HashSet::unit(n1_id.clone());
        let result = ValidatedSelection::try_new(&nodes, &selected);
        assert!(result.is_ok());
    }
}
