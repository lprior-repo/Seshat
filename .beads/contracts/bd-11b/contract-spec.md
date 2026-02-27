# Contract Specification

## Context
- Feature: Add disabled states to Undo/Redo toolbar buttons
- Domain terms:
  - `History`: Dioxus Signal containing undo/redo stack state
  - `can_undo()`: Returns `true` when undo_stack is non-empty
  - `can_redo()`: Returns `true` when redo_stack is non-empty
- Dependencies:
  - `bd-1jm`: Provides `can_undo()` and `can_redo()` on History struct
- Assumptions:
  - History signal is available via `use_context::<Signal<History>>()`
  - Dioxus `disabled` attribute prevents onclick firing
  - Visual feedback via opacity/cursor communicates disabled state
- Open questions: None

## Preconditions
- **P1**: History signal must be available in context (inherited from parent component)
- **P2**: `can_undo()` and `can_redo()` methods exist on History (guaranteed by bd-1jm)

## Postconditions
- **Q1 (Undo button)**: `disabled` attribute is `true` when `!history.read().can_undo()`
- **Q2 (Redo button)**: `disabled` attribute is `true` when `!history.read().can_redo()`
- **Q3 (Visual state)**: Disabled buttons have reduced opacity and `cursor: not-allowed`
- **Q4 (Click prevention)**: Disabled buttons do not fire `onclick` handlers

## Invariants
- **I1**: Button state reacts to History signal changes (reactive binding)
- **I2**: Disabled styling follows existing pattern (Delete button at lines 144-150)
- **I3**: No direct History mutation from disabled state logic (read-only query)

## Error Taxonomy
None - disabled state is computed from infallible boolean queries.

## Contract Signatures

```rust
// In toolbar.rs component

// History access pattern (already exists)
let history_signal = use_context::<Signal<History>>();

// Undo button contract
button {
    disabled: !history_signal.read().can_undo(),
    style: "... opacity: {undo_opacity}; cursor: {undo_cursor}; ...",
    onclick: move |_| actions::undo(doc_signal, history_signal),
    "Undo"
}

// Redo button contract
button {
    disabled: !history_signal.read().can_redo(),
    style: "... opacity: {redo_opacity}; cursor: {redo_cursor}; ...",
    onclick: move |_| actions::redo(doc_signal, history_signal),
    "Redo"
}
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: History in context | Runtime | `use_context::<Signal<History>>()` panics if missing |
| P2: Methods exist | Compile-time | Rust method resolution |

## Violation Examples (REQUIRED)

**VIOLATES Q1**: Undo button enabled when undo_stack is empty
```rust
// WRONG: Always enabled
button { disabled: false, "Undo" }
```

**VIOLATES Q2**: Redo button disabled when redo_stack is non-empty
```rust
// WRONG: Inverted logic
button { disabled: history_signal.read().can_redo(), "Redo" }
```

**VIOLATES Q3**: No visual feedback for disabled state
```rust
// WRONG: Missing opacity/cursor changes
button { disabled: !can_undo, style: "opacity: 1.0; cursor: pointer;", "Undo" }
```

**VIOLATES Q4**: onclick fires on disabled button (Dioxus handles this, but...)
```rust
// WRONG: Manual guard instead of disabled attribute
onclick: move |_| {
    if history_signal.read().can_undo() {
        actions::undo(doc_signal, history_signal);
    }
}
```

## Ownership Contracts

- Shared borrow: `history_signal.read()` -- temporary borrow for query
- No cloning: Boolean values are Copy
- Signal access pattern: Read-only, reactive

## Implementation Pattern

Follow the Delete button pattern at lines 144-150:
1. Compute disabled boolean: `let undo_disabled = !history_signal.read().can_undo();`
2. Derive visual values: `let undo_opacity = if undo_disabled { "0.4" } else { "1.0" };`
3. Derive cursor: `let undo_cursor = if undo_disabled { "not-allowed" } else { "pointer" };`
4. Apply to button: `disabled: undo_disabled, style: "... opacity: {undo_opacity}; cursor: {undo_cursor};"`

## Non-goals
- Does not modify History state (that's actions::undo/redo)
- Does not add keyboard shortcuts (separate concern)
- Does not add tooltips explaining disabled state
