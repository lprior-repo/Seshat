# Contract Specification

## Context
- Feature: Group Translate (MUL-006 to MUL-010)
- Domain terms: `DiagramDocument`, `NodeId`, `TransformError`, `dx`, `dy`, `selected_items`.
- Assumptions: Translating a selected node updates its `x` and `y` coordinates. Ancestor containers' bounds must be recomputed.
- Open questions: None

## Preconditions
- [x] P1: The selection must not be empty.
- [x] P2: No selected node may be locked.
- [x] P3: Translation deltas (`dx`, `dy`) must be finite numbers (no NaN or Infinity).

## Postconditions
- [x] Q1: Every selected node's `x` coordinate is increased exactly by `dx`.
- [x] Q2: Every selected node's `y` coordinate is increased exactly by `dy`.
- [x] Q3: Unselected nodes that are not ancestors of selected nodes remain strictly unmodified.
- [x] Q4: The bounds of all ancestor containers of translated nodes are recomputed.

## Invariants
- [x] I1: The number of nodes in the document does not change.
- [x] I2: Node locked status and node kinds do not change.
- [x] I3: The selected items set does not change during translation.

## Error Taxonomy
- `TransformError::EmptySelection` - when the document's `selected_items` set is empty.
- `TransformError::LockedNode(NodeId)` - when at least one node in `selected_items` has `locked == true`.
- `TransformError::InvalidDelta` - when `dx` or `dy` is not a finite number.

## Contract Signatures
- `pub fn translate_selection(doc: &mut DiagramDocument, dx: f64, dy: f64) -> Result<(), TransformError>`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Selection non-empty | Error variant | `Result<(), TransformError::EmptySelection>` |
| P2: No locked nodes | Error variant | `Result<(), TransformError::LockedNode>` |
| P3: Deltas are finite | Compile-time | `OrderedFloat<f64>` or `Result<(), TransformError::InvalidDelta>` |

## Violation Examples
- VIOLATES P1: `translate_selection(&mut doc_with_empty_selection, 10.0, 10.0)` -- should produce `Err(TransformError::EmptySelection)`
- VIOLATES P2: `translate_selection(&mut doc_with_locked_selected_node, 10.0, 10.0)` -- should produce `Err(TransformError::LockedNode(locked_node_id))`
- VIOLATES P3: `translate_selection(&mut doc, f64::NAN, 10.0)` -- should produce `Err(TransformError::InvalidDelta)`

## Ownership Contracts
- Exclusive borrow: `fn translate_selection(doc: &mut DiagramDocument, ...)` -- Mutates `doc.document.nodes` specifically the `x` and `y` fields of selected nodes, and `x, y, width, height` fields of ancestor containers. Does not mutate `editor_state.selected_items`.

## Non-goals
- Translating nodes that are partially locked (nodes are either fully locked or not).
- Rotating nodes.
