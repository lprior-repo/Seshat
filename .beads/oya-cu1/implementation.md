# Implementation Summary - EDG-031

## Bead: oya-cu1
## Feature: Edge routing stable when endpoints swap order

## Changes Made

### File: `diagram_tool/src/geometry/mod.rs`

1. **Modified `orthogonal_route` function** (lines 952-978)
   - Changed the corner calculation from asymmetric `(to.x, from.y)` to symmetric `(min(from.x, to.x), max(from.y, to.y))`
   - This ensures that swapping source/target produces a reversed path, not a different geometry

2. **Added new tests for EDG-031** (lines 1020-1055)
   - `test_edge_routing_stable_when_endpoints_swap_order`: Verifies basic stability
   - `test_edge_routing_stable_different_start_point`: Verifies stability with different coordinates

3. **Updated existing test** (lines 975-990)
   - `test_edge_routing_orthogonal_l_shape`: Updated to expect vertical-first routing (new behavior)

## Behavior Change

**Before (asymmetric):**
- from (0,0) to (100,50): route = [(0,0), (100,0), (100,50)]
- from (100,50) to (0,0): route = [(100,50), (0,50), (0,0)]
- These are geometrically different paths!

**After (symmetric):**
- from (0,0) to (100,50): route = [(0,0), (0,50), (100,50)]
- from (100,50) to (0,0): route = [(100,50), (0,50), (0,0)]
- These are reverses of each other ✓

## Test Results
- All geometry tests pass: 314 passed
- EDG-031 stability tests: 2 passed

## Notes
- The routing style changed from "horizontal-first" to "vertical-first" 
- This is a behavioral change but maintains the L-shape pattern
- The important property (stability when swapping) is now guaranteed
