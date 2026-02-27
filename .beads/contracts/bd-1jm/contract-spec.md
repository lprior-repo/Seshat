# Contract Specification

## Context
- Feature: Add `can_undo()` and `can_redo()` query methods to `History` struct
- Domain terms:
  - `undo_stack`: rpds::List<DiagramDocument> - previous document states
  - `redo_stack`: rpds::List<DiagramDocument> - states available for redo
- Assumptions:
  - rpds::List::is_empty() is O(1) (persistent data structure property)
  - Methods will be used by UI to enable/disable toolbar buttons
- Open questions: None

## Preconditions
- **P1**: None - methods are always callable on any `&History`

## Postconditions
- **Q1 (can_undo)**: Returns `true` iff `undo_stack` is non-empty
- **Q2 (can_redo)**: Returns `true` iff `redo_stack` is non-empty
- **Q3**: No state mutation - `undo_stack` and `redo_stack` unchanged after call

## Invariants
- **I1**: Time complexity O(1) - must not iterate or allocate
- **I2**: No heap allocation - pure query methods
- **I3**: Thread-safe (via `&self` immutable borrow)

## Error Taxonomy
None - these are infallible query methods returning `bool`.

## Contract Signatures

```rust
impl History {
    #[must_use]
    pub fn can_undo(&self) -> bool;

    #[must_use]
    pub fn can_redo(&self) -> bool;
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| None required | N/A | Methods take `&self`, always valid |

## Violation Examples (REQUIRED)

No preconditions to violate - methods are always callable.

Postcondition violations would indicate implementation bugs:
- VIOLATES Q1: `can_undo()` returns `false` when `undo_stack.len() > 0` -- indicates bug in implementation
- VIOLATES Q2: `can_redo()` returns `false` when `redo_stack.len() > 0` -- indicates bug in implementation
- VIOLATES Q3: Stack contents differ before and after call -- indicates unintended mutation

## Ownership Contracts

- Shared borrow: `fn can_undo(&self)` -- read-only, no mutation, caller retains full access
- Shared borrow: `fn can_redo(&self)` -- read-only, no mutation, caller retains full access
- Clone policy: No cloning required or performed
- These methods are pure queries with zero ownership transfer

## Non-goals
- These methods do not perform undo/redo operations (use `undo()`/`redo()` for that)
- These methods do not return stack size (only boolean availability)
- These methods do not peek at stack contents
