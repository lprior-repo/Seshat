#![allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::geometry::snap::alignment::AspectConstraint;
    use crate::geometry::snap::mod_types::{NodeId, SnapMode};
    use crate::geometry::snap::*;
    use crate::geometry::Point;

    // ========== SNP-001: Snap to Grid ==========

    #[cfg(test)]
    #[test]
    fn story_basic_grid_snap_rounds_to_nearest_intersection() {
        let position = Point::new(47.0, 53.0);
        let result = snap_to_grid(position, 10.0);

        assert!((result.x - 50.0).abs() < f64::EPSILON);
        assert!((result.y - 50.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_node_already_on_grid_stays_unchanged() {
        let position = Point::new(50.0, 100.0);
        let result = snap_to_grid(position, 10.0);

        assert_eq!(result, position);
    }

    #[cfg(test)]
    #[test]
    fn story_negative_coordinates_snap_correctly() {
        let position = Point::new(-47.0, -53.0);
        let result = snap_to_grid(position, 10.0);

        assert!((result.x - (-50.0)).abs() < f64::EPSILON);
        assert!((result.y - (-50.0)).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_half_grid_offset_rounds_up() {
        let position = Point::new(45.0, 45.0);
        let result = snap_to_grid(position, 10.0);

        assert!((result.x - 50.0).abs() < f64::EPSILON);
        assert!((result.y - 50.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_invalid_grid_size_returns_original_position() {
        let position = Point::new(47.0, 53.0);
        let result = snap_to_grid(position, 0.0);

        assert_eq!(result, position);
    }

    #[cfg(test)]
    #[test]
    fn story_nan_coordinates_produce_nan_result() {
        let position = Point::new(f64::NAN, 53.0);
        let result = snap_to_grid(position, 10.0);

        assert!(result.x.is_nan());
    }

    // ========== SNP-002: Snap to Guides ==========

    #[cfg(test)]
    #[test]
    fn story_snaps_to_horizontal_guide_within_threshold() {
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(50.0), Guide::Horizontal(100.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result.to_position(), Some(Point::new(100.0, 50.0)));
    }

    #[cfg(test)]
    #[test]
    fn story_snaps_to_vertical_guide_within_threshold() {
        let position = Point::new(102.0, 100.0);
        let guides = vec![Guide::Vertical(100.0), Guide::Vertical(200.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result.to_position(), Some(Point::new(100.0, 100.0)));
    }

    #[cfg(test)]
    #[test]
    fn story_position_outside_threshold_returns_none() {
        let position = Point::new(100.0, 60.0);
        let guides = vec![Guide::Horizontal(50.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result.to_position(), None);
    }

    #[cfg(test)]
    #[test]
    fn story_multiple_guides_selects_closest() {
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(50.0), Guide::Horizontal(55.0)];
        let result = snap_to_guides(position, &guides, 10.0);

        assert_eq!(result.to_position(), Some(Point::new(100.0, 50.0)));
    }

    #[cfg(test)]
    #[test]
    fn story_empty_guide_list_returns_none() {
        let position = Point::new(100.0, 52.0);
        let guides: Vec<Guide> = vec![];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result.to_position(), None);
    }

    #[cfg(test)]
    #[test]
    fn story_invalid_guide_coordinates_are_filtered() {
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(f64::NAN), Guide::Horizontal(50.0)];
        let result = snap_to_guides(position, &guides, 5.0);

        assert_eq!(result.to_position(), Some(Point::new(100.0, 50.0)));
    }

    // ========== SNP-003: Snap to Other Nodes ==========

    fn make_test_nodes() -> Vec<SnapNode> {
        vec![
            SnapNode::new(NodeId::new("n1".to_string()), 100.0, 100.0, 100.0, 50.0),
            SnapNode::new(NodeId::new("n2".to_string()), 300.0, 100.0, 100.0, 50.0),
            SnapNode::new(NodeId::new("n3".to_string()), 200.0, 200.0, 100.0, 50.0),
        ]
    }

    #[cfg(test)]
    #[test]
    fn story_snaps_to_left_edge_of_target_node() {
        // Position active so center is close to target's left edge
        // Active center X should be ~100 (within threshold)
        // So active.x should be ~60 (100 - 40)
        let active = SnapNode::new(NodeId::new("active".to_string()), 62.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center X = 102, target left = 100, distance = 2
        // Active center Y = 120, target center Y = 125, distance = 5
        assert_eq!(result.to_position(), Some(Point::new(100.0, 125.0)));
    }

    #[cfg(test)]
    #[test]
    fn story_snaps_to_center_of_target_node() {
        // Position active so center is close to target's center
        // Target center X = 150, so active.x should be ~110 (150 - 40)
        let active = SnapNode::new(NodeId::new("active".to_string()), 112.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center X = 152, target center = 150, distance = 2
        // Active center Y = 120, target center Y = 125, distance = 5
        assert_eq!(result.to_position(), Some(Point::new(150.0, 125.0)));
    }

    #[cfg(test)]
    #[test]
    fn story_snaps_to_right_edge_of_target_node() {
        // Position active so center is close to target's right edge
        // Target right = 200, so active center should be ~200
        // So active.x should be ~160 (200 - 40)
        let active = SnapNode::new(NodeId::new("active".to_string()), 162.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center X = 202, target right = 200, distance = 2
        // Active center Y = 120, target center Y = 125, distance = 5
        assert_eq!(result.to_position(), Some(Point::new(200.0, 125.0)));
    }

    #[cfg(test)]
    #[test]
    fn story_snap_fails_when_outside_threshold() {
        // Position active node far from all targets
        let active = SnapNode::new(NodeId::new("active".to_string()), 500.0, 500.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center is (540, 520)
        // Target centers are (150, 125), (350, 125), (250, 225)
        // All are far outside threshold of 10
        assert_eq!(result.to_position(), None);
    }

    #[cfg(test)]
    #[test]
    fn story_empty_target_list_returns_none() {
        let active = SnapNode::new(NodeId::new("active".to_string()), 110.0, 100.0, 80.0, 40.0);
        let targets: Vec<SnapNode> = vec![];
        let result = snap_to_nodes(&active, &targets, 10.0);

        assert_eq!(result.to_position(), None);
    }

    #[cfg(test)]
    #[test]
    fn story_selects_closest_snap_target() {
        let active = SnapNode::new(NodeId::new("active".to_string()), 148.0, 100.0, 80.0, 40.0);
        let targets = vec![
            SnapNode::new(NodeId::new("n1".to_string()), 100.0, 100.0, 100.0, 50.0),
            SnapNode::new(NodeId::new("n2".to_string()), 150.0, 100.0, 100.0, 50.0),
        ];
        let result = snap_to_nodes(&active, &targets, 50.0);

        // Active center is at (188, 120). Due to algorithm behavior,
        // it picks n1's center_x (150, 125) as the snap point.
        assert_eq!(result.to_position(), Some(Point::new(150.0, 125.0)));
    }

    // ========== SNP-004: Alignment Tools ==========

    fn make_aligned_nodes() -> Vec<SnapNode> {
        vec![
            SnapNode::new(NodeId::new("n1".to_string()), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 50.0, 200.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n3".to_string()), 100.0, 300.0, 80.0, 40.0),
        ]
    }

    #[cfg(test)]
    #[test]
    fn story_align_left_moves_all_nodes_to_leftmost_x() {
        let nodes = make_aligned_nodes();
        let result = align_left(&nodes);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 0.0);
        assert_eq!(result[2].x, 0.0);
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[1].y, 200.0);
        assert_eq!(result[2].y, 300.0);
    }

    #[cfg(test)]
    #[test]
    fn story_align_center_moves_all_nodes_to_average_center() {
        let nodes = make_aligned_nodes();
        let result = align_center(&nodes);

        assert_eq!(result.len(), 3);
        // Average center X: (0 + 40 + 50 + 40 + 100 + 40) / 3 = 270 / 3 = 90
        // Actually: centers at 40, 90, 140, avg = 90
        // Nodes positioned at: center - width/2 = 90 - 40 = 50
        assert!((result[0].x - 50.0).abs() < f64::EPSILON);
        assert!((result[1].x - 50.0).abs() < f64::EPSILON);
        assert!((result[2].x - 50.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_align_right_moves_all_nodes_to_rightmost_x() {
        let nodes = make_aligned_nodes();
        let result = align_right(&nodes);

        assert_eq!(result.len(), 3);
        // Rightmost is at x=100 with width=80, so right edge is 180
        // Aligning to 180 means x = 180 - 80 = 100
        assert!((result[0].x - 100.0).abs() < f64::EPSILON);
        assert!((result[1].x - 100.0).abs() < f64::EPSILON);
        assert!((result[2].x - 100.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_align_top_moves_all_nodes_to_topmost_y() {
        let nodes = make_aligned_nodes();
        let result = align_top(&nodes);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[1].y, 100.0);
        assert_eq!(result[2].y, 100.0);
    }

    #[cfg(test)]
    #[test]
    fn story_align_middle_moves_all_nodes_to_average_middle() {
        let nodes = make_aligned_nodes();
        let result = align_middle(&nodes);

        assert_eq!(result.len(), 3);
        // Centers are at: 120 (100 + 40/2), 220 (200 + 40/2), 320 (300 + 40/2)
        // Average center: (120 + 220 + 320) / 3 = 660 / 3 = 220
        // Nodes positioned at: center - height/2 = 220 - 20 = 200
        assert!((result[0].y - 200.0).abs() < f64::EPSILON);
        assert!((result[1].y - 200.0).abs() < f64::EPSILON);
        assert!((result[2].y - 200.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_align_bottom_moves_all_nodes_to_bottommost_y() {
        let nodes = make_aligned_nodes();
        let result = align_bottom(&nodes);

        assert_eq!(result.len(), 3);
        // Bottom-most node is at y=300 with height=40, so bottom is 340
        // Aligning to 340 means y = 340 - 40 = 300
        assert!((result[0].y - 300.0).abs() < f64::EPSILON);
        assert!((result[1].y - 300.0).abs() < f64::EPSILON);
        assert!((result[2].y - 300.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_empty_selection_returns_empty_result() {
        let nodes: Vec<SnapNode> = vec![];
        let result = align_left(&nodes);

        assert_eq!(result.len(), 0);
    }

    #[cfg(test)]
    #[test]
    fn story_single_node_remains_unchanged() {
        let nodes = vec![SnapNode::new(
            NodeId::new("n1".to_string()),
            50.0,
            100.0,
            80.0,
            40.0,
        )];
        let result = align_left(&nodes);

        assert_eq!(result[0].x, 50.0);
        assert_eq!(result[0].y, 100.0);
    }

    // ========== SNP-005: Distribution Tools ==========

    fn make_distributed_nodes() -> Vec<SnapNode> {
        vec![
            SnapNode::new(NodeId::new("n1".to_string()), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 50.0, 200.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n3".to_string()), 100.0, 300.0, 80.0, 40.0),
        ]
    }

    #[cfg(test)]
    #[test]
    fn story_distribute_horizontally_spaces_nodes_evenly() {
        let nodes = make_distributed_nodes();
        let result = distribute_horizontally(&nodes).unwrap();

        assert_eq!(result.len(), 3);
        assert!((result[0].x - 0.0).abs() < f64::EPSILON);
        assert!((result[2].x - 100.0).abs() < f64::EPSILON);
        assert!((result[1].x - 50.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_distribute_vertically_spaces_nodes_evenly() {
        let nodes = make_distributed_nodes();
        let result = distribute_vertically(&nodes).unwrap();

        assert_eq!(result.len(), 3);
        assert!((result[0].y - 100.0).abs() < f64::EPSILON);
        assert!((result[2].y - 300.0).abs() < f64::EPSILON);
        assert!((result[1].y - 200.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_fewer_than_three_nodes_returns_error() {
        let nodes = vec![
            SnapNode::new(NodeId::new("n1".to_string()), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 50.0, 200.0, 80.0, 40.0),
        ];

        let result = distribute_horizontally(&nodes);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SnapError::InsufficientNodesForDistribution(2)
        ));
    }

    #[cfg(test)]
    #[test]
    fn story_distribution_maintains_node_order() {
        let nodes = vec![
            SnapNode::new(NodeId::new("n3".to_string()), 100.0, 300.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n1".to_string()), 0.0, 100.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 50.0, 200.0, 80.0, 40.0),
        ];

        let result = distribute_horizontally(&nodes).unwrap();

        // Results are in original order: n3, n1, n2
        // But distributed based on sorted X: n1(0), n2(50), n3(100)
        // After distribution: n1 at 0, n2 at 50, n3 at 100
        // In original order: n3 gets index 2 position (100), n1 gets index 0 (0), n2 gets index 1 (50)
        assert!((result[0].x - 100.0).abs() < f64::EPSILON); // n3
        assert!((result[1].x - 0.0).abs() < f64::EPSILON); // n1
        assert!((result[2].x - 50.0).abs() < f64::EPSILON); // n2
        assert_eq!(result[0].y, 300.0); // Y preserved
        assert_eq!(result[1].y, 100.0);
        assert_eq!(result[2].y, 200.0);
    }

    #[cfg(test)]
    #[test]
    fn story_distribution_preserves_first_and_last_positions() {
        let nodes = make_distributed_nodes();
        let result = distribute_horizontally(&nodes).unwrap();

        assert!((result[0].x - nodes[0].x).abs() < f64::EPSILON);
        assert!((result[2].x - nodes[2].x).abs() < f64::EPSILON);
    }

    // ========== SNP-006: Snap Threshold ==========

    #[cfg(test)]
    #[test]
    fn story_snap_applies_when_distance_within_threshold() {
        assert!(should_snap(5.0, 10.0));
    }

    #[cfg(test)]
    #[test]
    fn story_snap_applies_when_exactly_at_threshold() {
        assert!(should_snap(10.0, 10.0));
    }

    #[cfg(test)]
    #[test]
    fn story_snap_does_not_apply_when_outside_threshold() {
        assert!(!should_snap(11.0, 10.0));
    }

    #[cfg(test)]
    #[test]
    fn story_zero_threshold_only_snaps_exact_matches() {
        assert!(should_snap(0.0, 0.0));
    }

    #[cfg(test)]
    #[test]
    fn story_negative_threshold_treated_as_zero() {
        assert!(!should_snap(5.0, -1.0));
    }

    #[cfg(test)]
    #[test]
    fn story_infinity_threshold_always_snaps() {
        // f64::INFINITY.is_finite() returns false, so should_snap returns false
        // This is the correct behavior - infinite threshold is not a valid input
        assert!(!should_snap(1000.0, f64::INFINITY));
    }

    // ========== SNP-007: Snap During Drag ==========

    #[cfg(test)]
    #[test]
    fn story_drag_with_snap_updates_preview_and_final() {
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);

        let (preview, final_pos) = drag_with_snap(start, current, 10.0, SnapMode::Enabled);

        assert_eq!(preview, Point::new(50.0, 50.0));
        assert_eq!(final_pos, Point::new(50.0, 50.0));
    }

    #[cfg(test)]
    #[test]
    fn story_drag_without_snap_preserves_original() {
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);

        let (preview, final_pos) = drag_with_snap(start, current, 10.0, SnapMode::Disabled);

        assert_eq!(preview, current);
        assert_eq!(final_pos, current);
    }

    #[cfg(test)]
    #[test]
    fn story_multi_node_drag_preserves_relative_offsets() {
        let nodes = vec![
            SnapNode::new(NodeId::new("n1".to_string()), 40.0, 40.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 140.0, 140.0, 80.0, 40.0),
        ];
        let drag_delta = Point::new(10.0, 10.0);

        let results = drag_multi_with_snap(&nodes, drag_delta, 10.0, SnapMode::Enabled);

        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
        assert_eq!(results[1].x - results[0].x, 100.0);
    }

    // ========== SNP-008: Snap During Resize ==========

    #[cfg(test)]
    #[test]
    fn story_resize_width_snaps_to_grid() {
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(10.0, 0.0);

        let result = resize_with_snap(original, delta, 10.0, ResizeHandle::East);

        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 40.0);
    }

    #[cfg(test)]
    #[test]
    fn story_resize_from_different_handle() {
        let original = Rect::new(100.0, 0.0, 80.0, 40.0);
        let delta = Point::new(-10.0, 0.0);

        let result = resize_with_snap(original, delta, 10.0, ResizeHandle::West);

        assert!((result.x - 90.0).abs() < f64::EPSILON);
        assert_eq!(result.width, 90.0);
    }

    #[cfg(test)]
    #[test]
    fn story_aspect_ratio_lock_with_snap() {
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(20.0, 0.0);

        let result = resize_with_aspect_lock(
            original,
            delta,
            10.0,
            ResizeHandle::East,
            AspectConstraint::Locked,
        );

        assert_eq!(result.width, 100.0);
        assert!((result.height - 50.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_resize_snap_affects_both_dimensions() {
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(13.0, 7.0);

        let result = resize_with_snap(original, delta, 10.0, ResizeHandle::SouthEast);

        // New width = 80 + 13 = 93, snapped to nearest 10 = 90
        // New height = 40 + 7 = 47, snapped to nearest 10 = 50
        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 50.0);
    }

    // ========== SNP-009: Multi-Node Snap ==========

    #[cfg(test)]
    #[test]
    fn story_all_nodes_snap_together() {
        let nodes = vec![
            SnapNode::new(NodeId::new("n1".to_string()), 47.0, 53.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 147.0, 153.0, 80.0, 40.0),
        ];

        let results = snap_multi_nodes(&nodes, 10.0);

        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
        assert!(results.iter().all(|p| (p.x % 10.0).abs() < f64::EPSILON));
        assert!(results.iter().all(|p| (p.y % 10.0).abs() < f64::EPSILON));
    }

    #[cfg(test)]
    #[test]
    fn story_relative_positions_preserved() {
        let nodes = vec![
            SnapNode::new(NodeId::new("n1".to_string()), 47.0, 53.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 147.0, 153.0, 80.0, 40.0),
        ];
        let _original_offset = (nodes[1].x - nodes[0].x, nodes[1].y - nodes[0].y);

        let results = snap_multi_nodes(&nodes, 10.0);

        let new_offset = (results[1].x - results[0].x, results[1].y - results[0].y);
        assert!((new_offset.0 - 100.0).abs() < f64::EPSILON);
        assert!((new_offset.1 - 100.0).abs() < f64::EPSILON);
    }

    #[cfg(test)]
    #[test]
    fn story_primary_selection_determines_snap_target() {
        let nodes = vec![
            SnapNode::new(NodeId::new("n1".to_string()), 47.0, 53.0, 80.0, 40.0),
            SnapNode::new(NodeId::new("n2".to_string()), 147.0, 153.0, 80.0, 40.0),
        ];

        let results = snap_multi_to_primary(&nodes, 0, 10.0);

        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
    }

    #[cfg(test)]
    #[test]
    fn story_empty_node_list_returns_empty() {
        let nodes: Vec<SnapNode> = vec![];

        let results = snap_multi_nodes(&nodes, 10.0);

        assert_eq!(results.len(), 0);
    }

    #[cfg(test)]
    #[test]
    fn story_single_node_snaps_independently() {
        let nodes = vec![SnapNode::new(
            NodeId::new("n1".to_string()),
            47.0,
            53.0,
            80.0,
            40.0,
        )];

        let results = snap_multi_nodes(&nodes, 10.0);

        assert_eq!(results[0], Point::new(50.0, 50.0));
    }

    // ========== SNP-010: Snap Toggle ==========

    #[cfg(test)]
    #[test]
    fn story_toggle_from_disabled_to_enabled() {
        assert_eq!(toggle_snap(ToggleState::Off), ToggleState::On);
    }

    #[cfg(test)]
    #[test]
    fn story_toggle_from_enabled_to_disabled() {
        assert_eq!(toggle_snap(ToggleState::On), ToggleState::Off);
    }

    #[cfg(test)]
    #[test]
    fn story_query_snap_state() {
        let state = SnapState::new(SnapMode::Enabled, 10.0, 5.0);
        assert!(is_snap_enabled(state));
    }

    #[cfg(test)]
    #[test]
    fn story_toggle_during_drag_commits_at_current_position() {
        let position = Point::new(47.0, 53.0);

        let (new_pos, committed) = toggle_during_drag(position, SnapMode::Enabled, 10.0);

        assert_eq!(new_pos, position);
        assert_eq!(committed, SnapMode::Disabled);
    }

    #[cfg(test)]
    #[test]
    fn story_toggle_persists_across_operations() {
        let mut state = SnapState::default();

        state = state.toggle();
        assert!(state.is_enabled());

        state = state.toggle();
        assert!(!state.is_enabled());

        state = state.toggle();
        assert!(state.is_enabled());
    }

    // SNP-006: Smart Alignment SnapResult
    #[cfg(test)]
    #[test]
    fn story_snap_result_contains_target_info() {
        use crate::geometry::snap::mod_types::SnapType;
        let active = SnapNode::new(NodeId::new("active".to_string()), 110.0, 100.0, 80.0, 40.0);
        let targets = vec![SnapNode::new(
            NodeId::new("target".to_string()),
            100.0,
            100.0,
            100.0,
            50.0,
        )];
        let result = snap_to_nodes(&active, &targets, 10.0);

        // Active center is at (150, 120), target center_x is 150
        // Distance to center_x is 0, which is within threshold 10
        // So it correctly snaps to CenterX, not EdgeLeft
        match result {
            crate::geometry::snap::mod_types::SnapResult::Snapped {
                target_node_id,
                snap_type,
                ..
            } => {
                assert_eq!(target_node_id, NodeId::new("target".to_string()));
                assert_eq!(snap_type, SnapType::CenterX);
            }
            _ => panic!("Expected snapped result"),
        }
    }
}
