# Contract Specification

## Context
- Feature: Diagram Canvas Domain Logic (`canvas_domain`)
- Domain terms:
  - Canvas: The infinite 2D space where nodes exist.
  - Viewport: The visible window into the canvas, determined by zoom and pan.
  - BoundingBox: A rectangular region `(top_left, bottom_right)`.
  - SnapGrid: A discrete coordinate system for aligning node positions/dimensions.
  - SelectionSet: The set of currently selected nodes.
  - Reparenting: Moving a node to become a child of another node, affecting local coordinates.
- Assumptions:
  - Canvas and viewport coordinates are stored as floating point values (`f64`).
  - The hierarchy is a directed acyclic graph (DAG) - a forest of trees.
- Open questions:
  - Do we allow zero-width or zero-height bounding boxes, or enforce a strict minimum size? (Assuming strict `width > 0` and `height > 0`).

## Preconditions
- [P1] `node_exists`: When translating or resizing, the target `NodeId` must exist in the canvas.
- [P2] `no_cycles`: When reparenting, the `target_parent_id` must exist and cannot be the node itself or one of its descendants.
- [P3] `valid_grid`: Grid snap resolution must be a strictly positive number (`> 0.0`).
- [P4] `valid_resize`: Resize dimensions must be strictly positive and satisfy minimum size constraints.
- [P5] `valid_zoom`: Coordinate mapping (viewport to canvas) requires a zoom scale strictly greater than zero (`> 0.0`).

## Postconditions
- [Q1] `translation_applied`: After translating, the node's `top_left` position is exactly `original_top_left + delta`. If snapping is enabled, the final position is rounded to the nearest multiple of the grid size.
- [Q2] `bounds_valid`: After resizing, the node's `BoundingBox` remains mathematically valid (`top_left.x < bottom_right.x` and `top_left.y < bottom_right.y`).
- [Q3] `absolute_position_preserved`: After reparenting, the node's absolute canvas position remains visually unchanged (its local position is recalculated relative to the new parent's absolute position).
- [Q4] `mapping_reversible`: After coordinate mapping, `viewport_to_canvas(canvas_to_viewport(p))` is approximately `p` (within standard floating point epsilon).

## Invariants
- [INV1] **Valid BoundingBox**: A node's width and height are always `> 0.0`. `top_left.x < bottom_right.x` and `top_left.y < bottom_right.y`.
- [INV2] **Tree Structure**: The node hierarchy is always a valid forest. No node is an ancestor of itself.
- [INV3] **Selection Validity**: The `SelectionSet` only ever contains `NodeId`s that currently exist in the canvas. If a node is deleted, it is removed from the selection.

## Error Taxonomy
- `CanvasError::NodeNotFound(NodeId)` - when operating on a non-existent node.
- `CanvasError::InvalidBoundingBox { width: f64, height: f64 }` - when a resize operation attempts to create non-positive dimensions.
- `CanvasError::CycleDetected(NodeId, NodeId)` - when reparenting would create a cycle (e.g., node to its own descendant).
- `CanvasError::InvalidGridResolution(f64)` - when grid size is `<= 0.0`.
- `CanvasError::InvalidZoomScale(f64)` - when zoom scale is `<= 0.0`.

## Contract Signatures
- `fn translate_node(node_id: NodeId, delta: CanvasVector) -> Result<(), CanvasError>`
- `fn resize_node(node_id: NodeId, new_bounds: ValidBoundingBox) -> Result<(), CanvasError>`
- `fn reparent_node(node_id: NodeId, new_parent: Option<NodeId>) -> Result<(), CanvasError>`
- `fn map_viewport_to_canvas(point: ViewportPoint, transform: ValidTransform) -> CanvasPoint`
- `fn snap_to_grid(point: CanvasPoint, grid_size: NonZeroPositiveF64) -> CanvasPoint`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P4: Valid Bounding Box | Compile-time (strongest) | `struct ValidBoundingBox { top_left: CanvasPoint, bottom_right: CanvasPoint }` (constructor validates and returns Result) |
| P3: Grid resolution > 0 | Compile-time | `struct NonZeroPositiveF64(f64)` (wrapper struct enforcing > 0.0) |
| P5: Zoom scale > 0 | Compile-time | `struct ValidTransform { zoom: NonZeroPositiveF64, ... }` |
| P1: Node exists | Error variant | `Result<T, CanvasError::NodeNotFound>` |
| P2: No cycles | Error variant | `Result<T, CanvasError::CycleDetected>` |

IMPORTANT: Prefer compile-time enforcement over runtime. Only fall through to
Result if the type system cannot enforce the constraint.

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES <P1>: `translate_node(NodeId(999), Vector(10.0, 10.0))` -- should produce `Err(CanvasError::NodeNotFound(NodeId(999)))`
- VIOLATES <P2>: `reparent_node(NodeId(1), Some(NodeId(1)))` -- should produce `Err(CanvasError::CycleDetected(NodeId(1), NodeId(1)))`
- VIOLATES <P3>: `NonZeroPositiveF64::new(0.0)` -- should produce `Err(CanvasError::InvalidGridResolution(0.0))`
- VIOLATES <P4>: `ValidBoundingBox::new(0.0, 0.0, -10.0, -10.0)` -- should produce `Err(CanvasError::InvalidBoundingBox { width: -10.0, height: -10.0 })`
- VIOLATES <P5>: `ValidTransform::new(zoom: -1.0, pan: (0,0))` -- should produce `Err(CanvasError::InvalidZoomScale(-1.0))`
- VIOLATES <Q1>: `translate_node` results in `node.top_left` not matching `original_top_left + delta` -- test fails explicitly.
- VIOLATES <Q2>: `resize_node` bypassing type safety and mutating width to `-5.0` -- compiler prevents this because fields of `ValidBoundingBox` are private.
- VIOLATES <Q3>: `reparent_node` visually shifting the node across the screen -- test fails explicitly by asserting absolute bounding box before and after.
- VIOLATES <Q4>: `viewport_to_canvas(canvas_to_viewport(p))` being `p + 5.0` -- test fails explicitly due to floating point epsilon check.

## Ownership Contracts (Rust-specific)
- Ownership transfer: `fn add_node(&mut self, node: ValidBoundingBox) -> NodeId` -- Canvas takes ownership of the node layout data.
- Shared borrow: `fn get_bounds(&self, node_id: NodeId) -> Result<ValidBoundingBox, CanvasError>` -- Reads state, no mutation.
- Exclusive borrow: `fn translate_node(&mut self, node_id: NodeId, delta: CanvasVector)` -- Mutates internal coordinates for `node_id`.
- Exclusive borrow: `fn reparent_node(&mut self, node_id: NodeId, new_parent: Option<NodeId>)` -- Mutates the hierarchy tree and the local coordinates of `node_id`.
- Clone policy: `NodeId` is `Copy`. Selections are cloned as `HashSet<NodeId>` when taking snapshots. Coordinate wrappers like `CanvasPoint` are `Copy`. Graph manipulation takes `&mut self` and does not clone subtrees.
