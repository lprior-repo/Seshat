# Contract Specification

## Context
- Feature: EDG-026 to EDG-030: Edge Style Variants
- Domain terms: 
  - `EdgeStyle`: Enum defining the visual rendering of an edge (Solid, Dashed, Dotted).
  - `DiagramProjection`: The application state containing nodes and edges.
  - `EdgeOpsError`: The error taxonomy for edge operations.
- Assumptions: 
  - Edge styles are applied via a functional operation, such as `apply_edge_style`.
  - The operation adheres to the functional `Data -> Calc -> Actions` pattern, consuming the prior state and returning a new state.
- Open questions: None.

## Preconditions
- [P1] The target edge must exist in the `DiagramProjection` before its style can be updated.

## Postconditions
- [Q1] The returned `DiagramProjection` must contain the target edge with its `style` field updated to the requested `EdgeStyle` variant.
- [Q2] All other properties of the target edge (source, target, label, etc.) must remain unchanged.
- [Q3] All other nodes and edges in the `DiagramProjection` must remain unchanged.

## Invariants
- [I1] The `style` field of an `Edge` is always a valid `EdgeStyle` variant.
- [I2] The overall `DiagramProjection` structural integrity (edge endpoints referencing valid nodes) is preserved.

## Error Taxonomy
- `EdgeOpsError::EdgeNotFound` - when attempting to set the style of a non-existent edge ID.

## Contract Signatures
- `pub fn apply_edge_style(state: DiagramProjection, id: &str, style: EdgeStyle) -> Result<DiagramProjection, EdgeOpsError>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Edge exists (P1) | Error variant | `Result<DiagramProjection, EdgeOpsError::EdgeNotFound>` |
| Valid style (I1) | Compile-time (strongest) | `EdgeStyle` enum (`Solid`, `Dashed`, `Dotted`) |

## Violation Examples (REQUIRED)
- VIOLATES <P1>: `apply_edge_style(state, "missing-edge", EdgeStyle::Dashed)` -- should produce `Err(EdgeOpsError::EdgeNotFound("missing-edge".to_string()))`
- VIOLATES <Q1>: Edge style is not updated in the returned projection after calling `apply_edge_style` -- should be verified by test `test_postcondition_edge_style_is_updated`.

## Ownership Contracts (Rust-specific)
- Ownership transfer: `fn apply_edge_style(state: DiagramProjection, id: &str, style: EdgeStyle)` -- Caller transfers ownership of `state`. The function consumes it and returns a new `DiagramProjection` upon success to enforce a linear history and eliminate mutation bugs.
- Shared borrow: The `id` parameter is a shared string slice (`&str`).
- Exclusive borrow: None. Pure functions must not use `&mut`.
- Clone policy: Uses `im::HashMap::update` to immutably update the edge, preserving structural sharing for historical revisions.

## Non-goals
- Implementing UI rendering for these edges (this contract is strictly for the domain model and state operations).