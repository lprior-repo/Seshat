# Martin Fowler Tests: Locked Node Exclusion (GEO-024)

## Test Strategy
Using Given-When-Then (GWT) format from Martin Fowler's testing approach.

---

## Test: GEO-024 - Locked Node Excluded from Selection Bounds

### Scenario 1: Single Unlocked Node Selection (Happy Path)

**GIVEN** a document with one unlocked node at (10, 10) size 100x50  
**AND** the node is selected in editor_state.selected_items  
**WHEN** selection_bounds() is called  
**THEN** return Some((10.0, 10.0, 100.0, 50.0))

---

### Scenario 2: Mixed Locked and Unlocked Nodes

**GIVEN** a document with:
- Node A at (0, 0) size 50x50, locked=false, selected
- Node B at (100, 0) size 50x50, locked=true, selected

**WHEN** selection_bounds() is called  
**THEN** return Some((0.0, 0.0, 50.0, 50.0))  
**AND** the returned bounds MUST NOT include Node B

---

### Scenario 3: All Selected Nodes Are Locked

**GIVEN** a document with:
- Node A at (0, 0) size 50x50, locked=true, selected
- Node B at (100, 0) size 50x50, locked=true, selected

**WHEN** selection_bounds() is called  
**THEN** return None (no unlocked nodes in selection)

---

### Scenario 4: No Nodes Selected

**GIVEN** a document with nodes but nothing in editor_state.selected_items  
**WHEN** selection_bounds() is called  
**THEN** return None

---

### Scenario 5: Selected Node ID List Excludes Locked

**GIVEN** a document with:
- Node A at (0, 0) size 50x50, locked=false, selected
- Node B at (100, 0) size 50x50, locked=true, selected
- Node C at (200, 0) size 50x50, locked=false, NOT selected

**WHEN** selected_node_ids() is called  
**THEN** return vec![NodeId("a")]  
**AND** NodeId("b") MUST NOT be in the returned list

---

### Scenario 6: Selection Box Drag with Locked Nodes Inside

**GIVEN** a document with:
- Node A at (10, 10) size 40x40, locked=false
- Node B at (30, 30) size 40x40, locked=true
- Both nodes within selection rectangle

**AND** user drags selection box to select both  
**WHEN** selection_bounds() is computed  
**THEN** bounds MUST only include Node A (unlocked)  
**AND** Node B (locked) MUST NOT affect the bounds

---

### Scenario 7: Locked Node Becomes Unlocked

**GIVEN** a document with:
- Node A at (0, 0) size 50x50, locked=true, selected

**WHEN** node A's locked property changes to false  
**AND** selection_bounds() is called  
**THEN** return Some((0.0, 0.0, 50.0, 50.0))

---

### Scenario 8: Document with Only Locked Nodes

**GIVEN** a document with multiple locked nodes  
**AND** all nodes are selected  
**WHEN** selection_bounds() is called  
**THEN** return None

---

## Implementation Hints

The fix is in `diagram_tool/src/ui/canvas/selection_geometry.rs`:

1. Modify `selected_node_ids()` to filter nodes where `node.locked == true`
2. The `selection_bounds()` function automatically benefits since it calls `selected_node_ids()`
3. Use `.filter(|n| !n.locked)` in the filter_map chain
