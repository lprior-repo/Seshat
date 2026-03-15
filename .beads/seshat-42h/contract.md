# Contract: Node Hit Testing (SEL-001, SEL-004)

## Overview
Clicking a node selects it. Clicking canvas clears selection. Only handle single node selection.

## Preconditions
- Coordinate must be valid screen space (x, y).

## Postconditions
- If a node intersects the coordinate, its ID is set as the sole item in `editor_state.selected_items`.
- If no node intersects, `editor_state.selected_items` is cleared.

## Invariants
- `selected_items` contains at most 1 item when single-click hit testing occurs.