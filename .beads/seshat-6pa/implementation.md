bead_id: seshat-6pa
bead_title: Implement subgraph parent validation (SUB-001)
phase: implementation
updated_at: 2026-03-14T18:30:00Z

## Implementation Summary

The subgraph parent validation (SUB-001) is already implemented in the codebase:

### Files Modified
- None required - validation already exists

### Existing Implementation

1. **validation.rs** (lines 78-100):
   - Validates that any node with a parent must have that parent be a Subgraph
   - Returns ValidationIssue with code "invalid-parent" if violated

2. **reparenting.rs** (lines 31-38, 102-103):
   - `validate_parent_is_subgraph()` function checks parent node kind
   - Called in `set_node_parent_ext()` before allowing reparenting
   - Returns Error::InvalidNodeType if parent is not a Subgraph

### Test Coverage
- validation.rs has tests:
  - `given_node_with_non_subgraph_parent_when_validated_then_invalid_parent_error`
  - `given_node_with_existing_subgraph_parent_when_validated_then_no_invalid_parent_issue`

### Contract Compliance
- [x] P1: Parent node must exist - validated in reparenting.rs
- [x] P2: Parent must be Subgraph - validated in both validation.rs and reparenting.rs
- [x] P3: Child must exist - validated in reparenting.rs
- [x] Q1: After reparenting, parent reference updated - handled by set_node_parent
- [x] Q2: Validation passes for valid Subgraph parent - tested in validation.rs

### Note
This bead was already implemented. The validation logic prevents nodes from being reparented to non-Subgraph nodes, maintaining valid hierarchy.
