bead_id: bd-11b
bead_title: toolbar: Add disabled states to Undo/Redo buttons
phase: p2
updated_at: 2026-03-01T00:46:05Z

# Implementation: disabled states

## Verification

Already implemented in `diagram_tool/src/ui/toolbar.rs`:

```rust
let undo_disabled = !history_signal.read().can_undo();
let redo_disabled = !history_signal.read().can_redo();
```

Used in button definitions:
```rust
disabled: undo_disabled,
...
disabled: redo_disabled,
```

## Acceptance Criteria Coverage
| Criteria | Status |
|----------|--------|
| Undo button disabled when can_undo() returns false | ✅ |
| Redo button disabled when can_redo() returns false | ✅ |
| Button visual state reflects actual availability | ✅ |
