# Contract Specification: Aspect Lock During Multi-Select Resize (MUL-013)

## Context
- **Feature**: Implement aspect lock during multi-select resize
- **Bead ID**: oya-avp
- **Domain terms**:
  - `SelectionBounds`: Bounding box of multi-selection (x, y, width, height)
  - `ResizeHandle`: Corner/edge being dragged (NW, N, NE, E, SE, S, SW, W)
  - `AspectRatio`: Width/height ratio (f64)
  - `InteractionMode::ResizingSelection`: State during resize operation
- **Assumptions**:
  - Aspect ratio is calculated from initial selection bounds at resize start
  - Aspect ratio lock is toggled via UI (keyboard modifier or toggle)
  - Locked aspect ratio applies to all nodes in selection proportionally
- **Open questions**:
  - What UI mechanism toggles aspect lock? (Assume Shift key modifier for now)

## Preconditions
- [P1] At least one node must be selected before resize can begin
- [P2] Resize handle must be valid (one of 8 corners/edges)
- [P3] Original bounds must have positive width and height (for valid aspect ratio)

## Postconditions
- [Q1] When aspect_ratio is `Some(ratio)`, new bounds MUST maintain that ratio within 1e-9 tolerance
- [Q2] When aspect_ratio is `None`, resize behaves as before (no ratio constraint)
- [Q3] All nodes in selection are scaled proportionally from their original positions
- [Q4] The aspect_ratio field is stored in ResizingSelection state

## Invariants
- [I1] If aspect_ratio is `Some(r)`, then new_width / new_height ≈ r (within floating-point tolerance)
- [I2] All node positions remain within canvas bounds after resize

## Error Taxonomy
- No new error variants needed - this is a feature enhancement, not error-handling change

## Contract Signatures

```rust
// In interaction_reducer.rs - ResizingSelection variant
enum InteractionMode {
    ResizingSelection {
        handle: ResizeHandle,
        original_bounds: (f64, f64, f64, f64),  // x, y, width, height
        originals: HashMap<NodeId, (f64, f64, f64, f64)>,
        anchor: (f64, f64),
        did_resize: bool,
        aspect_ratio: Option<f64>,  // NEW: locked aspect ratio
    },
    // ... other variants
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: At least one node selected | Runtime-checked | `HashMap` not empty |
| P2: Valid resize handle | Compile-time | `enum ResizeHandle { Nw, N, Ne, E, Se, S, Sw, W }` |
| P3: Positive dimensions | Runtime-checked | `width > 0.0 && height > 0.0` |

## Violation Examples
- VIOLATES Q1: Resize with aspect_ratio=Some(2.0) to width=100 -> height should be 50, not 60
- VIOLATES Q3: Node at position (100,100) with original (50,50,100,100) -> new position should scale proportionally

## Ownership Contracts
- `originals: HashMap<NodeId, (f64, f64, f64, f64)>` - borrowed from document, no ownership taken
- `aspect_ratio: Option<f64>` - Copy type, no ownership concerns

## Non-goals
- [ ] Implementing UI toggle for aspect lock (handled in separate bead)
- [ ] Aspect lock persistence across sessions
- [ ] Rotation-aware aspect preservation
