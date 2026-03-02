bead_id: bd-2ng
bead_title: toolbar: Node alignment tools for selected nodes
phase: p2
updated_at: 2026-03-01T00:00:00Z

# Verification: Node Alignment Tools

## Test Results

```
running 15 tests
test ui::commands::tests::given_empty_selection_when_align_then_returns_false ... ok
test ui::commands::tests::given_alignment_when_successful_then_revision_incremented ... ok
test ui::commands::tests::given_locked_node_when_align_then_skips_locked ... ok
test ui::commands::tests::given_selection_with_infinite_coords_when_align_then_returns_false ... ok
test ui::commands::tests::given_three_nodes_when_align_center_horizontal_then_centered ... ok
test ui::commands::tests::given_three_nodes_when_align_middle_vertical_then_centered ... ok
test ui::commands::tests::given_two_nodes_when_align_left_then_both_share_min_x ... ok
test ui::commands::tests::given_two_nodes_when_align_right_then_both_share_max_right ... ok
test ui::commands::tests::given_alignment_when_successful_then_dimensions_unchanged ... ok
test ui::commands::tests::given_two_nodes_when_align_bottom_then_both_share_max_bottom ... ok
test ui::commands::tests::given_two_nodes_when_align_top_then_both_share_min_y ... ok
test ui::commands::tests::given_single_node_selected_when_align_then_returns_false ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

## Full Test Suite

```
cargo test --package diagram_tool
test result: ok. 971 passed; 0 failed; 5 ignored

Running tests/cli_e2e.rs
test result: ok. 13 passed; 0 failed; 0 ignored
```

## Contract Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Alignment buttons appear when 2+ nodes selected | PASS | `disabled: stats.selected_count < 2` in toolbar.rs |
| Align Left sets all x to min_x | PASS | `given_two_nodes_when_align_left_then_both_share_min_x` |
| Align Right sets all right edges to max_right | PASS | `given_two_nodes_when_align_right_then_both_share_max_right` |
| Align Center H centers nodes | PASS | `given_three_nodes_when_align_center_horizontal_then_centered` |
| Align Top sets all y to min_y | PASS | `given_two_nodes_when_align_top_then_both_share_min_y` |
| Align Bottom sets all bottom edges to max_bottom | PASS | `given_two_nodes_when_align_bottom_then_both_share_max_bottom` |
| Align Middle V centers nodes | PASS | `given_three_nodes_when_align_middle_vertical_then_centered` |
| Single node selection returns false | PASS | `given_single_node_selected_when_align_then_returns_false` |
| Empty selection returns false | PASS | `given_empty_selection_when_align_then_returns_false` |
| Locked nodes are skipped | PASS | `given_locked_node_when_align_then_skips_locked` |
| Non-finite coords return false | PASS | `given_selection_with_infinite_coords_when_align_then_returns_false` |
| Node dimensions unchanged | PASS | `given_alignment_when_successful_then_dimensions_unchanged` |
| Revision incremented | PASS | `given_alignment_when_successful_then_revision_incremented` |
| data-testid attributes present | PASS | All 6 buttons have test IDs |

## Compilation

```
cargo check --package diagram_tool
Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.47s
```

No errors, no new warnings.
