bead_id: bd-2dg
bead_title: tests: Implement SEL selection tests 1/5
phase: p1
updated_at: 2026-03-01T21:35:00Z

# Implementation: Selection Tests (bd-2dg)

## Summary
Added 5 new selection tests to `diagram_tool/e2e/diagram.nodes-and-selection.spec.ts` covering core selection behaviors.

## Tests Implemented

### SEL-001: Click node selects @baseline
- **Location**: Line 179-199
- **Behavior**: Clicking a node selects it; clicking another node replaces selection
- **Verification**: `expectSelectedCount(page, 1)` after each click

### SEL-002: Click empty clears selection @baseline
- **Location**: Line 201-227
- **Behavior**: Clicking on empty canvas area with select tool active clears selection
- **Verification**: `expectSelectedCount(page, 0)` after empty click

### SEL-003: Shift-click adds to selection @baseline
- **Location**: Line 229-262
- **Behavior**: Shift-clicking unselected nodes adds them to existing selection
- **Verification**: Selection count increments from 1 -> 2 -> 3

### SEL-004: Marquee drag selects nodes within rectangle @baseline
- **Location**: Line 264-297
- **Behavior**: Right-to-left marquee (intersect mode) selects nodes that intersect
- **Verification**: Both nodes selected after marquee drag

### SEL-005: Marquee left-to-right requires full containment @baseline
- **Location**: Line 299-345
- **Behavior**: Left-to-right marquee only selects nodes fully contained
- **Verification**: Partial overlap selects 0, full containment selects 1

## Code Changes
- File: `/home/lewis/src/seshat/diagram_tool/e2e/diagram.nodes-and-selection.spec.ts`
- Added 5 new tests using existing helper functions
- All tests use `@baseline` tag for stability tracking
- All tests trap page errors for reliability

## Patterns Followed
- Uses `freshStart()` for clean state
- Uses `clearCanvasOverlays()` to dismiss panels
- Uses `createTextNode()` for node creation
- Uses `runEffect()` and `runEffectsSequential()` for deterministic actions
- Uses `expectSelectedCount()` for assertions
- Uses `trapPageErrors()` to catch runtime errors
