# Contract Specification: seshat-x68 - MUL-016 to MUL-020 Multi-select rotation

## Context
- **Feature**: Rotate a group of nodes around a center point without deformation
- **Domain terms**:
  - `Multi-selection`: A set of 1+ nodes selected by the user
  - `Rotation center`: The pivot point around which nodes rotate (typically selection centroid)
  - `Rotation angle`: The angle of rotation in radians
  - `Snap angle`: Rotation snaps to cardinal directions (0, 90, 180, 270 degrees)
  - `Relative distance`: Distance between any two nodes in the selection
- **Assumptions**:
  - Rotation is performed in a 2D coordinate space
  - Nodes have x, y, width, height properties
  - Selection centroid is computed from node bounding box
- **Open questions**:
  - Should rotation affect node rotation property or just position? (Currently position only)
  - Is there a maximum rotation angle? (No limit, wraps via modulo)

## Preconditions
- [P1] Selection must contain at least one node (`NonEmptyVec<NodeId>` enforces at compile-time)
- [P2] All nodes in selection must exist in the document (checked at runtime via `NodeNotFound`)
- [P3] No node in selection may be locked (checked at runtime via `ItemLocked`)
- [P4] Selection must not contain invalid hierarchy (parent-child both selected) - checked via `InvalidHierarchy`
- [P5] Rotation angle must be finite (checked via `InvalidRotation` error)
- [P6] Center point must have finite coordinates (checked via `InvalidRotation` error)

## Postconditions
- [Q1] All nodes in selection are rotated around the center by the specified angle
- [Q2] Selection centroid remains at the same position after rotation (center of rotation invariant)
- [Q3] Relative distances between all pairs of nodes are preserved (no deformation)
- [Q4] All nodes remain in the document after rotation
- [Q5] Rotation uses subpixel precision (f64 coordinates)

## Invariants
- [I1] Node count in document remains constant after rotation
- [I2] Node identities (IDs) remain unchanged after rotation
- [I3] For any rotation angle A, rotating by A then -A returns to original positions

## Error Taxonomy
- `Error::EmptySelection` - Selection is empty (should never occur due to NonEmptyVec)
- `Error::NodeNotFound` - A node in the selection does not exist in the document
- `Error::ItemLocked` - One or more nodes in the selection are locked
- `Error::InvalidHierarchy` - Selection contains both parent and child nodes
- `Error::InvalidRotation` - Rotation angle or center point has NaN/Infinity
- `Error::InvalidScale` - Used by scale operations (not rotation)

## Contract Signatures
```rust
/// Rotate a multi-selection around its centroid
fn rotate_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    angle_radians: f64,
) -> Result<(), Error>;

/// Rotate a multi-selection around a custom pivot point
fn rotate_selection_around_point(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    pivot: Point,
    angle_radians: f64,
) -> Result<(), Error>;

/// Snap angle to nearest cardinal direction (0, 90, 180, 270 degrees)
fn snap_angle_to_cardinal(angle_radians: f64, tolerance: f64) -> f64;
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| selection not empty | Compile-time | `NonEmptyVec<NodeId>` |
| nodes exist | Runtime | `doc.document.nodes.get(id)` returns Some |
| nodes not locked | Runtime | `node.locked == false` check |
| no invalid hierarchy | Runtime | parent-child mutual exclusion check |
| angle finite | Runtime | `angle_radians.is_finite()` |
| center finite | Runtime | `center.x.is_finite() && center.y.is_finite()` |

## Violation Examples (REQUIRED)
- VIOLATES P3: `rotate_selection(doc, NonEmptyVec::try_from([locked_node_id]), PI/2)` -- should produce `Err(Error::ItemLocked)`
- VIOLATES P5: `rotate_selection(doc, selection, f64::NAN)` -- should produce `Err(Error::InvalidRotation)`
- VIOLATES P5: `rotate_selection(doc, selection, f64::INFINITY)` -- should produce `Err(Error::InvalidRotation)`
- VIOLATES Q3: After rotating asymmetric selection, verify `distance(node_a, node_b)` equals original distance (test verifies postcondition)
- VIOLATES Q2: Verify centroid of rotated positions equals original centroid (test verifies postcondition)

## Ownership Contracts (Rust-specific)
- `doc: &mut DiagramDocument` - Exclusive borrow, mutates node positions (x, y fields)
- `selection: NonEmptyVec<NodeId>` - Owned parameter, consumed by function
- `angle_radians: f64` - Copy type, no ownership implications
- No clone operations in rotation - nodes are mutated in-place

## Non-goals
- Rotating individual nodes (single-node rotation handled by existing single-select code)
- Rotating edges (future feature)
- Animated rotation (UI concern, not data model)
- Rotation snap UI (handled in UI layer)
