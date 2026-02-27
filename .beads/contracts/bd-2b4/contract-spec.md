# Contract Specification: Copy/Paste Operations Unit Tests

**Bead ID:** bd-2b4  
**Target:** `diagram_tool/src/ui/commands.rs`  
**Functions:** `apply_copy_selection`, `apply_paste_selection`

---

## 1. System Under Test (SUT)

### `apply_copy_selection(doc_signal: Signal<DiagramDocument>) -> bool`

**Purpose:** Copy selected nodes and their internal edges to thread-local clipboard.

**Preconditions:**
- `doc_signal` is a valid initialized Signal
- Document may contain 0..N nodes
- Document may contain 0..N edges
- `editor_state.selected_items` may be empty or contain node/edge IDs

**Postconditions:**
- Returns `false` if `selected_items` contains no valid node IDs
- Returns `true` if at least one node was copied
- On success, `CLIPBOARD` contains:
  - `nodes`: Vec of `(NodeId, Node)` tuples for selected nodes that exist in document
  - `edges`: Vec of `Edge` where BOTH `source` and `target` are in selected nodes
  - `paste_serial`: 0 (reset on copy)

**Invariants:**
- Document state is NOT modified (read-only operation)
- Clipboard state is thread-local
- Edges with partially-selected endpoints are NOT copied

---

### `apply_paste_selection(doc_signal, history_signal) -> bool`

**Purpose:** Paste clipboard contents with offset, generating new IDs.

**Preconditions:**
- `CLIPBOARD` may be `None` or contain previous copy
- If `Some`, clipboard `nodes` may be empty
- `history_signal` is valid

**Postconditions:**
- Returns `false` if `CLIPBOARD` is `None`
- Returns `false` if clipboard `nodes` is empty
- Returns `true` and modifies document on success:
  - New nodes have UNIQUE `NodeId`s (UUID-based)
  - Node positions offset by `20.0 * paste_serial.max(1)`
  - Parent relationships remapped if parent was also pasted
  - Edges remapped to new node IDs
  - `editor_state.selected_items` contains ONLY new node IDs
  - `revision` is incremented
  - History receives pre-paste state

**Invariants:**
- `paste_serial` increments via `saturating_add(1)`
- Original nodes remain unchanged
- Offset calculation: `offset = 20.0 * serial.max(1)`
  - First paste (serial=1): offset = 20.0
  - Second paste (serial=2): offset = 40.0

---

## 2. Error Taxonomy

| Error Category | Condition | Return Value | Side Effects |
|----------------|-----------|--------------|--------------|
| Empty Selection | `selected_items` empty or no valid nodes | `false` | None |
| Empty Clipboard | `CLIPBOARD` is `None` | `false` | None |
| No Nodes in Clipboard | `CLIPBOARD.nodes.is_empty()` | `false` | None |

---

## 3. Type Contracts

### NodeId Uniqueness
```
forall paste operation:
  old_ids = clipboard.nodes.map(|(id, _)| id)
  new_ids = pasted_nodes.map(|(id, _)| id)
  intersection(old_ids, new_ids) = empty_set
```

### Edge Remapping
```
forall edge in clipboard.edges:
  if edge.source in old_ids AND edge.target in old_ids:
    new_edge.source = id_map[old_edge.source]
    new_edge.target = id_map[old_edge.target]
    new_edge.id = fresh_uuid()
```

### Parent Remapping
```
forall pasted_node with parent:
  if parent in old_ids:
    pasted_node.parent = id_map[parent]
  else:
    pasted_node.parent = original_parent (unchanged)
```

---

## 4. Test Design Constraints

### Zero-Unwrap Policy
Tests MUST NOT use `.unwrap()` or `.expect()` on:
- `Option` types from clipboard operations
- `HashMap` lookups for node/edge retrieval
- Signal read/write operations

**Allowed patterns:**
- `if let Some(x) = ...`
- `match opt { Some(x) => ..., None => ... }`
- `.unwrap_or_default()` for collections
- `let _ = result;` for explicitly ignored results

### Helper Reuse
Tests MUST use existing helpers:
- `make_doc_with_zoom(f64) -> DiagramDocument`
- May extend with `make_doc_with_nodes(nodes: Vec<Node>) -> DiagramDocument`

### Isolation
Each test MUST:
- Clear clipboard before test start: `CLIPBOARD.with(|s| *s.borrow_mut() = None);`
- Use fresh `DiagramDocument` instances
- Not rely on execution order

---

## 5. Coverage Requirements

| Scenario | `apply_copy_selection` | `apply_paste_selection` |
|----------|------------------------|-------------------------|
| Empty selection | Covered | N/A |
| Single node | Covered | Covered |
| Multiple nodes | Covered | Covered |
| Nodes with edges | Covered | Covered |
| Empty clipboard | N/A | Covered |
| Parent-child relationship | Covered | Covered |
| Multiple paste iterations | N/A | Covered |

---

## 6. Acceptance Criteria

1. All test functions compile without warnings
2. All tests pass with `cargo test --package diagram_tool`
3. No `.unwrap()` or `.expect()` in test code (except test module allow attribute)
4. Test names follow `given_X_when_Y_then_Z` pattern
5. Each test has exactly ONE assertion focus
6. Clipboard state is isolated between tests
