# Subgraph Save-Reload Stability Research Notes

## Overview
This document captures the key findings about subgraph operations that must be preserved during save-reload cycles.

## Subgraph Data Model

### Node Structure
- Subgraphs are stored as `Node` with `kind: NodeKind::Subgraph`
- Key fields for subgraph persistence:
  - `kind`: `NodeKind::Subgraph`
  - `x`, `y`, `width`, `height`: Position and dimensions
  - `parent`: `Option<NodeId>` - parent subgraph (for nested subgraphs)
  - `locked`: Always `true` for subgraphs
  - `style`: `Some(NodeStyle::Box)` 
  - `collapsed`: `Some(bool)` - visibility state

### Parent-Child Relationships
- Child nodes reference parent subgraphs via `parent: Option<NodeId>`
- Valid parent must be a subgraph node
- Schema validation enforces parent-child rules (see `models/schema.rs`)
- Circular parent chains are detected and rejected

## Persistence Mechanism

### Auto-Save to localStorage
- Module: `ui/toolbar/auto_save.rs`
- Key: `"diagram_tool.autosave"`
- Data structure: `AutoSavedDiagram` containing:
  - `version`: Schema version (currently 1)
  - `document`: `DiagramDocument`
  - `tool_mode`: Current tool mode string
  - `edge_style`, `arrow_type`: Editor preferences

### Import/Export
- File import uses `persistence_compat.rs` for JSON parsing
- Revision is reset to `Revision::INITIAL` on import
- Undo history is preserved after failed imports

## Operations That Must Be Preserved

### 1. Subgraph Creation
- Creating a subgraph with selected nodes
- Subgraph bounds must match selection rectangle
- Child nodes must have parent reference set

### 2. Subgraph Resize (Proportional Transform)
- When resizing a subgraph:
  - Subgraph dimensions change
  - All direct child nodes must be proportionally repositioned/resized
  - Nested subgraphs must maintain relative proportions
- Canvas code (`canvas.rs`): `scale_selected_nodes` function handles this
- Resize logic in `interaction_reducer.rs`: `resize_target_ids` includes geometrically contained nodes

### 3. Nested Subgraph Hierarchy
- Inner subgraph must have parent reference to outer subgraph
- Resizing outer should proportionally resize inner
- Example from `scene_nested_subgraph_v1.json`:
  - `outer` (x:80, y:80, w:760, h:480, z_index:-2)
  - `inner` (x:180, y:180, w:420, h:240, parent:"outer", z_index:-1)
  - `t1`, `t2` text nodes with parent:"inner"

### 4. Node-Subgraph Relationships
- Moving nodes into/out of subgraphs
- Parent field must be correctly serialized/deserialized
- Child node positions are in absolute coordinates (not relative to parent)

## Potential Regression Areas

### 1. Parent Field Serialization
- Ensure `Option<NodeId>` serializes as `null` or the ID string
- Check deserialization handles missing/null parent correctly

### 2. Proportional Transform Integrity
- When resizing, child nodes must maintain relative position/size
- Test that `(node.x - sub.x) / sub.width` ratio is preserved after reload

### 3. Nested Subgraph Hierarchy
- Deep nesting must survive round-trip
- Parent references must not be lost or corrupted

### 4. Subgraph Bounds
- Minimum size constraints (20x20 grid-based)
- Negative width/height rejected by schema

## Test Strategy
Created E2E tests (`diagram.subgraph-save-reload.spec.ts`) and unit tests (`models/subgraph_persistence_tests.rs`) that:
1. Create subgraphs with various configurations
2. Perform resize operations
3. Trigger save (auto-save or manual export)
4. Reload page/import saved document
5. Verify structural integrity and proportional relationships

## Bug Fixes Found

### 1. scene_nested_subgraph_v1.json - Invalid Field Name
- **Issue**: The test fixture file had `font_size` instead of `fontSize` in the edge object
- **Fix**: Changed `"font_size": null` to `"fontSize": null` in the JSON file
- **Impact**: This was causing the document parsing to fail when loading nested subgraph scenes
