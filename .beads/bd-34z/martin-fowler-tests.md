# Martin Fowler Test Specification: bd-34z - Snap/Alignment Tests

## Test Philosophy

Following Martin Fowler's testing principles:
- **Test behavior, not implementation**
- **One assertion per test concept**
- **Descriptive test names that tell a story**
- **Arrange-Act-Then structure**
- **Test both happy paths and edge cases**

## Test Organization

### Test Categories

1. **Unit Tests**: Individual snap/alignment functions
2. **Integration Tests**: Multi-node operations
3. **Property Tests**: Invariant preservation
4. **Edge Case Tests**: Boundary conditions
5. **Error Path Tests**: Failure modes

## Test Specifications

### Category 1: Grid Snapping (SNP-001)

```rust
mod snp_001_snap_to_grid {
    use super::*;

    #[test]
    fn story_basic_grid_snap_rounds_to_nearest_intersection() {
        // Arrange: Node positioned off-grid
        let position = Point::new(47.0, 53.0);
        let grid_size = 10.0;

        // Act: Snap to grid
        let result = snap_to_grid(position, grid_size);

        // Then: Position rounds to nearest grid intersection
        assert!((result.x - 50.0).abs() < f64::EPSILON);
        assert!((result.y - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn story_node_already_on_grid_stays_unchanged() {
        // Arrange: Node already on grid intersection
        let position = Point::new(50.0, 100.0);
        let grid_size = 10.0;

        // Act: Snap to grid
        let result = snap_to_grid(position, grid_size);

        // Then: Position remains exactly the same
        assert_eq!(result, position);
    }

    #[test]
    fn story_negative_coordinates_snap_correctly() {
        // Arrange: Node in negative coordinate space
        let position = Point::new(-47.0, -53.0);
        let grid_size = 10.0;

        // Act: Snap to grid
        let result = snap_to_grid(position, grid_size);

        // Then: Snaps to nearest grid intersection in negative space
        assert!((result.x - (-50.0)).abs() < f64::EPSILON);
        assert!((result.y - (-50.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn story_half_grid_offset_rounds_up() {
        // Arrange: Node exactly halfway between grid lines
        let position = Point::new(45.0, 45.0);
        let grid_size = 10.0;

        // Act: Snap to grid
        let result = snap_to_grid(position, grid_size);

        // Then: Rounds to nearest (ties round up for positive)
        assert!((result.x - 50.0).abs() < f64::EPSILON);
        assert!((result.y - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn story_invalid_grid_size_returns_original_position() {
        // Arrange: Invalid grid size (zero or negative)
        let position = Point::new(47.0, 53.0);
        let grid_size = 0.0;

        // Act: Attempt snap with invalid grid size
        let result = snap_to_grid(position, grid_size);

        // Then: Original position returned unchanged
        assert_eq!(result, position);
    }

    #[test]
    fn story_nan_coordinates_produce_nan_result() {
        // Arrange: Position with NaN coordinate
        let position = Point::new(f64::NAN, 53.0);
        let grid_size = 10.0;

        // Act: Snap to grid
        let result = snap_to_grid(position, grid_size);

        // Then: Result preserves NaN (handled by caller)
        assert!(result.x.is_nan());
    }
}
```

### Category 2: Guide Snapping (SNP-002)

```rust
mod snp_002_snap_to_guides {
    use super::*;

    #[test]
    fn story_snaps_to_horizontal_guide_within_threshold() {
        // Arrange: Node near horizontal guide line
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(50.0), Guide::Horizontal(100.0)];
        let threshold = 5.0;

        // Act: Snap to guides
        let result = snap_to_guides(position, &guides, threshold);

        // Then: Snaps to nearest guide within threshold
        assert_eq!(result, Some(Point::new(100.0, 50.0)));
    }

    #[test]
    fn story_snaps_to_vertical_guide_within_threshold() {
        // Arrange: Node near vertical guide line
        let position = Point::new(102.0, 100.0);
        let guides = vec![Guide::Vertical(100.0), Guide::Vertical(200.0)];
        let threshold = 5.0;

        // Act: Snap to guides
        let result = snap_to_guides(position, &guides, threshold);

        // Then: Snaps to vertical guide
        assert_eq!(result, Some(Point::new(100.0, 100.0)));
    }

    #[test]
    fn story_position_outside_threshold_returns_none() {
        // Arrange: Node far from any guide
        let position = Point::new(100.0, 60.0);
        let guides = vec![Guide::Horizontal(50.0)];
        let threshold = 5.0;

        // Act: Attempt snap
        let result = snap_to_guides(position, &guides, threshold);

        // Then: No snap applied
        assert_eq!(result, None);
    }

    #[test]
    fn story_multiple_guides_selects_closest() {
        // Arrange: Node between two guides
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(50.0), Guide::Horizontal(55.0)];
        let threshold = 10.0;

        // Act: Snap to guides
        let result = snap_to_guides(position, &guides, threshold);

        // Then: Selects closest guide (52 is 2 from 50, 3 from 55)
        assert_eq!(result, Some(Point::new(100.0, 50.0)));
    }

    #[test]
    fn story_empty_guide_list_returns_none() {
        // Arrange: No guides available
        let position = Point::new(100.0, 52.0);
        let guides: Vec<Guide> = vec![];
        let threshold = 5.0;

        // Act: Attempt snap
        let result = snap_to_guides(position, &guides, threshold);

        // Then: No snap possible
        assert_eq!(result, None);
    }

    #[test]
    fn story_invalid_guide_coordinates_are_filtered() {
        // Arrange: Guides with NaN coordinates
        let position = Point::new(100.0, 52.0);
        let guides = vec![Guide::Horizontal(f64::NAN), Guide::Horizontal(50.0)];
        let threshold = 5.0;

        // Act: Snap to guides
        let result = snap_to_guides(position, &guides, threshold);

        // Then: Invalid guides ignored, valid guides used
        assert_eq!(result, Some(Point::new(100.0, 50.0)));
    }
}
```

### Category 3: Node Snapping (SNP-003)

```rust
mod snp_003_snap_to_nodes {
    use super::*;

    fn make_test_nodes() -> Vec<Node> {
        vec![
            Node::new("n1", 100.0, 100.0, 100.0, 50.0),
            Node::new("n2", 300.0, 100.0, 100.0, 50.0),
            Node::new("n3", 200.0, 200.0, 100.0, 50.0),
        ]
    }

    #[test]
    fn story_snaps_to_left_edge_of_target_node() {
        // Arrange: Active node near left edge of target
        let active = Node::new("active", 110.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let threshold = 10.0;

        // Act: Snap to nodes
        let result = snap_to_nodes(&active, &targets, threshold);

        // Then: Snaps to left edge of target node (100)
        assert_eq!(result, Some(Point::new(100.0, 100.0)));
    }

    #[test]
    fn story_snaps_to_center_of_target_node() {
        // Arrange: Active node near target center
        let active = Node::new("active", 145.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let threshold = 10.0;

        // Act: Snap to nodes
        let result = snap_to_nodes(&active, &targets, threshold);

        // Then: Snaps to center of target (150)
        assert_eq!(result, Some(Point::new(150.0, 100.0)));
    }

    #[test]
    fn story_snaps_to_right_edge_of_target_node() {
        // Arrange: Active node near right edge
        let active = Node::new("active", 188.0, 100.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let threshold = 10.0;

        // Act: Snap to nodes
        let result = snap_to_nodes(&active, &targets, threshold);

        // Then: Snaps to right edge (200)
        assert_eq!(result, Some(Point::new(200.0, 100.0)));
    }

    #[test]
    fn story_snap_fails_when_outside_threshold() {
        // Arrange: Active node far from all targets
        let active = Node::new("active", 150.0, 150.0, 80.0, 40.0);
        let targets = make_test_nodes();
        let threshold = 10.0;

        // Act: Attempt snap
        let result = snap_to_nodes(&active, &targets, threshold);

        // Then: No snap applied
        assert_eq!(result, None);
    }

    #[test]
    fn story_empty_target_list_returns_none() {
        // Arrange: No target nodes
        let active = Node::new("active", 110.0, 100.0, 80.0, 40.0);
        let targets: Vec<Node> = vec![];
        let threshold = 10.0;

        // Act: Attempt snap
        let result = snap_to_nodes(&active, &targets, threshold);

        // Then: No snap possible
        assert_eq!(result, None);
    }

    #[test]
    fn story_selects_closest_snap_target() {
        // Arrange: Multiple targets at different distances
        let active = Node::new("active", 148.0, 100.0, 80.0, 40.0);
        let targets = vec![
            Node::new("n1", 100.0, 100.0, 100.0, 50.0),  // Left edge: 48 away
            Node::new("n2", 150.0, 100.0, 100.0, 50.0),  // Left edge: 2 away
        ];
        let threshold = 50.0;

        // Act: Snap to nodes
        let result = snap_to_nodes(&active, &targets, threshold);

        // Then: Selects closest target
        assert_eq!(result, Some(Point::new(150.0, 100.0)));
    }
}
```

### Category 4: Alignment Tools (SNP-004)

```rust
mod snp_004_alignment_tools {
    use super::*;

    fn make_aligned_nodes() -> Vec<Node> {
        vec![
            Node::new("n1", 0.0, 100.0, 80.0, 40.0),
            Node::new("n2", 50.0, 200.0, 80.0, 40.0),
            Node::new("n3", 100.0, 300.0, 80.0, 40.0),
        ]
    }

    #[test]
    fn story_align_left_moves_all_nodes_to_leftmost_x() {
        // Arrange: Nodes at different X positions
        let nodes = make_aligned_nodes();

        // Act: Align left
        let result = align_left(&nodes);

        // Then: All nodes aligned to X=0
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 0.0);
        assert_eq!(result[2].x, 0.0);
        // Y positions preserved
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[1].y, 200.0);
        assert_eq!(result[2].y, 300.0);
    }

    #[test]
    fn story_align_center_moves_all_nodes_to_average_center() {
        // Arrange: Nodes at different positions
        let nodes = make_aligned_nodes();

        // Act: Align center
        let result = align_center(&nodes);

        // Then: All nodes centered at X=50
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 50.0 - 40.0); // Center minus half width
        assert_eq!(result[1].x, 50.0 - 40.0);
        assert_eq!(result[2].x, 50.0 - 40.0);
    }

    #[test]
    fn story_align_right_moves_all_nodes_to_rightmost_x() {
        // Arrange: Nodes at different positions
        let nodes = make_aligned_nodes();

        // Act: Align right
        let result = align_right(&nodes);

        // Then: All nodes aligned to rightmost
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 100.0 - 80.0); // Right minus width
        assert_eq!(result[1].x, 100.0 - 80.0);
        assert_eq!(result[2].x, 100.0 - 80.0);
    }

    #[test]
    fn story_align_top_moves_all_nodes_to_topmost_y() {
        // Arrange: Nodes at different Y positions
        let nodes = make_aligned_nodes();

        // Act: Align top
        let result = align_top(&nodes);

        // Then: All nodes aligned to Y=100
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[1].y, 100.0);
        assert_eq!(result[2].y, 100.0);
    }

    #[test]
    fn story_align_middle_moves_all_nodes_to_average_middle() {
        // Arrange: Nodes at different Y positions
        let nodes = make_aligned_nodes();

        // Act: Align middle
        let result = align_middle(&nodes);

        // Then: All nodes centered vertically
        assert_eq!(result.len(), 3);
        let avg_y = (100.0 + 200.0 + 300.0) / 3.0;
        assert!((result[0].y - (avg_y - 20.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn story_align_bottom_moves_all_nodes_to_bottommost_y() {
        // Arrange: Nodes at different Y positions
        let nodes = make_aligned_nodes();

        // Act: Align bottom
        let result = align_bottom(&nodes);

        // Then: All nodes aligned to bottom
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 300.0 - 40.0); // Bottom minus height
        assert_eq!(result[1].y, 300.0 - 40.0);
        assert_eq!(result[2].y, 300.0 - 40.0);
    }

    #[test]
    fn story_empty_selection_returns_empty_result() {
        // Arrange: No nodes selected
        let nodes: Vec<Node> = vec![];

        // Act: Attempt alignment
        let result = align_left(&nodes);

        // Then: Empty result returned
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn story_single_node_remains_unchanged() {
        // Arrange: Only one node
        let nodes = vec![Node::new("n1", 50.0, 100.0, 80.0, 40.0)];

        // Act: Align
        let result = align_left(&nodes);

        // Then: Node position unchanged
        assert_eq!(result[0].x, 50.0);
        assert_eq!(result[0].y, 100.0);
    }
}
```

### Category 5: Distribution Tools (SNP-005)

```rust
mod snp_005_distribution_tools {
    use super::*;

    fn make_distributed_nodes() -> Vec<Node> {
        vec![
            Node::new("n1", 0.0, 100.0, 80.0, 40.0),
            Node::new("n2", 50.0, 200.0, 80.0, 40.0),
            Node::new("n3", 100.0, 300.0, 80.0, 40.0),
        ]
    }

    #[test]
    fn story_distribute_horizontally_spaces_nodes_evenly() {
        // Arrange: Nodes at irregular X positions
        let nodes = make_distributed_nodes();

        // Act: Distribute horizontally
        let result = distribute_horizontally(&nodes);

        // Then: Nodes evenly spaced
        assert_eq!(result.len(), 3);
        // Leftmost at 0, rightmost at 100, middle at 50
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[2].x, 100.0);
        assert_eq!(result[1].x, 50.0);
    }

    #[test]
    fn story_distribute_vertically_spaces_nodes_evenly() {
        // Arrange: Nodes at irregular Y positions
        let nodes = make_distributed_nodes();

        // Act: Distribute vertically
        let result = distribute_vertically(&nodes);

        // Then: Nodes evenly spaced vertically
        assert_eq!(result.len(), 3);
        // Top at 100, bottom at 300, middle at 200
        assert_eq!(result[0].y, 100.0);
        assert_eq!(result[2].y, 300.0);
        assert_eq!(result[1].y, 200.0);
    }

    #[test]
    fn story_fewer_than_three_nodes_returns_error() {
        // Arrange: Only two nodes
        let nodes = vec![
            Node::new("n1", 0.0, 100.0, 80.0, 40.0),
            Node::new("n2", 50.0, 200.0, 80.0, 40.0),
        ];

        // Act: Attempt distribution
        let result = distribute_horizontally(&nodes);

        // Then: Error returned
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SnapError::InsufficientNodesForDistribution));
    }

    #[test]
    fn story_distribution_maintains_node_order() {
        // Arrange: Nodes in specific order
        let nodes = vec![
            Node::new("n3", 100.0, 300.0, 80.0, 40.0),
            Node::new("n1", 0.0, 100.0, 80.0, 40.0),
            Node::new("n2", 50.0, 200.0, 80.0, 40.0),
        ];

        // Act: Distribute (sorts by position)
        let result = distribute_horizontally(&nodes).unwrap();

        // Then: Order maintained by original Y
        assert_eq!(result[0].y, 100.0); // Bottom
        assert_eq!(result[1].y, 200.0); // Middle
        assert_eq!(result[2].y, 300.0); // Top
    }

    #[test]
    fn story_distribution_preserves_first_and_last_positions() {
        // Arrange: Nodes with specific first and last
        let nodes = make_distributed_nodes();

        // Act: Distribute
        let result = distribute_horizontally(&nodes).unwrap();

        // Then: First and last positions unchanged
        assert_eq!(result[0].x, nodes[0].x);
        assert_eq!(result[2].x, nodes[2].x);
    }
}
```

### Category 6: Snap Threshold (SNP-006)

```rust
mod snp_006_snap_threshold {
    use super::*;

    #[test]
    fn story_snap_applies_when_distance_within_threshold() {
        // Arrange: Position 5 units from target
        let distance = 5.0;
        let threshold = 10.0;

        // Act: Check if should snap
        let result = should_snap(distance, threshold);

        // Then: Snap applies
        assert_eq!(result, true);
    }

    #[test]
    fn story_snap_applies_when_exactly_at_threshold() {
        // Arrange: Distance exactly equals threshold
        let distance = 10.0;
        let threshold = 10.0;

        // Act: Check if should snap
        let result = should_snap(distance, threshold);

        // Then: Snap applies
        assert_eq!(result, true);
    }

    #[test]
    fn story_snap_does_not_apply_when_outside_threshold() {
        // Arrange: Position 11 units from target
        let distance = 11.0;
        let threshold = 10.0;

        // Act: Check if should snap
        let result = should_snap(distance, threshold);

        // Then: No snap
        assert_eq!(result, false);
    }

    #[test]
    fn story_zero_threshold_only_snaps_exact_matches() {
        // Arrange: Zero threshold
        let distance = 0.0;
        let threshold = 0.0;

        // Act: Check if should snap
        let result = should_snap(distance, threshold);

        // Then: Only exact matches snap
        assert_eq!(result, true);
    }

    #[test]
    fn story_negative_threshold_treated_as_zero() {
        // Arrange: Invalid negative threshold
        let distance = 5.0;
        let threshold = -1.0;

        // Act: Check if should snap
        let result = should_snap(distance, threshold);

        // Then: Treated as zero threshold
        assert_eq!(result, false);
    }

    #[test]
    fn story_infinity_threshold_always_snaps() {
        // Arrange: Infinite threshold
        let distance = 1000.0;
        let threshold = f64::INFINITY;

        // Act: Check if should snap
        let result = should_snap(distance, threshold);

        // Then: Always snaps
        assert_eq!(result, true);
    }
}
```

### Category 7: Snap During Drag (SNP-007)

```rust
mod snp_007_snap_during_drag {
    use super::*;

    #[test]
    fn story_drag_with_snap_updates_preview_and_final() {
        // Arrange: Drag operation with snap enabled
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);
        let grid_size = 10.0;
        let snap_enabled = true;

        // Act: Calculate drag with snap
        let (preview, final) = drag_with_snap(start, current, grid_size, snap_enabled);

        // Then: Preview and final both snapped
        assert_eq!(preview, Point::new(50.0, 50.0));
        assert_eq!(final, Point::new(50.0, 50.0));
    }

    #[test]
    fn story_drag_without_snap_preserves_original() {
        // Arrange: Drag with snap disabled
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);
        let grid_size = 10.0;
        let snap_enabled = false;

        // Act: Calculate drag
        let (preview, final) = drag_with_snap(start, current, grid_size, snap_enabled);

        // Then: No snap applied
        assert_eq!(preview, current);
        assert_eq!(final, current);
    }

    #[test]
    fn story_multi_node_drag_preserves_relative_offsets() {
        // Arrange: Multiple nodes being dragged
        let nodes = vec![
            Node::new("n1", 40.0, 40.0, 80.0, 40.0),
            Node::new("n2", 140.0, 140.0, 80.0, 40.0),
        ];
        let drag_delta = Point::new(10.0, 10.0);
        let grid_size = 10.0;

        // Act: Drag multiple nodes with snap
        let results = drag_multi_with_snap(&nodes, drag_delta, grid_size, true);

        // Then: All nodes snap, offsets preserved
        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
        assert_eq!(results[1].x - results[0].x, 100.0); // Offset preserved
    }

    #[test]
    fn story_drag_snap_toggle_mid_drag_applies_immediately() {
        // Arrange: Drag in progress, snap toggled
        let start = Point::new(40.0, 40.0);
        let current = Point::new(47.0, 53.0);
        let grid_size = 10.0;

        // Act: Toggle snap during drag
        let (preview_before, final_before) = drag_with_snap(start, current, grid_size, false);
        let (preview_after, final_after) = drag_with_snap(start, current, grid_size, true);

        // Then: Snap applies immediately
        assert_eq!(preview_before, current);
        assert_eq!(preview_after, Point::new(50.0, 50.0));
    }
}
```

### Category 8: Snap During Resize (SNP-008)

```rust
mod snp_008_snap_during_resize {
    use super::*;

    #[test]
    fn story_resize_width_snaps_to_grid() {
        // Arrange: Resize operation
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(10.0, 0.0);
        let grid_size = 10.0;
        let handle = ResizeHandle::East;

        // Act: Resize with snap
        let result = resize_with_snap(original, delta, grid_size, handle);

        // Then: Width snaps to grid
        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 40.0);
    }

    #[test]
    fn story_resize_from_different_handle() {
        // Arrange: Resize from west handle
        let original = Rect::new(100.0, 0.0, 80.0, 40.0);
        let delta = Point::new(-10.0, 0.0);
        let grid_size = 10.0;
        let handle = ResizeHandle::West;

        // Act: Resize with snap
        let result = resize_with_snap(original, delta, grid_size, handle);

        // Then: Width adjusts from left
        assert_eq!(result.x, 90.0);
        assert_eq!(result.width, 90.0);
    }

    #[test]
    fn story_aspect_ratio_lock_with_snap() {
        // Arrange: Resize with aspect ratio locked
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(20.0, 0.0);
        let grid_size = 10.0;
        let handle = ResizeHandle::East;
        let lock_aspect = true;

        // Act: Resize with snap and aspect lock
        let result = resize_with_aspect_lock(original, delta, grid_size, handle, lock_aspect);

        // Then: Aspect ratio maintained, snapped
        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 45.0); // Half of width
    }

    #[test]
    fn story_resize_snap_affects_both_dimensions() {
        // Arrange: Corner resize
        let original = Rect::new(0.0, 0.0, 80.0, 40.0);
        let delta = Point::new(13.0, 7.0);
        let grid_size = 10.0;
        let handle = ResizeHandle::SouthEast;

        // Act: Resize corner with snap
        let result = resize_with_snap(original, delta, grid_size, handle);

        // Then: Both dimensions snap
        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 50.0);
    }
}
```

### Category 9: Multi-Node Snap (SNP-009)

```rust
mod snp_009_multi_node_snap {
    use super::*;

    #[test]
    fn story_all_nodes_snap_together() {
        // Arrange: Multiple selected nodes
        let nodes = vec![
            Node::new("n1", 47.0, 53.0, 80.0, 40.0),
            Node::new("n2", 147.0, 153.0, 80.0, 40.0),
        ];
        let grid_size = 10.0;

        // Act: Snap all nodes
        let results = snap_multi_nodes(&nodes, grid_size);

        // Then: All nodes snapped
        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
        assert!(results.iter().all(|p| p.x % 10.0 == 0.0));
        assert!(results.iter().all(|p| p.y % 10.0 == 0.0));
    }

    #[test]
    fn story_relative_positions_preserved() {
        // Arrange: Nodes with specific offsets
        let nodes = vec![
            Node::new("n1", 47.0, 53.0, 80.0, 40.0),
            Node::new("n2", 147.0, 153.0, 80.0, 40.0),
        ];
        let original_offset = (
            nodes[1].x - nodes[0].x,
            nodes[1].y - nodes[0].y,
        );

        // Act: Snap all nodes
        let results = snap_multi_nodes(&nodes, 10.0);

        // Then: Offsets preserved
        let new_offset = (results[1].x - results[0].x, results[1].y - results[0].y);
        assert_eq!(new_offset.0, 100.0);
        assert_eq!(new_offset.1, 100.0);
    }

    #[test]
    fn story_primary_selection_determines_snap_target() {
        // Arrange: Nodes with primary selected
        let nodes = vec![
            Node::new("n1", 47.0, 53.0, 80.0, 40.0),
            Node::new("n2", 147.0, 153.0, 80.0, 40.0),
        ];
        let primary_index = 0;

        // Act: Snap to primary
        let results = snap_multi_to_primary(&nodes, primary_index, 10.0);

        // Then: Primary snaps, others follow
        assert_eq!(results[0], Point::new(50.0, 50.0));
        assert_eq!(results[1], Point::new(150.0, 150.0));
    }

    #[test]
    fn story_empty_node_list_returns_empty() {
        // Arrange: No nodes
        let nodes: Vec<Node> = vec![];

        // Act: Snap
        let results = snap_multi_nodes(&nodes, 10.0);

        // Then: Empty result
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn story_single_node_snaps_independently() {
        // Arrange: Single node
        let nodes = vec![Node::new("n1", 47.0, 53.0, 80.0, 40.0)];

        // Act: Snap
        let results = snap_multi_nodes(&nodes, 10.0);

        // Then: Snaps to grid
        assert_eq!(results[0], Point::new(50.0, 50.0));
    }
}
```

### Category 10: Snap Toggle (SNP-010)

```rust
mod snp_010_snap_toggle {
    use super::*;

    #[test]
    fn story_toggle_from_disabled_to_enabled() {
        // Arrange: Snap currently disabled
        let state = false;

        // Act: Toggle
        let new_state = toggle_snap(state);

        // Then: Snap enabled
        assert_eq!(new_state, true);
    }

    #[test]
    fn story_toggle_from_enabled_to_disabled() {
        // Arrange: Snap currently enabled
        let state = true;

        // Act: Toggle
        let new_state = toggle_snap(state);

        // Then: Snap disabled
        assert_eq!(new_state, false);
    }

    #[test]
    fn story_query_snap_state() {
        // Arrange: Snap state
        let state = true;

        // Act: Query
        let is_enabled = is_snap_enabled(state);

        // Then: Returns current state
        assert_eq!(is_enabled, true);
    }

    #[test]
    fn story_toggle_during_drag_commits_at_current_position() {
        // Arrange: Drag in progress at unsnapped position
        let position = Point::new(47.0, 53.0);
        let snap_was_enabled = true;

        // Act: Toggle snap off during drag
        let (new_pos, committed) = toggle_during_drag(position, snap_was_enabled, 10.0);

        // Then: Position committed as-is
        assert_eq!(new_pos, position);
        assert_eq!(committed, false);
    }

    #[test]
    fn story_toggle_persists_across_operations() {
        // Arrange: Initial state
        let mut state = SnapState::default();

        // Act: Toggle multiple times
        state = state.toggle();
        assert_eq!(state.enabled, true);

        state = state.toggle();
        assert_eq!(state.enabled, false);

        state = state.toggle();
        assert_eq!(state.enabled, true);
    }
}
```

## Property-Based Tests

```rust
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_snap_to_grid_preserves_modulo(x in -1000.0..1000.0, y in -1000.0..1000.0, grid in 1.0..100.0) {
            let point = Point::new(x, y);
            let snapped = snap_to_grid(point, grid);

            // Snapped coordinates are multiples of grid size
            prop_assert!((snapped.x / grid).fract().abs() < f64::EPSILON);
            prop_assert!((snapped.y / grid).fract().abs() < f64::EPSILON);
        }

        #[test]
        fn prop_snap_within_threshold_never_exceeds_threshold(pos in 0.0..1000.0, target in 0.0..1000.0, threshold in 0.0..100.0) {
            let distance = (pos - target).abs();
            let should_snap = should_snap(distance, threshold);

            if should_snap {
                prop_assert!(distance <= threshold);
            }
        }

        #[test]
        fn prop_align_preserves_node_count(nodes in prop::collection::vec(node_strategy(), 1..10)) {
            let aligned = align_left(&nodes);
            prop_assert_eq!(aligned.len(), nodes.len());
        }
    }
}
```

## Integration Tests

```rust
mod integration_tests {
    use super::*;

    #[test]
    fn story_complete_snap_workflow() {
        // Arrange: Document with multiple nodes
        let mut doc = DiagramDocument::default();
        doc.add_node(Node::new("n1", 47.0, 53.0, 80.0, 40.0));
        doc.add_node(Node::new("n2", 147.0, 153.0, 80.0, 40.0));

        // Act: Snap all nodes to grid
        let snapped = snap_document_to_grid(&doc, 10.0);

        // Then: All nodes snapped
        assert_eq!(snapped.get_node("n1").unwrap().x, 50.0);
        assert_eq!(snapped.get_node("n2").unwrap().x, 150.0);
    }

    #[test]
    fn story_alignment_then_distribution() {
        // Arrange: Nodes at random positions
        let nodes = vec![
            Node::new("n1", 23.0, 47.0, 80.0, 40.0),
            Node::new("n2", 123.0, 147.0, 80.0, 40.0),
            Node::new("n3", 223.0, 247.0, 80.0, 40.0),
        ];

        // Act: Align left then distribute
        let aligned = align_left(&nodes);
        let distributed = distribute_horizontally(&aligned).unwrap();

        // Then: Aligned and evenly spaced
        assert_eq!(distributed[0].x, 0.0);
        assert_eq!(distributed[1].x, 100.0);
        assert_eq!(distributed[2].x, 200.0);
    }
}
```

## Test Smells to Avoid

1. **Brittle tests**: Don't test exact floating-point comparisons, use epsilon
2. **Implementation testing**: Test behavior, not how it's achieved
3. **Magic numbers**: Use named constants for thresholds, grid sizes
4. **Test logic duplication**: Don't re-implement production logic in tests
5. **Over-mocking**: Test real behavior, not mocked components
6. **Greedy tests**: One assertion per conceptual test
7. **Mystery guests**: Test data should be obvious from reading the test
