# Martin Fowler Test Plan: Copy/Paste Operations

**Bead ID:** bd-2b4  
**Style:** Given-When-Then (BDD)  
**Target:** `diagram_tool/src/ui/commands.rs`

---

## Test Suite: `apply_copy_selection`

### TC-COPY-001: Empty Selection Returns False

```
GIVEN a document with no selected items
WHEN apply_copy_selection is called
THEN it returns false
AND clipboard remains empty
```

**Setup:**
```rust
let doc = DiagramDocument::default(); // selected_items is empty
let doc_signal = Signal::new(doc);
CLIPBOARD.with(|s| *s.borrow_mut() = None);
```

**Assertion:**
- `assert!(!result);`
- `CLIPBOARD.with(|s| assert!(s.borrow().is_none()));`

---

### TC-COPY-002: Selection With Non-Existent Node IDs Returns False

```
GIVEN a document with selected_items containing IDs not in nodes
WHEN apply_copy_selection is called
THEN it returns false
AND clipboard remains empty
```

**Setup:**
```rust
let mut doc = DiagramDocument::default();
doc.editor_state.selected_items.insert("ghost-id".to_string());
// document.nodes is empty
```

---

### TC-COPY-003: Single Node Selection Copies Successfully

```
GIVEN a document with one selected node
WHEN apply_copy_selection is called
THEN it returns true
AND clipboard contains exactly one node
AND clipboard edges is empty
AND paste_serial is 0
```

**Setup:**
```rust
let mut doc = DiagramDocument::default();
let node_id = NodeId::new("node-1".to_string());
let node = Node { label: "Test".into(), x: OrderedFloat(100.0), y: OrderedFloat(50.0), ... };
doc.document.nodes.insert(node_id.clone(), node);
doc.editor_state.selected_items.insert(node_id.to_string());
```

**Assertion:**
- `assert!(result);`
- Clipboard has 1 node, 0 edges, paste_serial == 0

---

### TC-COPY-004: Multiple Nodes Selection Copies All

```
GIVEN a document with three selected nodes
WHEN apply_copy_selection is called
THEN it returns true
AND clipboard contains exactly three nodes
```

**Setup:** Create 3 nodes, select all via `selected_items`

---

### TC-COPY-005: Edge Copied Only When Both Endpoints Selected

```
GIVEN a document with two nodes and one edge between them
AND both nodes are selected
WHEN apply_copy_selection is called
THEN clipboard contains both nodes
AND clipboard contains exactly one edge
```

**Setup:**
```rust
let node_a = NodeId::new("a".into());
let node_b = NodeId::new("b".into());
let edge = Edge { source: node_a.clone(), target: node_b.clone(), ... };
// Add both nodes and edge to document
// Select both node_a and node_b
```

---

### TC-COPY-006: Partial Edge Selection Excludes Edge

```
GIVEN a document with two nodes and one edge between them
AND only source node is selected
WHEN apply_copy_selection is called
THEN clipboard contains one node
AND clipboard edges is empty
```

**Rationale:** Edge is only copied when BOTH source AND target are selected.

---

### TC-COPY-007: Nested Nodes Preserve Parent Reference

```
GIVEN a document with parent node P and child node C
AND both nodes are selected
WHEN apply_copy_selection is called
THEN clipboard contains both nodes
AND child node's parent field references the original parent ID
```

**Note:** Parent remapping happens at PASTE time, not COPY time.

---

## Test Suite: `apply_paste_selection`

### TC-PASTE-001: Empty Clipboard Returns False

```
GIVEN clipboard is None
WHEN apply_paste_selection is called
THEN it returns false
AND document is unchanged
```

**Setup:**
```rust
CLIPBOARD.with(|s| *s.borrow_mut() = None);
let doc_signal = Signal::new(DiagramDocument::default());
let history_signal = Signal::new(History::default());
```

---

### TC-PASTE-002: Clipboard With Empty Nodes Returns False

```
GIVEN clipboard exists with empty nodes vector
WHEN apply_paste_selection is called
THEN it returns false
```

**Setup:**
```rust
CLIPBOARD.with(|s| {
    *s.borrow_mut() = Some(ClipboardState {
        nodes: vec![],
        edges: vec![],
        paste_serial: 0,
    });
});
```

---

### TC-PASTE-003: Single Node Paste Creates New Node With Unique ID

```
GIVEN clipboard contains one node with ID "original-id"
WHEN apply_paste_selection is called
THEN document has one additional node
AND new node ID is different from "original-id"
AND selection contains only the new node ID
```

**Verification:**
```rust
let original_count = doc.document.nodes.len();
// ... paste ...
assert_eq!(doc.document.nodes.len(), original_count + 1);
// Verify new ID != old ID
```

---

### TC-PASTE-004: Paste Applies Offset To Position

```
GIVEN clipboard contains node at position (100.0, 50.0)
AND paste_serial becomes 1
WHEN apply_paste_selection is called
THEN pasted node position is (120.0, 70.0)
```

**Calculation:** `offset = 20.0 * max(1, serial) = 20.0 * 1 = 20.0`

---

### TC-PASTE-005: Second Paste Applies Double Offset

```
GIVEN clipboard contains node at position (100.0, 50.0)
AND this is the second paste (serial = 2)
WHEN apply_paste_selection is called
THEN pasted node position is (140.0, 90.0)
```

**Calculation:** `offset = 20.0 * 2 = 40.0`

---

### TC-PASTE-006: Multiple Nodes Get Unique IDs

```
GIVEN clipboard contains three nodes
WHEN apply_paste_selection is called
THEN document has three additional nodes
AND all three new node IDs are unique
AND all three new node IDs differ from original IDs
```

**Verification:**
```rust
let new_ids: HashSet<_> = doc.document.nodes.keys().collect();
assert_eq!(new_ids.len(), 3); // All unique
```

---

### TC-PASTE-007: Edge Is Remapped To New Node IDs

```
GIVEN clipboard contains nodes A, B and edge A->B
WHEN apply_paste_selection is called
THEN pasted edge source is new_A's ID
AND pasted edge target is new_B's ID
AND pasted edge has a new edge ID
```

---

### TC-PASTE-008: Parent Relationship Remapped When Parent Also Pasted

```
GIVEN clipboard contains parent node P and child node C
AND C.parent = P.id
WHEN apply_paste_selection is called
THEN pasted child's parent = new_P.id
```

---

### TC-PASTE-009: Parent Relationship Preserved When Parent Not Pasted

```
GIVEN clipboard contains child node C only
AND C.parent = P.id (P exists in document but not selected)
WHEN apply_paste_selection is called
THEN pasted child's parent = original P.id
```

---

### TC-PASTE-010: Selection Updated To Pasted Nodes Only

```
GIVEN document has selection containing "old-node"
AND clipboard contains one node
WHEN apply_paste_selection is called
THEN selection contains only the new pasted node ID
AND "old-node" is NOT in selection
```

---

### TC-PASTE-011: Revision Incremented After Paste

```
GIVEN document has revision R
WHEN apply_paste_selection is called successfully
THEN document revision is R + 1
```

---

### TC-PASTE-012: History Receives Pre-Paste State

```
GIVEN history is empty
WHEN apply_paste_selection is called successfully
THEN history can undo to pre-paste state
```

**Verification:**
```rust
let pre_paste = doc_signal.read().clone();
// ... paste ...
// Verify undo restores pre_paste state
```

---

## Test Implementation Template

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod copy_paste_tests {
    use super::*;

    fn make_doc_with_node(id: &str, x: f64, y: f64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new(id.to_string());
        let node = Node {
            kind: NodeKind::Rect,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            width: OrderedFloat(100.0),
            height: OrderedFloat(50.0),
            font_size: None,
            font_weight: None,
            locked: false,
            parent: None,
            dag_rank: None,
            tags: Vec::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        };
        let _ = doc.document.nodes.insert(node_id, node);
        doc
    }

    fn clear_clipboard() {
        CLIPBOARD.with(|s| *s.borrow_mut() = None);
    }

    #[test]
    fn given_empty_selection_when_copy_then_returns_false() {
        clear_clipboard();
        let doc = DiagramDocument::default();
        let doc_signal = Signal::new(doc);
        
        let result = apply_copy_selection(doc_signal);
        
        assert!(!result);
        CLIPBOARD.with(|s| assert!(s.borrow().is_none()));
    }

    // ... remaining tests following pattern
}
```

---

## Execution Order

1. Run copy tests first (TC-COPY-001 through TC-COPY-007)
2. Run paste tests (TC-PASTE-001 through TC-PASTE-012)
3. Each test clears clipboard in setup phase
4. Verify with: `cargo test --package diagram_tool copy_paste`
