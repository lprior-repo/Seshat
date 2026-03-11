use crate::models::document::NodeId;
use crate::models::selection_ops::{
    clear_selection, marquee_select, select_item, DiagramState, Error, Point, Rect, SelectionMode,
};
use im::{HashMap, HashSet};

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_state() -> DiagramState {
        let mut nodes = HashMap::new();
        // Node A: (10, 10) to (60, 60)
        nodes.insert(
            NodeId::new("node-a".to_string()),
            Rect::new(10.0, 10.0, 50.0, 50.0),
        );
        // Node B: (100, 100) to (150, 150)
        nodes.insert(
            NodeId::new("node-b".to_string()),
            Rect::new(100.0, 100.0, 50.0, 50.0),
        );

        DiagramState::with_nodes(nodes)
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_001_click_replaces_selection() {
        let mut state = setup_state();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        // Pre-select A
        state.selected_items = HashSet::unit(node_a.clone());

        // When
        select_item(&mut state, node_b.clone(), SelectionMode::Replace).unwrap();

        // Then
        assert!(state.selected_items.contains(&node_b));
        assert!(!state.selected_items.contains(&node_a));
        assert_eq!(state.selected_items.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_shift_click_adds_to_selection() {
        let mut state = setup_state();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        state.selected_items = HashSet::unit(node_a.clone());

        // When
        select_item(&mut state, node_b.clone(), SelectionMode::Toggle).unwrap();

        // Then
        assert!(state.selected_items.contains(&node_a));
        assert!(state.selected_items.contains(&node_b));
        assert_eq!(state.selected_items.len(), 2);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_002_shift_click_removes_from_selection() {
        let mut state = setup_state();
        let node_a = NodeId::new("node-a".to_string());
        let node_b = NodeId::new("node-b".to_string());

        state.selected_items = HashSet::unit(node_a.clone()).update(node_b.clone());

        // When
        select_item(&mut state, node_b.clone(), SelectionMode::Toggle).unwrap();

        // Then
        assert!(state.selected_items.contains(&node_a));
        assert!(!state.selected_items.contains(&node_b));
        assert_eq!(state.selected_items.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_003_left_to_right_marquee_selects_contained_nodes() {
        let mut state = setup_state();
        // Node A is at 10,10 to 60,60
        // Left-to-Right contain requires full containment
        // Marquee from 0,0 to 30,30 only partially covers A
        marquee_select(&mut state, Point::new(0.0, 0.0), Point::new(30.0, 30.0)).unwrap();

        assert!(!state
            .selected_items
            .contains(&NodeId::new("node-a".to_string())));
        assert_eq!(state.selected_items.len(), 0);

        // Marquee from 0,0 to 70,70 fully covers A
        marquee_select(&mut state, Point::new(0.0, 0.0), Point::new(70.0, 70.0)).unwrap();

        assert!(state
            .selected_items
            .contains(&NodeId::new("node-a".to_string())));
        assert_eq!(state.selected_items.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_004_click_empty_canvas_clears_selection() {
        let mut state = setup_state();
        state.selected_items = HashSet::unit(NodeId::new("node-a".to_string()));

        // When
        clear_selection(&mut state).unwrap();

        // Then
        assert!(state.selected_items.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_sel_005_right_to_left_marquee_selects_intersected_nodes() {
        let mut state = setup_state();
        // Modify node-a to be smaller so 30,30 is empty
        state.nodes.insert(
            NodeId::new("node-a".to_string()),
            Rect::new(10.0, 10.0, 10.0, 10.0), // 10,10 to 20,20
        );

        // Right-to-Left intersect requires only partial coverage
        // Marquee from 30,30 to 0,0 (R->L)
        marquee_select(&mut state, Point::new(30.0, 30.0), Point::new(0.0, 0.0)).unwrap();

        assert!(state
            .selected_items
            .contains(&NodeId::new("node-a".to_string())));
        assert_eq!(state.selected_items.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_selecting_non_existent_node() {
        let mut state = setup_state();
        let result = select_item(
            &mut state,
            NodeId::new("non-existent".to_string()),
            SelectionMode::Replace,
        );

        assert_eq!(
            result,
            Err(Error::ItemNotFound(NodeId::new("non-existent".to_string())))
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_returns_error_when_marquee_starts_on_node() {
        let mut state = setup_state();
        // Point 20,20 is inside Node A
        let result = marquee_select(&mut state, Point::new(20.0, 20.0), Point::new(100.0, 100.0));

        assert_eq!(result, Err(Error::InvalidInteractionState));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_marquee_selection_with_no_contained_or_intersected_nodes() {
        let mut state = setup_state();
        // Box from 200,200 to 300,300
        marquee_select(
            &mut state,
            Point::new(200.0, 200.0),
            Point::new(300.0, 300.0),
        )
        .unwrap();

        assert!(state.selected_items.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_marquee_selection_exactly_matching_node_bounds() {
        let mut state = setup_state();
        // Change Node A to start slightly inside to allow marquee to start on empty space
        state.nodes.insert(
            NodeId::new("node-a".to_string()),
            Rect::new(10.1, 10.1, 49.8, 49.8), // completely inside 10 to 60
        );

        // L->R exact match of the outer conceptual bounds
        marquee_select(&mut state, Point::new(10.0, 10.0), Point::new(60.0, 60.0)).unwrap();

        assert!(state
            .selected_items
            .contains(&NodeId::new("node-a".to_string())));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_selection_set_contains_no_duplicates() {
        // HashSet naturally prevents duplicates.
        let mut state = setup_state();
        select_item(
            &mut state,
            NodeId::new("node-a".to_string()),
            SelectionMode::Replace,
        )
        .unwrap();

        // Try adding it again via Toggle, which will remove it, so do another replace
        select_item(
            &mut state,
            NodeId::new("node-a".to_string()),
            SelectionMode::Replace,
        )
        .unwrap();

        assert_eq!(state.selected_items.len(), 1);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_invariant_selection_set_only_contains_existing_nodes() {
        let mut state = setup_state();
        // Attempting to select a non-existent node returns error and doesn't modify state
        let _ = select_item(
            &mut state,
            NodeId::new("ghost".to_string()),
            SelectionMode::Replace,
        );

        assert!(state.selected_items.is_empty());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p1_violation_returns_item_not_found_error() {
        let mut state = setup_state();
        let result = select_item(
            &mut state,
            NodeId::new("non-existent".to_string()),
            SelectionMode::Replace,
        );
        assert_eq!(
            result,
            Err(Error::ItemNotFound(NodeId::new("non-existent".to_string())))
        );
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn test_p2_violation_returns_invalid_interaction_state_error() {
        let mut state = setup_state();
        let result = marquee_select(
            &mut state,
            Point::new(15.0, 15.0), // On Node A
            Point::new(100.0, 100.0),
        );
        assert_eq!(result, Err(Error::InvalidInteractionState));
    }
}
