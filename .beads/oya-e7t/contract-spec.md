# Contract Specification: Exclude Locked Nodes from Selection Bounding Box (GEO-024)

## EARS Requirements

### EARS-001: Locked Node Exclusion in Selection
**E**liminate **A**ll **R**ecursive **S**election of locked nodes.

**EARS Syntax:**
```
WHERE the diagram document contains nodes
AND some nodes have locked=true
WHEN selection_bounds() is called
THEN the returned bounding box MUST exclude all nodes where node.locked == true
```

### EARS-002: Selected Node IDs Must Filter Locked
```
WHERE selected_node_ids() is called
AND the document contains selected items with locked nodes
THEN the returned list MUST NOT contain any NodeId where node.locked == true
```

### EARS-003: Empty Selection After Locked Filter
```
WHERE all selected nodes are locked
WHEN selection_bounds() is called
THEN return None (empty selection bounds)
```

---

## KIRK Contracts

### Contract: Filter Locked from SelectedNodeIds

**Invariant (KIRK-style):**
```
FORALL node_id IN selected_node_ids(doc):
  REQUIRE doc.document.nodes.get(node_id).locked == false
```

**Preconditions:**
- `doc` must be a valid `DiagramDocument`
- `doc.document.nodes` must contain the referenced node IDs

**Postconditions:**
- Returned `Vec<NodeId>` contains ONLY nodes where `node.locked == false`
- If all selected nodes are locked, return empty `Vec`
- Selection bounds computed from filtered nodes only

---

## Implementation Notes

### Current Implementation (selection_geometry.rs)
```rust
pub(super) fn selected_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let nid = NodeId::new(id.clone());
            doc.document.nodes.contains_key(&nid).then_some(nid)
        })
        .collect()
}
```

### Required Change
Add filter to exclude nodes where `locked == true`:
```rust
pub(super) fn selected_node_ids(doc: &DiagramDocument) -> Vec<NodeId> {
    doc.editor_state
        .selected_items
        .iter()
        .filter_map(|id| {
            let nid = NodeId::new(id.clone());
            // NEW: Filter out locked nodes
            doc.document.nodes.get(&nid)
                .filter(|n| !n.locked)
                .map(|_| nid)
        })
        .collect()
}
```

---

## Test Coverage Requirements

- GEO-024: Locked nodes excluded from selection_bounds()
- Verify selection_bounds() computes bbox using only unlocked nodes
- Verify empty selection when all selected nodes are locked
