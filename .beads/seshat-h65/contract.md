# Contract Specification

## Context
- Feature: Group Scale (MUL-011 to MUL-015)
- Domain terms:
  - **Group Scale**: Scaling a set of multiple selected nodes around an anchor point (usually their collective bounding box center).
  - **Scale Factor**: A positive, non-zero multiplier applied to both node dimensions and their offsets from the anchor.
  - **Minimum Dimension**: The smallest allowed width/height for a node (e.g., 1.0).
  - **Anchor**: The fixed point around which the scale transformation occurs.
- Assumptions:
  - Text nodes might scale their bounding boxes but re-wrap text, or they might scale font size (assuming geometric scale for now).
  - All items in the selection must exist and be unlocked to perform a group scale.
- Open questions:
  - Should edges/lines scale their exact control points, or just their start/end anchors? (Assuming start/end anchors scale relative to the group anchor).

## Preconditions
- [P1] The `selection` slice must not be empty.
- [P2] `scale_factor` must be strictly greater than 0.0.
- [P3] All `NodeId`s in the `selection` must exist in the `Subgraph`.
- [P4] None of the nodes in the `selection` may be locked.
- [P5] The resulting scaled dimensions and coordinates must remain finite and within global canvas bounds (`MAX_DIMENSION` / `MAX_COORDINATE`).

## Postconditions
- [Q1] The relative distances between the centers of any two selected nodes are scaled by exactly `scale_factor`.
- [Q2] The dimensions (width/height) of resizable nodes in the selection are multiplied by `scale_factor`, clamped to `MIN_DIMENSION`.
- [Q3] Unselected nodes in the `Subgraph` are strictly not mutated.
- [Q4] Applying `scale_factor` then `1.0 / scale_factor` (inverse) returns all nodes to their original positions and dimensions within `epsilon` (< 1e-6 drift).

## Invariants
- [I1] The total number of nodes in the `Subgraph` remains unchanged.
- [I2] The logical hierarchy and node types remain unchanged.

## Error Taxonomy
- `GroupTransformError::EmptySelection` - when `selection` is empty.
- `GroupTransformError::NodeNotFound(NodeId)` - when a provided ID does not exist in the subgraph.
- `GroupTransformError::NodeLocked(NodeId)` - when attempting to scale a locked node.
- `GroupTransformError::OutOfBounds` - when scaling would produce non-finite coordinates or exceed canvas bounds.

## Contract Signatures
- `pub fn scale_group(subgraph: &mut Subgraph, selection: &[NodeId], scale_factor: PositiveScale, anchor: Point) -> Result<(), GroupTransformError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| [P1] selection not empty | Runtime (Result) | `if selection.is_empty() { return Err(EmptySelection); }` |
| [P2] scale > 0.0 | Compile-time (strongest) | `PositiveScale` newtype (guarantees > 0.0 internally) |
| [P3] nodes exist | Runtime (Result) | Checked during iteration, `Err(NodeNotFound)` |
| [P4] nodes unlocked | Runtime (Result) | Checked via node state, `Err(NodeLocked)` |
| [P5] within bounds | Runtime (Result) | Checked post-calculation, `Err(OutOfBounds)` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES [P1]: `scale_group(&mut subgraph, &[], scale, anchor)` -- should produce `Err(GroupTransformError::EmptySelection)`
- VIOLATES [P2]: Prevented at compile time by requiring `PositiveScale::try_new(0.0).unwrap()`, which fails before calling the function. If using f64: `scale_group(..., 0.0, ...)` produces type error or initialization error.
- VIOLATES [P3]: `scale_group(&mut subgraph, &[missing_id], scale, anchor)` -- should produce `Err(GroupTransformError::NodeNotFound(missing_id))`
- VIOLATES [P4]: `scale_group(&mut subgraph, &[locked_id], scale, anchor)` -- should produce `Err(GroupTransformError::NodeLocked(locked_id))`
- VIOLATES [P5]: `scale_group(&mut subgraph, &[id], huge_scale, anchor)` -- should produce `Err(GroupTransformError::OutOfBounds)`
- VIOLATES [Q1]: `validate_relative_distances(subgraph, initial_state)` after scaling returns false due to coordinate overflow -- should produce `Err(GroupTransformError::OutOfBounds)`
- VIOLATES [Q2]: Node would scale below `MIN_DIMENSION` but clamping fails (tested by asserting dimensions >= MIN_DIMENSION). If validation fails, could conceptually `Err(GroupTransformError::OutOfBounds)`.
- VIOLATES [Q3]: Unselected node `unselected_id` position is modified -- test assertion fails. 

## Ownership Contracts (Rust-specific)
- Exclusive borrow: `fn scale_group(subgraph: &mut Subgraph, ...)` -- Mutates the `position`, `width`, and `height` fields of only the nodes specified in the `selection` slice. 
- Shared borrow: `selection: &[NodeId]` -- Read-only reference to the list of nodes to scale. No ownership taken.
- Value types: `scale_factor: PositiveScale` and `anchor: Point` -- Copied into the function. No cloning of heap data required.

## Non-goals
- Scaling nested sub-containers automatically (unless explicitly included in the `selection`).
- Path-level morphological scaling of complex SVG paths (only bounding box / rect scaling is considered here).
