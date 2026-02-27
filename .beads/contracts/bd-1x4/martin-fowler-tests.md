# Martin Fowler Test Plan: Copy/Paste Toolbar Buttons

**Bead ID:** bd-1x4  
**Test Framework:** Dioxus testing with `data-testid` selectors

---

## 1. Copy Button Rendering

### 1.1 Copy button renders in toolbar
**Given** the Toolbar component is mounted  
**When** the DOM renders  
**Then** a button with `data-testid="toolbar-copy"` exists  
**And** the button text is "Copy"

---

### 1.2 Copy button disabled with no selection
**Given** `selected_count == 0`  
**When** the Toolbar renders  
**Then** button `toolbar-copy` has `disabled=true`

---

### 1.3 Copy button enabled with selection
**Given** `selected_count == 1`  
**When** the Toolbar renders  
**Then** button `toolbar-copy` has `disabled=false`

---

### 1.4 Copy button enabled with multiple selections
**Given** `selected_count >= 2`  
**When** the Toolbar renders  
**Then** button `toolbar-copy` has `disabled=false`

---

## 2. Paste Button Rendering

### 2.1 Paste button renders in toolbar
**Given** the Toolbar component is mounted  
**When** the DOM renders  
**Then** a button with `data-testid="toolbar-paste"` exists  
**And** the button text is "Paste"

---

### 2.2 Paste button disabled with empty clipboard
**Given** clipboard has never been populated  
**When** the Toolbar renders  
**Then** button `toolbar-paste` has `disabled=true`

---

### 2.3 Paste button enabled after copy
**Given** user has copied a selection  
**When** the Toolbar re-renders  
**Then** button `toolbar-paste` has `disabled=false`

---

### 2.4 Paste button disabled after clipboard cleared
**Given** clipboard was populated  
**When** clipboard becomes empty (if clear feature added)  
**Then** button `toolbar-paste` has `disabled=true`

---

## 3. Copy Action Behavior

### 3.1 Copy populates clipboard from selection
**Given** document has node A selected (`selected_count == 1`)  
**When** user clicks `toolbar-copy`  
**Then** `apply_copy_selection()` returns `true`  
**And** clipboard contains node A

---

### 3.2 Copy includes connected edges
**Given** document has nodes A and B selected with edge E between them  
**When** user clicks `toolbar-copy`  
**Then** clipboard contains nodes A and B  
**And** clipboard contains edge E

---

### 3.3 Copy excludes partial edges
**Given** document has node A selected but not node B  
**And** edge E connects A to B  
**When** user clicks `toolbar-copy`  
**Then** clipboard contains node A  
**And** clipboard does NOT contain edge E

---

### 3.4 Copy does not mutate document
**Given** document has 3 nodes with 1 selected  
**When** user clicks `toolbar-copy`  
**Then** document node count remains 3  
**And** document revision unchanged

---

## 4. Paste Action Behavior

### 4.1 Paste creates new nodes
**Given** clipboard contains node A  
**When** user clicks `toolbar-paste`  
**Then** `apply_paste_selection()` returns `true`  
**And** document has 1 additional node (new UUID)

---

### 4.2 Paste creates new edges
**Given** clipboard contains nodes A, B and edge E  
**When** user clicks `toolbar-paste`  
**Then** document has 2 additional nodes  
**And** document has 1 additional edge (remapped source/target)

---

### 4.3 Paste offsets position
**Given** clipboard contains node A at position (100, 100)  
**When** user clicks `toolbar-paste` first time  
**Then** pasted node is at position (120, 120)  
**When** user clicks `toolbar-paste` again  
**Then** second pasted node is at position (140, 140)

---

### 4.4 Paste updates selection
**Given** clipboard contains nodes A and B  
**And** document currently has node C selected  
**When** user clicks `toolbar-paste`  
**Then** selection becomes pasted nodes A' and B'  
**And** node C is no longer selected

---

### 4.5 Paste pushes history
**Given** history stack has N entries  
**When** user clicks `toolbar-paste`  
**Then** history stack has N+1 entries  
**And** undo restores pre-paste state

---

## 5. Integration Scenarios

### 5.1 Full copy-paste workflow
**Given** document has nodes A, B, C with no selection  
**When** user selects node A  
**Then** `toolbar-copy` becomes enabled  
**When** user clicks `toolbar-copy`  
**Then** `toolbar-paste` becomes enabled  
**When** user clicks `toolbar-paste`  
**Then** document has 4 nodes  
**And** pasted node A' is selected

---

### 5.2 Copy-paste with edges preserves connections
**Given** document has nodes A, B with edge A→B, both selected  
**When** user copies and pastes  
**Then** pasted nodes A' and B' exist  
**And** edge A'→B' exists  
**And** original edge A→B still exists

---

### 5.3 Multiple paste operations
**Given** clipboard contains node A at (50, 50)  
**When** user pastes 3 times  
**Then** 3 new nodes exist at (70, 70), (90, 90), (110, 110)  
**And** last pasted node is selected

---

## 6. Edge Cases

### 6.1 Copy empty selection returns false
**Given** `selected_count == 0`  
**When** `apply_copy_selection()` called directly  
**Then** returns `false`  
**And** clipboard unchanged

---

### 6.2 Paste empty clipboard returns false
**Given** clipboard is `None`  
**When** `apply_paste_selection()` called directly  
**Then** returns `false`  
**And** document unchanged

---

### 6.3 Paste clipboard with empty nodes returns false
**Given** clipboard is `Some(ClipboardState { nodes: vec![], edges: vec![], paste_serial: 0 })`  
**When** `apply_paste_selection()` called  
**Then** returns `false`

---

## 7. Accessibility

### 7.1 Buttons have discernible text
**Given** toolbar is rendered  
**When** inspecting `toolbar-copy` and `toolbar-paste`  
**Then** each button has text content "Copy" or "Paste"

---

### 7.2 Disabled state perceivable
**Given** `toolbar-copy` is disabled  
**When** user inspects the button  
**Then** button has `disabled` attribute  
**And** visual style indicates disabled state (opacity or color change)

---

## 8. Test Selector Reference

| Test ID | Selector | Purpose |
|---------|----------|---------|
| `toolbar-copy` | `button[data-testid="toolbar-copy"]` | Copy button |
| `toolbar-paste` | `button[data-testid="toolbar-paste"]` | Paste button |
| `selected-count` | `span[data-testid="selected-count"]` | Selection counter |
| `counter-selected` | `span[data-testid="counter-selected"]` | Selection text |

---

## 9. Test Data Builders

```rust
fn doc_with_selected_node() -> DiagramDocument {
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("test-node".to_string());
    doc.document.nodes.insert(node_id.clone(), default_node());
    doc.editor_state.selected_items.insert(node_id.to_string());
    doc
}

fn doc_with_two_connected_nodes() -> DiagramDocument {
    // Creates A, B with edge A→B, both selected
}
```
