# Contract: canvas: Box select with drag marquee

bead_id: bd-rpv
bead_title: canvas: Box select with drag marquee
phase: p0
updated_at: 2026-02-28T19:40:00Z

## EARS Requirements

### Ubiquitous
- THE SYSTEM SHALL display a selection rectangle while dragging on canvas
- THE SYSTEM SHALL select all nodes fully contained within the marquee

### Event-Driven
- WHEN user drags on empty canvas, THE SYSTEM SHALL start marquee selection mode and draw rectangle
- WHEN user releases mouse after marquee drag, THE SYSTEM SHALL select all nodes within rectangle bounds

### Unwanted
- IF marquee contains no nodes, THE SYSTEM SHALL NOT clear existing selection, because: Accidental empty marquee should preserve work

## Preconditions
- auth_required: false
- required_inputs: []
- system_state:
  - Canvas has marquee interaction mode defined
  - Selection state supports multiple nodes

## Postconditions
- state_changes:
  - Marquee rectangle is cleared after selection
  - All nodes within bounds are selected

## Invariants
- Marquee coordinates are in canvas space
- Partial overlap nodes are not selected

## Implementation Status: COMPLETE
1. RubberBand mode exists - ✅ InteractionMode::RubberBand
2. Visual rectangle overlay - ✅ rubber_band_overlay function
3. Node filtering by bounds - ✅ node_ids_in_rect_with_mode
4. Selection on release - ✅ apply_rubber_band_release

## Contract Compliance
- Display selection rectangle while dragging: ✅
- Select nodes fully within marquee: ✅
- Start marquee on drag on empty canvas: ✅
- Select nodes on release: ✅
- Preserve selection on empty marquee: ✅
