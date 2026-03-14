# Implementation Summary - seshat-idr

bead_id: seshat-idr
bead_title: SUB-023 to SUB-027: Subgraph relative coordinates
phase: p2
updated_at: 2026-03-14T18:30:00Z

## Changes Applied

### 1. Model Core (`diagram_tool/src/models/document.rs`)
- Added `Node::get_world_coords_im(&self, nodes: &im::HashMap<NodeId, Node>) -> Result<(f64, f64), String>`
- This recursively calculates the absolute position of any node by summing relative offsets up the parent chain.

### 2. Reparenting Logic (`diagram_tool/src/models/subgraph/reparenting.rs`)
- Introduced `set_node_parent_ext` and `unparent_node_ext` with `keep_world_pos: bool` flag.
- When `keep_world_pos` is true (default for public API), it calculates the new relative coordinates such that the node stays in the same global position after changing parents.

### 3. Subgraph Types (`diagram_tool/src/models/subgraph/types.rs`)
- Added `calculate_container_bounds_from_ids` which uses world coordinates for all children to calculate a global bounding box.
- Preserved `calculate_container_bounds` for cases where nodes are already in the same coordinate space.

### 4. Grouping Operations (`diagram_tool/src/models/subgraph/grouping.rs`)
- Updated `group_nodes` to use world-space bounds when creating a new container.
- Updated `ungroup_nodes` to use `unparent_node_ext` or `set_node_parent_ext` to ensure children don't jump when their container is removed.

### 5. Verification
- Created `diagram_tool/src/models/subgraph_relative_tests.rs` with explicit test cases for SUB-023 through SUB-027.
- Verified all tests pass.
