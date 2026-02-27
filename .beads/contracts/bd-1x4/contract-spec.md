# Contract Specification: Copy/Paste Toolbar Buttons

**Bead ID:** bd-1x4  
**Component:** `diagram_tool/src/ui/toolbar.rs`  
**Functions:** `apply_copy_selection()`, `apply_paste_selection()` in `commands.rs`

---

## 1. Purpose

Add Copy and Paste buttons to the toolbar with proper disabled state management:
- Copy button disabled when no selection exists
- Paste button disabled when clipboard is empty

---

## 2. Function Contracts

### 2.1 Copy Button Handler

```rust
fn handle_copy(
    doc_signal: Signal<DiagramDocument>,
) -> bool
```

**Preconditions:**
- `doc_signal` must be a valid `Signal<DiagramDocument>` from context
- `selected_count > 0` (button disabled otherwise)

**Postconditions:**
- Returns `true` if clipboard was populated with selected nodes/edges
- Returns `false` if no selection existed
- `CLIPBOARD` thread-local now contains `Some(ClipboardState { nodes, edges, paste_serial: 0 })`
- Clipboard contains all nodes where `node.id` is in `selected_items`
- Clipboard contains all edges where both `edge.source` and `edge.target` are in selected nodes

**Side Effects:**
- Mutates `CLIPBOARD` thread-local storage
- No document mutation
- No history push

---

### 2.2 Paste Button Handler

```rust
fn handle_paste(
    mut doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) -> bool
```

**Preconditions:**
- `doc_signal` must be a valid `Signal<DiagramDocument>` from context
- `history_signal` must be a valid `Signal<History>` from context
- `CLIPBOARD` must contain `Some(ClipboardState)` (button disabled otherwise)
- `CLIPBOARD.nodes` must not be empty

**Postconditions:**
- Returns `true` if paste succeeded
- Returns `false` if clipboard was empty or had no nodes
- New nodes created with fresh UUIDs
- Pasted nodes offset by `20.0 * paste_serial` in both x and y
- Pasted nodes become the new selection
- `doc.revision` incremented
- History pushed before mutation

**Side Effects:**
- Mutates `DiagramDocument` (adds nodes, edges, updates selection)
- Mutates `History` (pushes undo snapshot)
- Increments `CLIPBOARD.paste_serial`

---

## 3. UI Component Contract

### 3.1 Copy Button

```rust
button {
    "data-testid": "toolbar-copy",
    disabled: stats.selected_count == 0,
    onclick: move |_| actions::copy_selection(doc_signal),
    "Copy"
}
```

**Invariants:**
- `data-testid` MUST be `"toolbar-copy"`
- Button MUST have `disabled` attribute
- `disabled` MUST be `true` when `selected_count == 0`
- `disabled` MUST be `false` when `selected_count > 0`
- Style MUST follow existing button patterns

**Ownership:**
- `doc_signal` is `Copy` (Signal<T>), no clone needed
- onclick closure captures `doc_signal` by value

---

### 3.2 Paste Button

```rust
button {
    "data-testid": "toolbar-paste",
    disabled: clipboard_is_empty(),
    onclick: move |_| actions::paste_selection(doc_signal, history_signal),
    "Paste"
}
```

**Invariants:**
- `data-testid` MUST be `"toolbar-paste"`
- Button MUST have `disabled` attribute
- `disabled` MUST be `true` when clipboard is `None`
- `disabled` MUST be `false` when clipboard is `Some(_)`
- Style MUST follow existing button patterns

**Ownership:**
- Both signals are `Copy`, captured by value in closure

---

## 4. Clipboard State Query

### 4.1 New Helper Function Required

```rust
pub fn clipboard_has_content() -> bool
```

**Contract:**
- Returns `true` if `CLIPBOARD` contains `Some(ClipboardState)` with non-empty nodes
- Returns `false` if `CLIPBOARD` is `None` or nodes is empty
- MUST be callable from toolbar (pub visibility)
- MUST NOT mutate clipboard state (read-only)

---

## 5. State Dependencies

| State | Source | Read By |
|-------|--------|---------|
| `selected_count` | `ToolbarStats` | Copy button disabled state |
| `CLIPBOARD` | `commands.rs` thread_local | Paste button disabled state |
| `doc_signal` | Context | Copy/Paste handlers |
| `history_signal` | Context | Paste handler only |

---

## 6. Error Taxonomy

| Error Case | Handling | User Feedback |
|------------|----------|---------------|
| Copy with no selection | Button disabled | Visual disabled state |
| Paste with empty clipboard | Button disabled | Visual disabled state |
| Paste into same location | Offset applied automatically | Nodes appear offset |

---

## 7. Type Encoding

No type-encoded preconditions required. Disabled states handle guards at UI layer.

---

## 8. Violation Examples

### Invalid: Copy without checking selection
```rust
// BAD: No precondition check
onclick: move |_| { apply_copy_selection(doc_signal); }
```

### Invalid: Paste without clipboard check
```rust
// BAD: Could panic or do nothing silently
onclick: move |_| { apply_paste_selection(doc_signal, history_signal); }
```

### Valid: Guarded by disabled state
```rust
// GOOD: UI prevents invocation when preconditions fail
disabled: stats.selected_count == 0,
onclick: move |_| actions::copy_selection(doc_signal),
```
