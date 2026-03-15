# Contract Specification

## Context
- **Feature**: Edge Labels (EDG-022 to EDG-026)
- **Domain terms**:
  - `Edge` - A connection between two nodes
  - `label` - Text content displayed on the edge
  - `label_offset_t` - Position along the edge (0.0 = source, 1.0 = target, 0.5 = midpoint)
  - `font_size` - Font size for the label text
- **Assumptions**: The Edge model already exists with label fields
- **Open questions**: None - feature is already implemented, this is verification

## Preconditions
- [ ] Edge must have valid source and target nodes
- [ ] Edge label_offset_t must be finite if set
- [ ] Edge label_offset_t must be in range [0.0, 1.0]

## Postconditions
- [ ] Edge with empty label renders without label text
- [ ] Edge with non-empty label renders label at computed position
- [ ] Default label_offset_t (0.5) positions label at geometric midpoint
- [ ] Custom label_offset_t positions label proportionally along edge path
- [ ] Edge label is included in document serialization
- [ ] Edge label deserializes correctly from JSON

## Invariants
- [ ] Edge label_offset_t clamped to [0.0, 1.0] when computing position
- [ ] Edge label rendering respects zoom threshold (visible at zoom >= 0.3)

## Error Taxonomy
- N/A - This is a verification bead, not implementing new error handling

## Contract Signatures
```rust
// Existing function signatures to verify
fn edge_label_position(sx: f64, sy: f64, tx: f64, ty: f64, edge: &Edge) -> (f64, f64)
fn apply_update_edge_label(state: &mut DiagramState, edge_id: &str, label: &str) -> Result<(), Error>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| label_offset_t finite | Runtime validation | `edge.label_offset_t.0.is_finite()` |
| label_offset_t in [0,1] | Runtime clamp | `t.clamp(0.0, 1.0)` |

## Violation Examples
- N/A - No new error handling being implemented

## Ownership Contracts
- `edge_label_position` takes `&Edge` - read-only, no mutation
- `apply_update_edge_label` takes `&mut DiagramState` - mutates edge label field

## Non-goals
- [ ] Adding new edge label features beyond midpoint positioning
- [ ] Edge label styling beyond font_size
