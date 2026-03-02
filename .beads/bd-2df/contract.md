---
bead_id: bd-2df
bead_title: tests: Implement EDG edge routing tests 3/4
phase: p0
updated_at: 2026-03-02T00:20:00Z
---

# Contract: EDG Edge Routing Tests 3/4

## Summary
Implement 5 edge routing tests focusing on container/subgraph interactions and edge stability.

## Tests Required

### EDG-011: Edge between nodes in same container
- **Given**: A subgraph container with 2 text nodes inside
- **When**: User creates an edge between the two nodes inside the container
- **Then**: Edge is created successfully, edge count = 1, no page errors

### EDG-012: Edge crossing container boundary
- **Given**: A subgraph container with 1 text node inside, and 1 text node outside the container
- **When**: User creates an edge from the inside node to the outside node
- **Then**: Edge is created successfully crossing the container boundary, edge count = 1, no page errors

### EDG-013: Reparent node with edges
- **Given**: A subgraph container with 2 text nodes inside, connected by an edge
- **When**: User drags one of the nodes outside the container boundary
- **Then**: Node is reparented, edge remains valid (not orphaned), edge count = 1, no page errors

### EDG-014: Edge routing stable on overlapping nodes (horizontal)
- **Given**: 4 nodes arranged such that edges cross at the same point (2 horizontal edges)
- **When**: User clicks at the intersection point repeatedly
- **Then**: Same edge is selected each time (deterministic hit-selection)

### EDG-015: Edge routing stable on overlapping nodes (vertical)
- **Given**: 4 nodes arranged such that edges cross at the same point (2 vertical edges)
- **When**: User clicks at the intersection point repeatedly
- **Then**: Same edge is selected each time (deterministic hit-selection)

## Acceptance Criteria
- All 5 tests pass without page errors
- Tests use freshStart() for clean state
- Tests use runEffect/runEffectsSequential for deterministic operations
- Tests verify edge counts before and after operations
- Tests verify expected selection behavior
- No console errors or page errors during test execution

## Implementation Notes
- File location: `diagram_tool/e2e/diagram.edges-and-routing.spec.ts`
- Follow patterns from existing edge tests in the same file
- Use helpers: freshStart, createTextNode, edgeCount, expectEdgeCount, selectedCount, trapPageErrors
- Use createSubgraphContainer helper from diagram.subgraph-behavior.spec.ts for container tests
- Use nodeCenters helper for getting node positions for edge clicks
- For container creation, follow the pattern in diagram.subgraph-behavior.spec.ts lines 41-61
