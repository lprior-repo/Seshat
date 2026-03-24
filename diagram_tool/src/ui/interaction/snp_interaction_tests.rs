#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::module_inception,
    clippy::let_unit_value,
    clippy::redundant_pattern_matching,
    unused_variables,
    unused_imports
)]
#[cfg(kani)]
#[kani::proof]
fn given_multi_select_drag_when_snap_enabled_then_all_nodes_use_snapped_delta() {
    let grid = GridSize::new(20.0).unwrap();

    // Three nodes at different positions
    let originals = HashMap::new()
        .update(NodeId::new("a".to_string()), (10.0, 15.0))
        .update(NodeId::new("b".to_string()), (100.0, 200.0))
        .update(NodeId::new("c".to_string()), (-50.0, 75.0));

    // Drag by (14.0, 26.0) - should snap to (20.0, 20.0) with grid 20.0
    // 14/20 = 0.7 -> rounds to 1 -> 20
    // 26/20 = 1.3 -> rounds to 1 -> 20
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), true, grid);

    // All nodes should have moved by (20.0, 20.0) - the snapped delta
    let pos_a = updated.get(&NodeId::new("a".to_string())).copied();
    let pos_b = updated.get(&NodeId::new("b".to_string())).copied();
    let pos_c = updated.get(&NodeId::new("c".to_string())).copied();

    assert_eq!(
        pos_a,
        Some((30.0, 35.0)),
        "Node a should be at (10+20, 15+20)"
    );
    assert_eq!(
        pos_b,
        Some((120.0, 220.0)),
        "Node b should be at (100+20, 200+20)"
    );
    assert_eq!(
        pos_c,
        Some((-30.0, 95.0)),
        "Node c should be at (-50+20, 75+20)"
    );
}

#[cfg(kani)]
#[kani::proof]
fn given_multi_select_drag_when_snap_disabled_then_all_nodes_use_raw_delta() {
    let grid = GridSize::new(20.0).unwrap();

    let originals = HashMap::new()
        .update(NodeId::new("a".to_string()), (10.0, 15.0))
        .update(NodeId::new("b".to_string()), (100.0, 200.0));

    // Drag by (14.0, 26.0) - no snap, should use raw delta
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (14.0, 26.0), false, grid);

    let pos_a = updated.get(&NodeId::new("a".to_string())).copied();
    let pos_b = updated.get(&NodeId::new("b".to_string())).copied();

    assert_eq!(
        pos_a,
        Some((24.0, 41.0)),
        "Node a should be at (10+14, 15+26)"
    );
    assert_eq!(
        pos_b,
        Some((114.0, 226.0)),
        "Node b should be at (100+14, 200+26)"
    );
}

#[cfg(kani)]
#[kani::proof]
fn given_multi_select_drag_from_nonzero_anchor_when_snap_enabled_then_snaps_correctly() {
    let grid = GridSize::new(20.0).unwrap();

    let originals = HashMap::new()
        .update(NodeId::new("a".to_string()), (50.0, 60.0))
        .update(NodeId::new("b".to_string()), (150.0, 160.0));

    // Anchor at (100, 100), current at (115, 128) -> delta (15, 28)
    // Delta (15, 28) with grid 20.0:
    // 15/20 = 0.75 -> rounds to 1 -> 20
    // 28/20 = 1.4 -> rounds to 1 -> 20
    let updated =
        dragged_positions_with_snap(&originals, (100.0, 100.0), (115.0, 128.0), true, grid);

    let pos_a = updated.get(&NodeId::new("a".to_string())).copied();
    let pos_b = updated.get(&NodeId::new("b".to_string())).copied();

    assert_eq!(
        pos_a,
        Some((70.0, 80.0)),
        "Node a should be at (50+20, 60+20)"
    );
    assert_eq!(
        pos_b,
        Some((170.0, 180.0)),
        "Node b should be at (150+20, 160+20)"
    );
}

#[cfg(kani)]
#[kani::proof]
fn given_single_node_drag_when_snap_enabled_then_position_snapped() {
    let grid = GridSize::new(20.0).unwrap();

    let originals = HashMap::new().update(NodeId::new("single".to_string()), (0.0, 0.0));

    // Small drag that crosses snap threshold
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (11.0, 9.0), true, grid);

    let pos = updated.get(&NodeId::new("single".to_string())).copied();
    // Delta (11, 9) snaps to (20, 0) with grid 20.0
    assert_eq!(pos, Some((20.0, 0.0)));
}

#[cfg(kani)]
#[kani::proof]
fn given_negative_drag_when_snap_enabled_then_snaps_to_negative_grid() {
    let grid = GridSize::new(20.0).unwrap();

    let originals = HashMap::new().update(NodeId::new("a".to_string()), (100.0, 100.0));

    // Negative drag
    // -15/20 = -0.75 -> rounds to -1 -> -20
    // -25/20 = -1.25 -> rounds to -1 -> -20
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (-15.0, -25.0), true, grid);

    let pos = updated.get(&NodeId::new("a".to_string())).copied();
    // Delta (-15, -25) snaps to (-20, -20) with grid 20.0
    assert_eq!(pos, Some((80.0, 80.0)));
}

#[cfg(kani)]
#[kani::proof]
fn given_drag_threshold_boundary_when_checked_then_engages_correctly() {
    // DRAG_THRESHOLD_PX is 3.0
    // Test just below threshold
    assert!(!has_drag_threshold((0.0, 0.0), (2.9, 0.0)));
    assert!(!has_drag_threshold((0.0, 0.0), (0.0, 2.9)));
    assert!(!has_drag_threshold((0.0, 0.0), (2.0, 2.0))); // sqrt(8) ≈ 2.83

    // Test at threshold
    assert!(has_drag_threshold((0.0, 0.0), (3.0, 0.0)));
    assert!(has_drag_threshold((0.0, 0.0), (0.0, 3.0)));

    // Test just above threshold
    assert!(has_drag_threshold((0.0, 0.0), (3.1, 0.0)));
    assert!(has_drag_threshold((0.0, 0.0), (0.0, 3.1)));
}

#[cfg(kani)]
#[kani::proof]
fn given_diagonal_drag_when_threshold_checked_then_uses_euclidean_distance() {
    // Diagonal distance should be Euclidean
    // sqrt(3^2 + 3^2) = sqrt(18) ≈ 4.24 > 3.0
    assert!(has_drag_threshold((0.0, 0.0), (3.0, 3.0)));

    // sqrt(2^2 + 2^2) = sqrt(8) ≈ 2.83 < 3.0
    assert!(!has_drag_threshold((0.0, 0.0), (2.0, 2.0)));

    // sqrt(2^2 + 3^2) = sqrt(13) ≈ 3.61 > 3.0
    assert!(has_drag_threshold((0.0, 0.0), (2.0, 3.0)));
}

#[cfg(kani)]
#[kani::proof]
fn given_empty_selection_when_dragged_then_returns_empty() {
    let grid = GridSize::new(20.0).unwrap();
    let originals = HashMap::new();

    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (100.0, 100.0), true, grid);

    assert!(updated.is_empty());
}

#[cfg(kani)]
#[kani::proof]
fn given_large_multi_select_when_snap_enabled_then_all_processed() {
    let grid = GridSize::new(20.0).unwrap();

    // Create many nodes
    let mut originals = HashMap::new();
    for i in 0..100 {
        let x = f64::from(i) * 10.0;
        let y = f64::from(i) * 5.0;
        originals = originals.update(NodeId::new(format!("node-{}", i)), (x, y));
    }

    // Delta (15, 25) with grid 20.0:
    // 15/20 = 0.75 -> rounds to 1 -> 20
    // 25/20 = 1.25 -> rounds to 1 -> 20
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (15.0, 25.0), true, grid);

    // All nodes should be present
    assert_eq!(updated.len(), 100);

    // Delta (15, 25) snaps to (20, 20)
    for i in 0..100 {
        let id = NodeId::new(format!("node-{}", i));
        let expected_x = f64::from(i) * 10.0 + 20.0;
        let expected_y = f64::from(i) * 5.0 + 20.0;
        let pos = updated.get(&id).copied();
        assert_eq!(pos, Some((expected_x, expected_y)));
    }
}

#[cfg(kani)]
#[kani::proof]
fn given_different_grid_sizes_when_snap_enabled_then_snaps_to_correct_grid() {
    // Test with minimum grid size
    let small_grid = GridSize::new(10.0).unwrap();
    let originals = HashMap::new().update(NodeId::new("a".to_string()), (0.0, 0.0));
    let updated = dragged_positions_with_snap(&originals, (0.0, 0.0), (6.0, 4.0), true, small_grid);
    let pos = updated.get(&NodeId::new("a".to_string())).copied();
    assert_eq!(pos, Some((10.0, 0.0))); // Delta (6, 4) snaps to (10, 0) with grid 10

    // Test with maximum grid size
    let large_grid = GridSize::new(100.0).unwrap();
    let originals = HashMap::new().update(NodeId::new("b".to_string()), (0.0, 0.0));
    let updated =
        dragged_positions_with_snap(&originals, (0.0, 0.0), (55.0, 45.0), true, large_grid);
    let pos = updated.get(&NodeId::new("b".to_string())).copied();
    assert_eq!(pos, Some((100.0, 0.0))); // Delta (55, 45) snaps to (100, 0) with grid 100
}
