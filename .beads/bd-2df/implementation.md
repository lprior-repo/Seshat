---
bead_id: bd-2df
bead_title: tests: Implement EDG edge routing tests 3/4
phase: p1
updated_at: 2026-03-02T00:21:00Z
---

# Implementation: EDG Edge Routing Tests 3/4

## Summary
Added 5 new edge routing tests to `diagram_tool/e2e/diagram.edges-and-routing.spec.ts` focusing on container/subgraph interactions and edge selection stability.

## Changes Made

### File Modified
- `diagram_tool/e2e/diagram.edges-and-routing.spec.ts`

### Imports Added
- `canvas` - helper for getting canvas locator
- `waitForNoRebuildOverlay` - helper for waiting for rebuild completion

### Tests Implemented

#### EDG-011: Edge between nodes in same container
- Creates a subgraph container with 2 text nodes inside
- Creates an edge between the two nodes inside the container
- Verifies edge is created successfully (edge count = 1)
- Verifies no page errors

#### EDG-012: Edge crossing container boundary
- Creates a subgraph container with 1 text node inside
- Creates 1 text node outside the container
- Creates an edge from the inside node to the outside node
- Verifies edge is created successfully crossing the container boundary
- Verifies no page errors

#### EDG-013: Reparent node with connected edge produces valid state
- Creates a subgraph container with 2 text nodes connected by an edge
- Drags one node outside the container boundary
- Verifies node count remains 3 (no nodes lost)
- Verifies edge still exists (not orphaned during reparent)
- Verifies no page errors

#### EDG-014: Horizontal edge overlap hit-selection is deterministic
- Creates 4 nodes arranged for two horizontal edges that overlap
- Creates both horizontal edges
- Clicks at the overlap point multiple times
- Verifies same edge is selected each time through delete/undo pattern
- Verifies no page errors

#### EDG-015: Vertical edge overlap hit-selection is deterministic
- Creates 4 nodes arranged for two vertical edges that overlap
- Creates both vertical edges
- Clicks at the overlap point multiple times
- Verifies same edge is selected each time through delete/undo pattern
- Verifies no page errors

## Test Patterns Used
- `freshStart()` for clean state
- `runEffect()` and `runEffectsSequential()` for deterministic operations
- `trapPageErrors()` for error detection
- `createTextNode()` for node creation
- `nodeCenters()` for getting node positions
- `expectEdgeCount()` and `expectNodeCount()` for assertions
- Subgraph container creation following `diagram.subgraph-behavior.spec.ts` pattern

## Verification
All tests follow existing patterns in the file and use the standard e2e test helpers.
