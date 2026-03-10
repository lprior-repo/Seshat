# Contract Specification

## Context
- Feature: Container + children resize behavior
- Bead ID: oya-1hl
- Domain: Diagram tool with container/subgraph nodes containing children
- Assumptions:
  - Container = NodeKind::Subgraph
  - Children = nodes with parent field pointing to container
  - Two resize modes: "scale children" vs "expand container"
- Open questions:
  - What happens when container shrinks smaller than children bounds?
  - Should there be a min/max size constraint?

## Preconditions
- [P1] Container node must exist in the diagram
- [P2] New dimensions must be valid (positive width/height)
- [P3] Resize mode must be valid (ScaleChildren or ExpandContainer)

## Postconditions
- [Q1] Container node dimensions are updated to new width/height
- [Q2] Children are transformed according to resize mode
- [Q3] All ancestor containers have bounds recomputed
- [Q4] All descendant nodes maintain relative positioning within container

## Invariants
- [I1] Container bounds must always encompass all children bounds (with padding)
- [I2] Children must remain within container bounds after resize
- [I3] Relative positions of children within container are preserved

## Error Taxonomy
- Error::ContainerNotFound - when container node ID doesn't exist
- Error::InvalidDimensions - when width/height <= 0
- Error::InvalidResizeMode - when mode is not recognized
- Error::ContainerTooSmall - when container would shrink below children bounds (ExpandContainer mode)

## Contract Signatures
```rust
pub enum ResizeMode {
    ScaleChildren,    // Children scale proportionally with container
    ExpandContainer,  // Container expands to fit children, children stay same size
}

pub fn apply_container_resize(
    state: DiagramProjection,
    container_id: &str,
    new_width: f64,
    new_height: f64,
    mode: ResizeMode,
) -> Result<DiagramProjection, ResizeError>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| container exists | Runtime-checked | Result<T, Error::ContainerNotFound> |
| dimensions > 0 | Compile-time | `NonZeroF64` or runtime check |
| valid mode | Compile-time | enum with known variants |

## Violation Examples
- VIOLATES P1: `apply_container_resize(state, "nonexistent", 100, 100, ScaleChildren)` → `Err(ResizeError::ContainerNotFound)`
- VIOLATES P2: `apply_container_resize(state, "container1", -50, 100, ScaleChildren)` → `Err(ResizeError::InvalidDimensions)`
- VIOLATES Q2: After ScaleChildren, children's width/height should scale proportionally

## Ownership Contracts
- `state: DiagramProjection` - shared borrow, returns new state (functional style)
- `container_id: &str` - borrowed string
- No mutation of input state (functional update pattern)

## Non-goals
- [ ] Resize individual children without container
- [ ] Drag-to-resize interaction (UI layer)
- [ ] Animation of resize (UI layer)
