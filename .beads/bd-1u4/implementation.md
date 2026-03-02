bead_id: bd-1u4
bead_title: tests: Implement EDG edge routing tests 4/4
phase: p1
updated_at: 2026-03-01T18:30:00Z

# Implementation: EDG Edge Routing Tests 4/4

## Summary

Added 5 new edge routing tests to `diagram_tool/e2e/diagram.edges-and-routing.spec.ts` covering:
1. Self-loop edge rejection
2. Curved edge hit-testing along quadratic bezier path
3. Thin horizontal edge hit-testing at zoom levels
4. Step-routed edge hit-testing
5. Sharp diagonal edge hit-testing

## Changes Made

### File: `diagram_tool/e2e/diagram.edges-and-routing.spec.ts`

Added tests EDG-016 through EDG-020:

#### EDG-016: Self-loop edge rejection
- Tests that creating an edge from a node to itself is rejected
- Creates a single node, clicks it twice in edge mode
- Verifies edge count remains 0 (self-loops violate DAG constraints)

#### EDG-017: Curved edge hit-testing
- Tests hit-testing along quadratic bezier curve path
- Sets arrow type to "curved" before creating edge
- Clicks at various points along the curve (peak and below peak)
- Verifies edge is selectable at curve points

#### EDG-018: Thin horizontal edge zoom hit-testing
- Complements existing vertical edge zoom test
- Creates horizontal edge and tests selection at 50%, 100%, 200%, 300% zoom
- Verifies consistent hit-testing across zoom levels

#### EDG-019: Step-routed edge hit-testing
- Tests hit-testing on step-routed edges (manhattan routing)
- Sets arrow type to "step" before creating edge
- Clicks on vertical segment and corner points
- Verifies edge is hittable at step path segments

#### EDG-020: Sharp diagonal edge hit-testing
- Tests hit-testing on sharp (straight diagonal) edges
- Sets arrow type to "sharp" before creating edge
- Creates diagonal edge between nodes at different positions
- Clicks at 1/4, 1/2, and 3/4 points along diagonal
- Verifies edge is hittable along entire path

## Test Patterns Used

- `freshStart(page)` - Clean browser state
- `clearCanvasOverlays(page)` - Close panels
- `createTextNode(page, canvas, x, y)` - Node creation
- `edgeClick(page, x, y)` - Click for edge selection
- `extrema(centers)` - Get left/right/top/bottom node positions
- `trapPageErrors(page)` - Capture console errors
- `resetZoom(page)` - Reset zoom to 100%
- `zoomInToAtLeast(page, targetPercent)` - Zoom to target level

## Verification

All tests:
- Follow existing naming conventions
- Use `@baseline` tag
- Trap and assert zero page errors
- Use deterministic patterns (no arbitrary timeouts)
- Follow existing test structure in the file
