# Martin Fowler Test Patterns for bd-139: Clipboard Operations

**Bead ID**: bd-139
**Title**: clipboard: Implement clipboard operations (CLP-001 to CLP-010)
**Reference**: Martin Fowler's "Testing Styles" and "xUnit Test Patterns"

## Test Strategy

This section outlines the test patterns used for clipboard operations, following Martin Fowler's classification of test styles.

## Test Categories by Style

### 1. Behavior Verification Tests (Outcome-Based)

These tests verify the behavior of clipboard operations without caring about implementation details.

#### CLP-001: Single Node Copy/Paste
```rust
#[test]
fn given_single_node_selected_when_copy_then_paste_creates_duplicate() {
    // Setup: Create document with one node
    let mut doc = DiagramDocument::default();
    let node_id = NodeId::new("node-1".to_string());
    insert_test_node(&mut doc, node_id.clone());

    // Act: Select and copy
    doc.editor_state.selected_items.insert(node_id.clone().to_string());
    let copied = copy_selection_to_clipboard(&doc);

    // Assert: Clipboard has content
    assert!(copied);
    CLIPBOARD.with(|slot| {
        let clip = slot.borrow();
        assert!(clip.is_some());
        assert_eq!(clip.as_ref().unwrap().nodes.len(), 1);
    });

    // Act: Paste into new document
    let mut doc2 = DiagramDocument::default();
    let pasted = paste_from_clipboard(&mut doc2);

    // Assert: Duplicate created
    assert!(pasted);
    assert_eq!(doc2.document.nodes.len(), 1);
}
```

#### CLP-002: Multi-Node Copy with Edges
```rust
#[test]
fn given_connected_nodes_when_copy_then_preserves_topology() {
    // Setup: Create two nodes with edge
    let mut doc = DiagramDocument::default();
    let node1 = NodeId::new("node-1".to_string());
    let node2 = NodeId::new("node-2".to_string());
    let edge = EdgeId::new("edge-1".to_string());

    insert_test_node(&mut doc, node1.clone());
    insert_test_node(&mut doc, node2.clone());
    insert_test_edge(&mut doc, edge.clone(), &node1, &node2);

    // Act: Select both and copy
    doc.editor_state.selected_items.insert(node1.to_string());
    doc.editor_state.selected_items.insert(node2.to_string());
    copy_selection_to_clipboard(&doc);

    // Act: Paste twice
    let mut doc2 = DiagramDocument::default();
    paste_from_clipboard(&mut doc2);
    paste_from_clipboard(&mut doc2);

    // Assert: Topology preserved
    assert_eq!(doc2.document.nodes.len(), 4);
    assert_eq!(doc2.document.edges.len(), 2);

    // Assert: Edges connect pasted nodes, not originals
    for edge in doc2.document.edges.values() {
        assert!(doc2.document.nodes.contains_key(&edge.source));
        assert!(doc2.document.nodes.contains_key(&edge.target));
    }
}
```

### 2. State-Based Tests

These tests verify the state transitions during clipboard operations.

#### Empty Selection State (CLP-010)
```rust
#[test]
fn given_empty_selection_when_copy_then_returns_false() {
    let mut doc = DiagramDocument::default();
    // No selection

    let result = copy_selection_to_clipboard(&doc);
    assert!(!result);

    CLIPBOARD.with(|slot| {
        assert!(slot.borrow().is_none());
    });
}
```

#### Empty Clipboard State (CLP-017)
```rust
#[test]
fn given_empty_clipboard_when_paste_then_returns_false() {
    clear_clipboard();

    let mut doc = DiagramDocument::default();
    let result = paste_from_clipboard(&mut doc);

    assert!(!result);
    assert_eq!(doc.document.nodes.len(), 0);
}
```

### 3. Contract Tests (Design by Contract)

These tests verify preconditions, postconditions, and invariants.

#### Postcondition: Unique IDs (CLP-001)
```rust
#[test]
fn when_paste_then_generates_unique_ids() {
    let mut doc = create_test_document_with_node();
    select_first_node(&mut doc);
    copy_selection_to_clipboard(&doc);

    let mut doc2 = DiagramDocument::default();
    paste_from_clipboard(&mut doc2);

    // Postcondition: New ID is different from original
    let original_id = get_first_node_id(&doc);
    let pasted_id = get_first_node_id(&doc2);
    assert_ne!(original_id, pasted_id);
}
```

#### Invariant: No NaN Coordinates
```rust
#[test]
fn when_paste_then_coordinates_remain_finite() {
    let mut doc = create_test_document_with_node();
    select_first_node(&mut doc);
    copy_selection_to_clipboard(&doc);

    let mut doc2 = DiagramDocument::default();
    paste_from_clipboard(&mut doc2);

    // Invariant: All coordinates are finite
    for node in doc2.document.nodes.values() {
        assert!(node.x.0.is_finite());
        assert!(node.y.0.is_finite());
    }
}
```

### 4. Boundary/Edge Case Tests

#### Offset Accumulation (CLP-009)
```rust
#[test]
fn when_multiple_pastes_then_offset_increments() {
    let mut doc = create_test_document_with_node();
    select_first_node(&mut doc);
    copy_selection_to_clipboard(&doc);

    let mut doc2 = DiagramDocument::default();

    // First paste: 20px offset
    paste_from_clipboard(&mut doc2);
    let pos1 = get_first_node_position(&doc2);

    // Second paste: 40px offset
    paste_from_clipboard(&mut doc2);
    let pos2 = get_second_node_position(&doc2);

    // Third paste: 60px offset
    paste_from_clipboard(&mut doc2);
    let pos3 = get_third_node_position(&doc2);

    // Assert incremental offsets
    assert!((pos2.x - pos1.x - 20.0).abs() < 0.01);
    assert!((pos3.x - pos2.x - 20.0).abs() < 0.01);
}
```

#### Large Payload (CLP-016)
```rust
#[test]
fn when_paste_100_nodes_then_succeeds() {
    let mut doc = DiagramDocument::default();
    let mut node_ids = Vec::new();

    // Create 100 nodes
    for i in 0..100 {
        let id = NodeId::new(format!("node-{i}"));
        insert_test_node(&mut doc, id.clone());
        node_ids.push(id);
    }

    // Select all and copy
    for id in &node_ids {
        doc.editor_state.selected_items.insert(id.to_string());
    }
    copy_selection_to_clipboard(&doc);

    // Paste should succeed
    let mut doc2 = DiagramDocument::default();
    let result = paste_from_clipboard(&mut doc2);

    assert!(result);
    assert_eq!(doc2.document.nodes.len(), 100);
}
```

### 5. Integration Tests (End-to-End)

These tests verify clipboard operations work with other subsystems.

#### Undo/Redo Integration (CLP-011)
```rust
#[test]
fn when_paste_then_undo_redo_succeeds() {
    let mut doc = create_test_document_with_node();
    let mut history = History::default();
    select_first_node(&mut doc);
    copy_selection_to_clipboard(&doc);

    apply_paste_selection(doc, history);

    // Undo
    apply_undo(doc, history);
    assert_eq!(node_count(&doc), 1);

    // Redo
    apply_redo(doc, history);
    assert_eq!(node_count(&doc), 2);
}
```

#### Subgraph Parent Assignment (CLP-006, CLP-013)
```rust
#[test]
fn when_paste_into_subgraph_then_parent_assigned() {
    let mut doc = DiagramDocument::default();

    // Create subgraph
    let subgraph_id = NodeId::new("subgraph-1".to_string());
    insert_test_subgraph(&mut doc, subgraph_id.clone());

    // Create external node and copy
    let node_id = NodeId::new("node-1".to_string());
    insert_test_node(&mut doc, node_id.clone());
    select_node(&mut doc, &node_id);
    copy_selection_to_clipboard(&doc);

    // Click inside subgraph and paste
    // (implementation depends on click position handling)
    paste_from_clipboard(&mut doc);

    // Assert: Pasted node has subgraph as parent
    let pasted_node = find_pasted_node(&doc);
    assert_eq!(pasted_node.parent, Some(subgraph_id));
}
```

### 6. Adversarial Tests (Red Queen)

These tests attempt to break the clipboard operations.

#### Malformed Clipboard State
```rust
#[test]
fn when_clipboard_has_corrupted_data_then_handles_gracefully() {
    // Manually set corrupted clipboard state
    CLIPBOARD.with(|slot| {
        *slot.borrow_mut() = Some(ClipboardState {
            nodes: vec![], // Empty nodes with non-zero serial
            edges: vec![],
            paste_serial: 999,
        });
    });

    let mut doc = DiagramDocument::default();
    let result = paste_from_clipboard(&mut doc);

    // Should return false for empty nodes
    assert!(!result);
}
```

#### Concurrent Access
```rust
#[test]
fn when_concurrent_paste_then_serial_increments_correctly() {
    // Note: Rust's thread_local prevents true concurrent access
    // This test verifies serial increments work correctly

    let mut doc = create_test_document_with_node();
    select_first_node(&mut doc);
    copy_selection_to_clipboard(&doc);

    let mut doc2 = DiagramDocument::default();

    // Rapid pastes
    paste_from_clipboard(&mut doc2);
    paste_from_clipboard(&mut doc2);
    paste_from_clipboard(&mut doc2);

    // Verify all offsets are different
    let positions = get_all_node_positions(&doc2);
    let unique_positions: HashSet<_> = positions.iter().map(|p| (p.x, p.y)).collect();
    assert_eq!(unique_positions.len(), positions.len());
}
```

## Test Organization

### Unit Tests (`diagram_tool/src/ui/commands.rs`)
- Individual clipboard operations
- State management
- Edge cases

### Integration Tests (`diagram_tool/e2e/diagram.clipboard.spec.ts`)
- End-to-end workflows
- Keyboard shortcuts
- UI interaction
- Undo/redo integration

### Property Tests (Future)
- Clipboard round-trip preserves structure
- Offset accumulation is monotonic
- ID generation is unique

## Coverage Goals

- **Branch Coverage**: 100% for clipboard logic
- **Line Coverage**: 95%+ for clipboard module
- **Edge Cases**: All documented edge cases tested
- **Error Paths**: All Result return values tested

## Test Maintenance

When modifying clipboard behavior:
1. Update corresponding test cases
2. Verify all CLP-XXX tests still pass
3. Run E2E test suite
4. Check for regressions in related operations (history, selection)

## References

- Martin Fowler: "Testing Styles" - https://martinfowler.com/articles/testing-style.html
- xUnit Test Patterns: Refactoring Test Code - Meszaros, Gerard
- Design by Contract - Bertrand Meyer
