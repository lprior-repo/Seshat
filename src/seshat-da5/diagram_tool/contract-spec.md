# Contract Specification: seshat diagram_tool mutation helpers

This document defines the design-by-contract specifications for the mutation helpers module
(`mutation/ui_helpers.rs`) and the Z-order operations in `projection.rs`.

## Table of Contents

1. [Error Taxonomy](#error-taxonomy)
2. [UI Helpers Module](#ui-helpers-module)
   - [mutate_doc_signal](#mutate_doc_signal)
   - [mutate_editor_signal](#mutate_editor_signal)
   - [mutate_doc_with_history](#mutate_doc_with_history)
3. [Z-Order Operations](#z-order-operations)
   - [apply_bring_forward](#apply_bring_forward)
   - [apply_send_backward](#apply_send_backward)
   - [apply_bring_to_front](#apply_bring_to_front)
   - [apply_send_to_back](#apply_send_to_back)
4. [Violation Examples](#violation-examples)

---

## Error Taxonomy

### MutationError

The primary error type for all mutation operations in this module.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MutationError {
    /// Schema validation failed
    #[error("schema error: {0}")]
    Schema(String),
    /// Semantic validation failed
    #[error("semantic validation error: {0}")]
    Semantic(String),
    /// History operation failed (e.g., no more undos)
    #[error("history error: {0}")]
    History(String),
    /// Signal update failed
    #[error("signal error: {0}")]
    Signal(String),
}
```

### ZOrderError

Specific errors for Z-order operations.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ZOrderError {
    /// No nodes were specified for the operation
    #[error("no nodes specified")]
    NoNodesSpecified,
    /// All specified nodes are invalid or don't exist
    #[error("all nodes invalid or not found: {0}")]
    AllNodesInvalid(String),
    /// Z-order computation overflow
    #[error("z-index overflow")]
    ZIndexOverflow,
    /// Operation failed with underlying error
    #[error("operation failed: {0}")]
    OperationFailed(String),
}
```

---

## UI Helpers Module

### Overview

The `mutation/ui_helpers.rs` module provides three primary functions for document mutation
through Dioxus signals. All functions follow the functional-rust requirements:
- No `unwrap()` or `panic()` in source code
- No `mut` - all functions use immutable patterns with explicit cloning
- All functions return `Result<T, E>` for explicit error handling

---

## mutate_doc_signal

### Purpose

Applies validated document mutations via Dioxus signals. This function validates the
document after mutation and returns the updated document.

### Function Signature

```rust
/// Applies a validated document mutation via signal
///
/// # Arguments
/// * `current` - The current document state
/// * `transform` - A closure that transforms the document (FnOnce)
///
/// # Returns
/// * `Ok(DiagramDocument)` - The mutated document
/// * `Err(MutationError)` - If validation fails or transformation errors
///
/// # Preconditions
/// - P1: `current` must be a valid document (passes schema validation)
/// - P2: The `transform` closure must be deterministic (same input → same output)
///
/// # Postconditions
/// - Q1: Returns a document that passes schema validation
/// - Q2: Returns a document that passes semantic validation
/// - Q3: Document revision is incremented by exactly 1
/// - Q4: Original `current` document is unchanged (immutable)
/// - Q5: All node IDs present in input are preserved in output
/// - Q6: All edge references that were valid remain valid
pub fn mutate_doc_signal<F>(
    current: &DiagramDocument,
    transform: F,
) -> Result<DiagramDocument, MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>;
```

### Error Cases

| Error Type | Condition |
|------------|-----------|
| `MutationError::Schema` | Result fails schema validation (version, required fields) |
| `MutationError::Semantic` | Result fails semantic validation (cycles, dangling refs) |

### Violation Examples

```rust
// VIOLATION: Calling with invalid schema version
let doc = DiagramDocument { version: 999, .. };
let result = mutate_doc_signal(&doc, |d| Ok(d.clone()));
// Expected: Err(MutationError::Schema(_))
// Actual (BAD): Would panic or succeed incorrectly

// VIOLATION: Transform creates cycle in DAG
let result = mutate_doc_signal(&doc, |d| {
    // Create edge A→B then B→A creating a cycle
    Ok(invalid_doc_with_cycle)
});
// Expected: Err(MutationError::Schema("cycle error: ..."))
// Actual (BAD): Would panic or succeed incorrectly
```

---

## mutate_editor_signal

### Purpose

Applies non-validated editor state mutations. Unlike `mutate_doc_signal`, this function
does NOT validate the document content - it only handles editor state (camera, zoom,
selection, etc.) which is transient and doesn't require validation.

### Function Signature

```rust
/// Applies a non-validated editor state mutation via signal
///
/// # Arguments
/// * `current` - The current editor state (part of DiagramDocument)
/// * `transform` - A closure that transforms the editor state
///
/// # Returns
/// * `Ok(EditorState)` - The mutated editor state
/// * `Err(MutationError::Signal)` - If transformation fails
///
/// # Preconditions
/// - P1: `current` must be a valid EditorState
///
/// # Postconditions
/// - Q1: Returns a valid EditorState
/// - Q2: Original editor state is unchanged (immutable)
/// - Q3: Document content is NOT modified (only editor state)
pub fn mutate_editor_signal<F>(
    current: &EditorState,
    transform: F,
) -> Result<EditorState, MutationError>
where
    F: FnOnce(&EditorState) -> Result<EditorState, MutationError>;
```

### Error Cases

| Error Type | Condition |
|------------|-----------|
| `MutationError::Signal` | Transform closure returns an error |

### Violation Examples

```rust
// VIOLATION: Mutating document content through editor signal
let doc = DiagramDocument::default();
let result = mutate_editor_signal(&doc.editor_state, |state| {
    // BAD: This shouldn't be possible - editor signal only touches editor_state
    // Should only modify: camera_x, camera_y, zoom, selected_items, etc.
    Ok(modified_state)
});
// Expected: EditorState only, no document content changes
```

---

## mutate_doc_with_history

### Purpose

Applies validated mutations with full undo/redo history support. This is the primary
function for user-facing mutations that should be undoable.

### Function Signature

```rust
/// Applies a validated mutation with undo/redo history support
///
/// # Arguments
/// * `current` - The current document state
/// * `history` - The current history state for undo/redo
/// * `transform` - A closure that transforms the document
///
/// # Returns
/// * `Ok((DiagramDocument, History))` - The mutated document and updated history
/// * `Err(MutationError)` - If validation or transformation fails
///
/// # Preconditions
/// - P1: `current` must be a valid document
/// - P2: `history` must be a valid History struct
///
/// # Postconditions
/// - Q1: Returns a document that passes schema validation
/// - Q2: Returns a document that passes semantic validation  
/// - Q3: Document revision is incremented by exactly 1
/// - Q4: History has the previous state pushed to undo stack
/// - Q5: History redo stack is cleared (new timeline branch)
/// - Q6: Original `current` and `history` are unchanged (immutable)
/// - Q7: History is capped at MAX_HISTORY (100) entries
pub fn mutate_doc_with_history<F>(
    current: &DiagramDocument,
    history: &History,
    transform: F,
) -> Result<(DiagramDocument, History), MutationError>
where
    F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>;
```

### Error Cases

| Error Type | Condition |
|------------|-----------|
| `MutationError::Schema` | Result fails schema validation |
| `MutationError::Semantic` | Result fails semantic validation |
| `MutationError::History` | History operation failed |

### Invariants

| Invariant | Description |
|-----------|-------------|
| I1 | Undo stack contains documents in reverse chronological order |
| I2 | Redo stack contains documents in chronological order |
| I3 | After successful mutation: redo stack is empty |

### Violation Examples

```rust
// VIOLATION: History not updated correctly
let doc = DiagramDocument::default();
let history = History::new();
let result = mutate_doc_with_history(&doc, &history, |d| Ok(d.clone()));
// Expected: history.undo_stack has 1 entry, redo_stack is empty
// Actual (BAD): Either stack incorrect

// VIOLATION: Revision not incremented
let initial_rev = doc.revision;
let result = mutate_doc_with_history(&doc, &history, |d| Ok(d.clone()));
// Expected: result.revision == initial_rev.increment()
// Actual (BAD): Revision unchanged

// VIOLATION: Original state mutated
let original_doc = DiagramDocument::default();
let history = History::new();
let _ = mutate_doc_with_history(&original_doc, &history, |d| Ok(d.clone()));
// Expected: original_doc unchanged
// Actual (BAD): original_doc was modified
```

---

## Z-Order Operations

### Overview

The Z-order operations manipulate the stacking order (z-index) of nodes in a diagram.
All operations follow functional-rust patterns: they take an immutable state and
return a new state with the z-order applied.

### apply_bring_forward

Moves selected nodes one step forward in the z-order (toward front).

```rust
/// Applies BringForward operation - moves selected nodes one step toward front
///
/// # Arguments
/// * `state` - The current diagram projection (immutable)
/// * `ids` - List of node IDs to bring forward
///
/// # Returns
/// * `Ok(DiagramProjection)` - New projection with z-order applied
/// * `Err(ZOrderError)` - If operation fails
///
/// # Preconditions
/// - P1: `ids` must not be empty
///
/// # Postconditions
/// - Q1: Selected nodes have z-index increased by 1 (relative to their neighbors)
/// - Q2: Non-selected nodes z-indices unchanged
/// - Q3: All node IDs preserved
/// - Q4: All edge references preserved
/// - Q5: Returns new state, original unchanged
pub fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ZOrderError>;
```

### apply_send_backward

Moves selected nodes one step backward in the z-order (toward back).

```rust
/// Applies SendBackward operation - moves selected nodes one step toward back
///
/// # Arguments
/// * `state` - The current diagram projection (immutable)
/// * `ids` - List of node IDs to send backward
///
/// # Returns
/// * `Ok(DiagramProjection)` - New projection with z-order applied
/// * `Err(ZOrderError)` - If operation fails
///
/// # Preconditions
/// - P1: `ids` must not be empty
///
/// # Postconditions
/// - Q1: Selected nodes have z-index decreased by 1 (relative to their neighbors)
/// - Q2: Non-selected nodes z-indices unchanged
/// - Q3: All node IDs preserved
/// - Q4: All edge references preserved
/// - Q5: Returns new state, original unchanged
pub fn apply_send_backward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ZOrderError>;
```

### apply_bring_to_front

Moves selected nodes to the front of the z-order stack.

```rust
/// Applies BringToFront operation - moves selected nodes to front of z-order
///
/// # Arguments
/// * `state` - The current diagram projection (immutable)
/// * `ids` - List of node IDs to bring to front
///
/// # Returns
/// * `Ok(DiagramProjection)` - New projection with z-order applied
/// * `Err(ZOrderError)` - If operation fails
///
/// # Preconditions
/// - P1: `ids` must not be empty
///
/// # Postconditions
/// - Q1: All selected nodes have higher z-index than all non-selected nodes
/// - Q2: Relative order among selected nodes preserved
/// - Q3: All node IDs preserved
/// - Q4: All edge references preserved
/// - Q5: Returns new state, original unchanged
pub fn apply_bring_to_front(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ZOrderError>;
```

### apply_send_to_back

Moves selected nodes to the back of the z-order stack.

```rust
/// Applies SendToBack operation - moves selected nodes to back of z-order
///
/// # Arguments
/// * `state` - The current diagram projection (immutable)
/// * `ids` - List of node IDs to send to back
///
/// # Returns
/// * `Ok(DiagramProjection)` - New projection with z-order applied
/// * `Err(ZOrderError)` - If operation fails
///
/// # Preconditions
/// - P1: `ids` must not be empty
///
/// # Postconditions
/// - Q1: All selected nodes have lower z-index than all non-selected nodes
/// - Q2: Relative order among selected nodes preserved
/// - Q3: All node IDs preserved
/// - Q4: All edge references preserved
/// - Q5: Returns new state, original unchanged
pub fn apply_send_to_back(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ZOrderError>;
```

### Error Cases (Z-Order)

| Error Type | Condition |
|------------|-----------|
| `ZOrderError::NoNodesSpecified` | `ids` slice is empty |
| `ZOrderError::AllNodesInvalid` | All specified node IDs don't exist in the projection |
| `ZOrderError::ZIndexOverflow` | Z-index computation would overflow i64 |
| `ZOrderError::OperationFailed` | Underlying operation failed |

### Invariants (Z-Order)

| Invariant | Description |
|-----------|-------------|
| I1 | Z-indices are contiguous integers (no gaps) |
| I2 | Z-indices fit within i64 range |
| I3 | Node ID to node mapping is preserved |
| I4 | Edge ID to edge mapping is preserved |
| I5 | All edge source/target references remain valid |

### Violation Examples

```rust
// VIOLATION: Empty ids list
let state = DiagramProjection::empty();
let result = apply_bring_forward(&state, &[]);
// Expected: Err(ZOrderError::NoNodesSpecified)
// Actual (BAD): Would panic or succeed with no-op

// VIOLATION: All nodes don't exist
let result = apply_bring_forward(&state, &["nonexistent".to_string()]);
// Expected: Err(ZOrderError::AllNodesInvalid(_))
// Actual (BAD): Would panic or succeed with no-op

// VIOLATION: Original state mutated
let state = DiagramProjection::empty();
let original_hash = projection_hash(&state).unwrap();
let _ = apply_bring_forward(&state, &["node".to_string()]);
let new_hash = projection_hash(&state).unwrap();
// Expected: original_hash == new_hash (state unchanged)
// Actual (BAD): Hashes differ, state was mutated

// VIOLATION: Z-indices not contiguous after operation
let result = apply_bring_to_front(&state, &["a".to_string(), "b".to_string()]);
let z_indices: Vec<_> = result.nodes.values().map(|n| n.z_index).collect();
let mut sorted = z_indices.clone();
sorted.sort();
let expected: Vec<_> = (0..sorted.len() as i64).collect();
// Expected: sorted == expected
// Actual (BAD): Gaps in z-index sequence
```

---

## Implementation Notes

### Functional Rust Patterns

All functions in this module MUST follow:

1. **No `mut` parameters** - All input data is consumed immutably
2. **No `unwrap()`** - Use `?` operator with explicit error types
3. **No `panic!()`** - Return errors instead of panicking
4. **Clone on output** - Return new data, don't mutate inputs
5. **Result-based errors** - All fallible operations return `Result<T, E>`

### Example Pattern

```rust
// GOOD: Immutable pattern
pub fn apply_bring_forward(
    state: DiagramProjection,
    ids: &[String],
) -> Result<DiagramProjection, ZOrderError> {
    // Convert ids to NodeIds, filter to only existing nodes
    let selected: BTreeSet<NodeId> = ids
        .iter()
        .map(|s| NodeId::new(s.clone()))
        .filter(|id| state.has_node(id))
        .collect();
    
    // Check preconditions
    if ids.is_empty() {
        return Err(ZOrderError::NoNodesSpecified);
    }
    if selected.is_empty() {
        return Err(ZOrderError::AllNodesInvalid(ids.join(", ")));
    }
    
    // Clone to create new state (no mut)
    let mut new_nodes = state.nodes.clone();
    
    // ... perform operation ...
    
    // Return new state, original unchanged
    Ok(DiagramProjection {
        nodes: new_nodes,
        ..state
    })
}

// BAD: Mutable pattern (DO NOT USE)
pub fn apply_bring_forward_bad(
    mut state: DiagramProjection,
    ids: &[String],
) -> DiagramProjection {
    // VIOLATION: Uses mut
    // VIOLATION: No error handling
    // VIOLATION: Mutates input
    for id in ids {
        if let Some(node) = state.nodes.get_mut(id) {
            node.z_index += 1;
        }
    }
    state
}
```

---

## Test Strategy

### Contract Testing

Each function should have tests verifying:

1. **Happy path** - Valid inputs produce expected outputs
2. **Precondition violations** - Invalid inputs return correct errors
3. **Postcondition verification** - Output satisfies all guarantees
4. **Immutability** - Original inputs unchanged after call
5. **Edge cases** - Empty inputs, boundary values, etc.

### Property-Based Testing

Use proptest to verify:
- Repeated application produces valid states
- Round-trip operations maintain invariants
- Large inputs don't cause overflow/performance issues

---

## Summary Table

| Function | Input | Output | Key Preconditions | Key Postconditions |
|----------|-------|--------|-------------------|-------------------|
| `mutate_doc_signal` | `&DiagramDocument`, `transform` | `Result<DiagramDocument, MutationError>` | Valid document | Schema+semantic valid, revision+1 |
| `mutate_editor_signal` | `&EditorState`, `transform` | `Result<EditorState, MutationError>` | Valid EditorState | Valid EditorState |
| `mutate_doc_with_history` | `&DiagramDocument`, `&History`, `transform` | `Result<(DiagramDocument, History), MutationError>` | Valid document+history | Schema+semantic valid, history updated |
| `apply_bring_forward` | `DiagramProjection`, `&[String]` | `Result<DiagramProjection, ZOrderError>` | Non-empty ids | Z-index +1, nodes preserved |
| `apply_send_backward` | `DiagramProjection`, `&[String]` | `Result<DiagramProjection, ZOrderError>` | Non-empty ids | Z-index -1, nodes preserved |
| `apply_bring_to_front` | `DiagramProjection`, `&[String]` | `Result<DiagramProjection, ZOrderError>` | Non-empty ids | Selected at front, nodes preserved |
| `apply_send_to_back` | `DiagramProjection`, `&[String]` | `Result<DiagramProjection, ZOrderError>` | Non-empty ids | Selected at back, nodes preserved |
