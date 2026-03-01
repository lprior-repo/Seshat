---
bead_id: bd-lq3
bead_title: tests: Implement SUB subgraph tests 2/4
phase: p0
updated_at: 2026-03-01T22:55:00Z
---

# Contract: SUB Subgraph Tests 2/4

## Summary
Implement 5 subgraph (container) tests focusing on parent-child relationships and ID management.

## Tests Required

### SUB-006: Delete container reparents children
- **Given**: A subgraph containing 2 text nodes
- **When**: User deletes the subgraph (container) only
- **Then**: Child nodes remain in document, now at root level (reparented)

### SUB-007: Duplicate container remaps IDs
- **Given**: A subgraph containing 2 text nodes with edges
- **When**: User duplicates the container (copy-paste)
- **Then**: New container and all children have new unique IDs, edges remapped to new nodes

### SUB-008: Drag child into container
- **Given**: A root-level text node and a separate subgraph container
- **When**: User drags the node into the container bounds
- **Then**: Node becomes a child of the container, position adjusted relative to container

### SUB-009: Drag child out becomes root
- **Given**: A subgraph container with a child text node
- **When**: User drags the child node outside container bounds
- **Then**: Node becomes root-level, no longer a child of container

### SUB-010: Drag across overlapping containers
- **Given**: Two overlapping subgraph containers with a node in container A
- **When**: User drags the node from container A into the overlapping region with container B
- **Then**: Node reparents to container B (the target of the drop)

## Acceptance Criteria
- All 5 tests pass without page errors
- Tests use freshStart() for clean state
- Tests use runEffect/runEffectsSequential for deterministic operations
- Tests verify node counts before and after operations
- Tests verify expected parent-child relationships through position checks

## Implementation Notes
- File location: `diagram_tool/e2e/diagram.subgraph-behavior.spec.ts`
- Follow patterns from existing subgraph tests in `diagram.subgraph-resize.spec.ts`
- Use helpers: freshStart, createTextNode, nodeCount, nodeFrameByLabel
