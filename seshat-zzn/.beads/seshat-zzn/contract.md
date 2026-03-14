# Contract Specification: seshat-zzn - Unit Tests for SEL-005..SEL-009

## Metadata
- **Bead ID**: seshat-zzn
- **Title**: Write unit tests for SEL-005..SEL-009
- **Priority**: P2
- **Type**: task
- **Created**: 2026-03-13
- **Parent Feature**: Selection Reliability (bd-2qs)

## Context

This contract specifies unit tests for test cases SEL-005 through SEL-009, which cover:
- Selection visual feedback and interaction affordances
- Touch input handling
- Drag threshold prevention

### Domain Terms
- **Marquee**: A rectangular selection area drawn by the user
- **Hit Area**: The region where a touch/click registers on an element
- **Drag Threshold**: Minimum movement distance before a drag operation is recognized
- **Visual Affordance**: Visual indication that an element is interactive (hover state)
- **TOUCH_HIT_RADIUS_PX**: 44.0 - WCAG-compliant minimum touch target size (44x44 pixels)
- **TOUCH_HIT_RADIUS_MIN**: 22.0 - minimum radius for handle hit areas (corresponds to 44px diameter)

### Assumptions
- Document model provides `DiagramDocument` with nodes and editor state
- Selection state is managed in `EditorState.selected_items` (HashSet<String>)
- Hover state is tracked via `hovered_node` signal in UI layer
- Touch input detection is available via `is_touch` boolean flag
- InteractionReducer handles the state machine for hover/drag transitions

### Open Questions (RESOLVED)
- [Q1] **RESOLVED**: Unit tests test model logic only; UI rendering verification is out of scope (requires integration/E2E tests with actual DOM)
- [Q2] **RESOLVED**: Tests verify behavior only; performance timing (<16ms) is verified via integration tests or manual testing
- [Q3] **RESOLVED**: Touch hit area tests verify exact pixel values (44.0) to match WCAG compliance

---

## Preconditions

### SEL-005: Marquee Direction (Already implemented in seshat-9n1)
- [P1] `marquee_select` called with a valid Rect
- [P2] Document contains at least one node

### SEL-006: Hover Shows Visual Affordances
- [P3] A node exists in the document
- [P4] Hover position is within node bounds
- [P5] No drag operation is in progress

### SEL-007: Resize Handles Are Clickable
- [P6] At least one node is selected
- [P7] Selection bounds are computed successfully
- [P8] Handle position is within the selection bounding box

### SEL-008: Touch Has Larger Hit Area
- [P9] Input device is touch (is_touch = true)
- [P10] Touch point is within base_hit_radius + TOUCH_HIT_RADIUS_PX distance

### SEL-009: Drag Threshold Prevents Accidental Drag
- [P11] A node is selected (anchor point exists)
- [P12] Current position is tracked after pointerdown
- [P13] Threshold value is 3.0 pixels

---

## Postconditions

### SEL-005: Marquee Direction
- [Q1] L→R marquee (positive width): only fully contained nodes selected
- [R→L marquee (negative/zero width): any intersecting nodes selected

### SEL-006: Hover Shows Visual Affordances
- [Q2] Hovered node ID is stored in hover state
- [Q3] Hover state changes from None to Some(node_id)
- [Q4] Hover transition completes (state machine moves to Hovering)

### SEL-007: Resize Handles Are Clickable
- [Q5] 8 handles exist for any non-empty selection (NW, N, NE, E, SE, S, SW, W)
- [Q6] Each handle initiates resize when dragged
- [Q7] Handle hit test returns the correct handle for a given point

### SEL-008: Touch Has Larger Hit Area
- [Q8] Touch hit radius = max(base_radius, 44.0) for nodes (WCAG 44x44 touch target)
- [Q9] Touch hit radius = max(base_radius, 44.0) for handles (44.0 > 14.0 handle size)
- [Q10] Mouse hit radius = base_radius (no extension)

### SEL-009: Drag Threshold Prevents Accidental Drag
- [Q11] Movement < 3px returns false from `has_drag_threshold`
- [Q12] Movement >= 3px returns true from `has_drag_threshold`
- [Q13] Euclidean distance used for diagonal movement

---

## Invariants

1. **I1**: Selection handles exist only when `selected_items` is non-empty
2. **I2**: Hover state is cleared when pointer leaves all nodes
3. **I3**: Touch hit area is always >= mouse hit area for the same element
4. **I4**: Drag threshold check uses Euclidean distance, not Manhattan
5. **I5**: Hover feedback is delivered within 16ms of mouseenter

---

## Error Taxonomy

Based on existing `SelectionError` enum in selection.rs:

| Error Variant | Condition | Test Coverage |
|---|---|---|
| `SelectionError::NodeNotFound` | Node ID doesn't exist in document | SEL-006, SEL-007 |
| `SelectionError::MovementExceededDragThreshold` | Drag moved past threshold | SEL-009 |
| `SelectionError::PreconditionViolated` | Contract precondition not met | All tests |
| `SelectionError::InvalidMarqueeBounds` | Negative width/height in marquee | SEL-005 |

---

## Contract Signatures

Based on existing functions in the codebase:

```rust
// From canvas_view.rs - TOUCH HIT RADIUS CONSTANTS
const TOUCH_HIT_RADIUS_PX: f64 = 44.0;     // WCAG-compliant touch target (44x44px)
const TOUCH_HIT_RADIUS_MIN: f64 = 22.0;    // Minimum radius for handles (44px diameter)

// From selection.rs
fn compute_selection_bounds(doc: &DiagramDocument) -> Result<SelectionBounds, SelectionError>
fn compute_marquee_selection(doc: &DiagramDocument, marquee: Rect, mode: MarqueeMode) -> Result<HashSet<ElementId>, SelectionError>

// From interaction.rs  
fn has_drag_threshold(origin: (f64, f64), current: (f64, f64)) -> bool

// From canvas_view.rs
fn touch_hit_radius(base_radius: f64, is_touch: bool) -> f64
fn touch_handle_hit_test(touch_x: f64, touch_y: f64, handle_x: f64, handle_y: f64, is_touch: bool) -> bool
```

---

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Node exists in document | Runtime-checked | `doc.nodes.get(id).is_some()` |
| Selection is non-empty | Runtime-checked | `!selected_items.is_empty()` |
| Drag threshold >= 0 | Compile-time | `const DRAG_THRESHOLD: f64 = 3.0` |
| Touch hit radius >= 44.0 | Compile-time constant | `const TOUCH_HIT_RADIUS_PX: f64 = 44.0` |
| Hover position valid | Runtime state | `InteractionState::Hovering` |

---

## Violation Examples

### SEL-006: Hover
- **VIOLATES P3**: `hover_node(doc, NodeId("nonexistent"))` -- should produce `Err(SelectionError::NodeNotFound)`

### SEL-007: Resize Handles
- **VIOLATES P6**: `get_handles(SelectionBounds { x: 0, y: 0, width: 0, height: 0 })` -- should produce 0 handles (empty selection case handled by invariant I1)
- **VIOLATES P7**: `hit_test_handle(Point(-1000, -1000), handles)` -- should return None (point outside all handles)

### SEL-008: Touch Hit Area (FIXED - using 44.0)
- **VIOLATES Q8**: `touch_hit_radius(10.0, false)` -- should return 10.0 (mouse, no extension)
- **VIOLATES Q9**: `touch_hit_radius(10.0, true)` -- should return max(10.0, 44.0) = 44.0 for nodes; handles use max(14.0, 44.0) = 44.0

### SEL-009: Drag Threshold
- **VIOLATES Q11**: `has_drag_threshold((0.0, 0.0), (2.9, 0.0))` -- should return false (under threshold)
- **VIOLATES Q12**: `has_drag_threshold((0.0, 0.0), (3.0, 0.0))` -- should return true (at threshold)
- **VIOLATES Q13**: `has_drag_threshold((0.0, 0.0), (2.0, 2.0))` -- should return false (sqrt(8) ≈ 2.83 < 3.0)

---

## Ownership Contracts

This contract does not modify any existing functions. Tests are written to verify behavior:

- **No ownership transfer**: All test functions borrow data (`&DiagramDocument`, `&Rect`)
- **No mutation**: Tests verify state, they do not mutate production state
- **Clone policy**: Test fixtures create owned values for setup, assertions use references

---

## Non-goals

- [ ] Implementing UI rendering verification (SEL-006 visual feedback) - requires E2E tests
- [ ] Testing performance timing (SEL-006 <16ms requirement) - verified via integration tests
- [ ] Integration tests with actual DOM events - out of scope for unit tests
- [ ] Testing selection across document boundaries (SEL-025)

---

## Testing Trophy Considerations

This test plan focuses on **unit tests** for core logic. The Testing Trophy hierarchy is addressed as follows:

| Level | Coverage | Location |
|-------|----------|----------|
| Unit Tests | Core logic functions (touch_hit_radius, has_drag_threshold, handle hit testing) | This bead (seshat-zzn) |
| Integration Tests | Component interactions (hover state machine, selection computation) | Separate beads |
| E2E Tests | Full user workflows (touch selection in browser) | Separate E2E bead |

**Rationale**: Unit tests verify the mathematical correctness of hit radius calculations and drag threshold logic. Integration/E2E tests verify these functions work correctly in the full application context.

(End of file - total 210 lines)
