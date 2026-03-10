# Contract Specification

## Context
- **Feature**: Multi-touch drag selection while another pointer is down (MUL-009)
- **Domain terms**: 
  - `pointerId`: Unique identifier for each pointer event
  - `active_pointers`: Set of currently active pointer IDs
  - `captured_pointer`: Pointer ID that initiated a drag operation
  - `multi_touch_active`: Flag indicating >= 2 pointers are down
- **Assumptions**:
  - The canvas uses pointer events (not touch-specific events)
  - Each pointer has a unique `pointerId` from the browser
  - The system must track which pointer initiated a drag
- **Open questions**:
  - Should new pointers be allowed to "steal" capture or should they be ignored?
  - What happens when the captured pointer releases while another is down?

## Preconditions
- [P1] Pointer down event MUST include a valid `pointerId` 
- [P2] Active pointer set MUST be initialized empty on canvas mount
- [P3] Only ONE pointer can be in captured/dragging state at a time

## Postconditions
- [Q1] After pointer down: active_pointers contains the new pointerId
- [Q2] After pointer up: active_pointers no longer contains that pointerId
- [Q3] If pointer down while captured_pointer is set: the new pointer is IGNORED (not added to active_pointers for drag purposes)
- [Q4] When captured_pointer releases: interaction mode resets to Select

## Invariants
- [I1] At any time, `captured_pointer` is either None or contains exactly one valid pointerId
- [I2] `active_pointers` count matches actual browser pointer downs
- [I3] Drag operations only proceed for the captured pointer

## Error Taxonomy
- N/A - This is UI event handling, not a fallible operation

## Contract Signatures

```rust
// Canvas-level signal/state management
fn on_pointer_down(pointer_id: u32, position: (f64, f64)) -> Effect {
    // If no captured_pointer: set captured_pointer = pointer_id, add to active_pointers
    // If captured_pointer exists: IGNORE (don't add to active_pointers, don't change mode)
}

fn on_pointer_move(pointer_id: u32, position: (f64, f64)) -> Effect {
    // Only process if pointer_id == captured_pointer
    // Ignore moves from other pointers
}

fn on_pointer_up(pointer_id: u32) -> Effect {
    // If pointer_id == captured_pointer: clear captured_pointer, reset mode to Select
    // Always remove from active_pointers
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| pointerId valid | Runtime (browser provides) | `u32` from JS event |
| Single captured pointer | Runtime state | `Signal<Option<u32>>` |
| Pointer in active set | Runtime | `Signal<HashSet<u32>>` |

## Violation Examples
- VIOLATES P3: Two simultaneous pointer downs -- second pointer gets processed for drag
  - Given: captured_pointer = Some(1), active_pointers = {1}, second pointer(2) down
  - When: on_pointer_down(2, ...)
  - Then: State corruption - two pointers now in dragging state
  
- VIOLATES Q3: New pointer overwrites captured
  - Given: captured_pointer = Some(1), pointer(2) down while (1) dragging
  - When: on_pointer_down(2, ...)
  - Then: captured_pointer becomes Some(2) -- WRONG

- VIOLATES Q2: Pointer not removed from active set
  - Given: active_pointers = {1, 2}, pointer(1) up
  - When: on_pointer_up(1)
  - Then: active_pointers = {1, 2} -- state leak

## Ownership Contracts
- N/A - UI event handlers don't own data, they mutate signals

## Non-goals
- Multi-finger gestures (pinch to zoom, two-finger pan)
- Pointer capture release due to navigation/overlay
