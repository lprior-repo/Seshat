# Contract: grid: Visual dot grid overlay on canvas

bead_id: bd-2xt
bead_title: grid: Visual dot grid overlay on canvas
phase: p0
updated_at: 2026-02-28T19:13:04Z

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL display a dot grid background on the canvas
- THE SYSTEM SHALL scale grid dots with viewport zoom

### Event-Driven
- WHEN viewport zoom changes, THE SYSTEM SHALL update grid background size to maintain visual consistency

### Unwanted
- IF grid overlay affects performance negatively, THE SYSTEM SHALL NOT render grid with excessive DOM elements, because: Performance must remain smooth during pan and zoom

## Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Canvas container has defined dimensions
  - Viewport has valid zoom and pan values

## Postconditions
- state_changes:
  - Grid dots are evenly spaced visually
  - Grid pans and zooms with content

## Invariants
- Grid dots remain crisp at all zoom levels
- Grid does not interfere with node interaction

## Research Requirements
- Read busted-flow/components/flow/flow-canvas.tsx:230-235 for existing patterns
- Decide between CSS background vs SVG overlay approach

## Implementation Tasks
1. Review busted-flow dot grid implementation
2. Decide between CSS background vs SVG overlay approach
3. Write visual test for grid rendering
4. Write test for grid scaling behavior
5. Add GridOverlay component using CSS radial-gradient
6. Wire GridOverlay to viewport zoom and pan signals
7. Add grid visibility toggle to toolbar

## Acceptance Tests
- test_happy_path: Valid inputs, User executes command, Exit code is 0, Output is correct
- test_error_path: Invalid inputs, User executes command, Exit code is non-zero, Error message is clear
