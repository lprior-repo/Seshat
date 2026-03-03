# Contract Specification for bd-139: Clipboard Operations

**Bead ID**: bd-139
**Title**: clipboard: Implement clipboard operations (CLP-001 to CLP-010)
**Status**: Draft
**Version**: 1.0.0

## Overview

This contract specifies the behavior of clipboard operations in the Seshat diagram tool, including copy, cut, paste, and duplicate operations for nodes and edges.

## Test Categories

### CLP-001: Copy Single Node
**Requirement**: Copying a single node to clipboard and pasting creates a duplicate node with a new ID.

**Preconditions**:
- At least one node exists in the document
- Node is selected

**Postconditions**:
- After copy + paste, node count increases by 1
- Pasted node has a unique ID (different from original)
- Pasted node is selected after paste
- Original node remains unchanged

**Invariants**:
- No NaN or Infinity in coordinates
- Positive dimensions
- Edge references valid nodes

### CLP-002: Copy Multiple Nodes with Edges
**Requirement**: Copying multiple nodes with their connecting edges preserves the topology.

**Preconditions**:
- At least two nodes exist with at least one edge between them
- All nodes in the subgraph are selected

**Postconditions**:
- After copy + paste, node count doubles (original + copy)
- After copy + paste, edge count doubles (original + copy)
- Pasted edges connect the pasted nodes (not original nodes)
- All pasted nodes have new unique IDs

### CLP-003: Copy with Subgraph Structure
**Requirement**: Copying a subgraph with child nodes preserves parent-child relationships.

**Preconditions**:
- A subgraph exists with at least one child node
- Both parent subgraph and child are selected

**Postconditions**:
- After copy + paste, subgraph structure is preserved
- Child node's parent reference points to the new subgraph ID
- Parent-child relationship maintained in paste

### CLP-004: Cut Operation
**Requirement**: Cut operation (copy + delete) removes original and allows paste.

**Preconditions**:
- At least one node exists
- Node is selected

**Postconditions**:
- After copy + delete, node count is 0
- After paste, node count is 1
- Pasted node has new unique ID

**Note**: The current implementation uses copy + delete as a workaround for Ctrl+X.

### CLP-005: Paste Operation
**Requirement**: Pasting from clipboard creates new nodes with offset positions.

**Preconditions**:
- Clipboard contains at least one node
- Document may be empty or have existing content

**Postconditions**:
- Each paste creates new nodes with unique IDs
- First paste applies 20px offset
- Subsequent pastes apply additional 20px increments (20px * serial)
- Pasted nodes are selected

### CLP-006: Paste at Position
**Requirement**: Pasting nodes can assign parent based on click position.

**Preconditions**:
- Clipboard contains nodes
- A subgraph container exists

**Postconditions**:
- If click is inside subgraph, pasted nodes become children
- Parent assignment respects container boundaries

### CLP-007: Clipboard Persistence
**Requirement**: Clipboard content persists across operations within the session.

**Preconditions**:
- Clipboard has content
- Multiple operations performed

**Postconditions**:
- Clipboard retains content until new copy operation
- Multiple pastes work from single copy

### CLP-008: Cross-Diagram Paste
**Requirement**: Clipboard serialization supports round-trip operations.

**Preconditions**:
- Nodes and edges are copied to clipboard

**Postconditions**:
- Clipboard state can be serialized
- Paste operation reproduces the copied structure
- No data loss in serialization round-trip

### CLP-009: Clipboard Format
**Requirement**: Clipboard uses thread-local storage with proper structure.

**Preconditions**:
- Clipboard is empty or contains data

**Postconditions**:
- ClipboardState contains nodes vector and edges vector
- paste_serial increments with each paste
- Format is compatible with paste_from function

### CLP-010: Empty Clipboard Handling
**Requirement**: Empty clipboard operations are handled gracefully.

**Preconditions**:
- Clipboard is empty
- No selection or empty selection

**Postconditions**:
- Copy with empty selection returns false
- Paste with empty clipboard returns false
- No nodes created on paste with empty clipboard
- No crashes or errors

## Additional Tests

### CLP-011: Undo/Redo Support
**Requirement**: Paste operations can be undone and redone.

### CLP-012: Duplicate with Undo
**Requirement**: Duplicate (Ctrl+D) operations can be undone.

### CLP-013: Parent Assignment on Paste
**Requirement**: Pasted nodes can be assigned to subgraph parent.

### CLP-014: Drag-Drop External Files
**Requirement**: Canvas handles drag events for visual feedback.

### CLP-015: Internal Field Exclusion
**Requirement**: Clipboard serialization excludes internal Rust fields.

### CLP-016: Large Payload Handling
**Requirement**: Paste handles large payloads (100+ nodes) gracefully.

### CLP-017: Empty Clipboard Paste
**Requirement**: Paste with empty clipboard after reload creates no nodes.

## Error Handling

All clipboard operations must:
- Return `false` on failure (no selection, empty clipboard)
- Never panic, unwrap, or use expect()
- Use Result types for fallible operations
- Handle edge cases gracefully

## Performance

- Copy operation: O(n + m) where n = nodes, m = edges
- Paste operation: O(n + m) for node/edge insertion
- Memory: Clipboard stores full copy of nodes and edges

## Security

- Clipboard content must not expose internal Rust pointers
- Serialization must exclude raw memory addresses
- No code execution through clipboard data

## Implementation References

- Main implementation: `diagram_tool/src/ui/commands.rs`
- Thread-local clipboard: `CLIPBOARD` static
- Test harness: `diagram_tool/src/test_harness.rs` (TestCategory::Clp)
- E2E tests: `diagram_tool/e2e/diagram.clipboard.spec.ts`
