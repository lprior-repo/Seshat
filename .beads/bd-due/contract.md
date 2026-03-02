bead_id: bd-due
bead_title: tests: Implement HIS undo/redo tests 2/2
phase: p0
updated_at: 2026-03-01T02:27:00Z

# Contract: bd-due - HIS Undo/Redo Tests 2/2

## Preconditions

- Rust unit test framework is available
- `diagram_tool/src/history.rs` contains existing HIS tests (HIS-001 through HIS-013)
- `History` struct implements undo/redo with persistent data structures

## Requirements

### EARS Requirements

**Ubiquitous:**
- THE SYSTEM SHALL have comprehensive unit tests for undo/redo history behavior

**Event-Driven:**
1. WHEN multiple undos are performed, THE SYSTEM SHALL preserve the redo chain for all undone states
2. WHEN a new action is pushed after undo, THE SYSTEM SHALL clear the redo stack completely
3. WHEN undo is performed across an autosave boundary, THE SYSTEM SHALL restore document state correctly regardless of autosave timing
4. WHEN undo/redo is performed, THE SYSTEM SHALL validate that inverse operations restore properties correctly

**Unwanted Behavior:**
- IF the redo chain is corrupted after multiple undos, THE SYSTEM SHALL NOT allow redo to proceed with wrong state
- IF a new action does not clear redo stack, THE SYSTEM SHALL NOT allow stale redo entries

## Postconditions

### State Changes
- 5 new tests added to `diagram_tool/src/history.rs` (HIS-014 through HIS-018):
  1. `HIS-014`: Redo chain preserved after multiple undos - verify redo stack integrity
  2. `HIS-015`: New action clears redo stack - verify push clears all redo entries
  3. `HIS-016`: Undo across autosave boundary - verify revision-based state restoration
  4. `HIS-017`: Inverse property validation for move operations
  5. `HIS-018`: Inverse property validation for resize operations

### Test Requirements
- All tests use the existing `make_node_for_his` helper
- All tests follow the existing naming convention `given_X_when_Y_then_Z`
- All tests are in the `tests` module of `history.rs`

## Invariants

- Tests verify state restoration, not just operation success
- Tests use deterministic document states with revision tracking
- Tests do not use `unwrap()` or `expect()` in test bodies (use pattern matching)

## Implementation Tasks

### Phase 0: Research
- Read existing HIS-001 through HIS-013 tests in history.rs
- Understand History struct API (push, undo, redo)

### Phase 1: Tests First
- Add HIS-014: Redo chain preserved after multiple undos
- Add HIS-015: New action clears redo stack
- Add HIS-016: Undo across autosave boundary
- Add HIS-017: Inverse property validation (move)
- Add HIS-018: Inverse property validation (resize)

### Phase 2: Verification
- Run `cargo test -p diagram_tool history::tests::his` to verify new tests
- Run `moon run :ci` for full validation

## Acceptance Criteria

- [ ] All 5 new HIS tests written and passing
- [ ] Tests follow existing naming conventions
- [ ] No use of unwrap/expect in test bodies
- [ ] `moon run :ci` passes
- [ ] Tests are in the existing `tests` module of history.rs

## Technical Constraints

- Use existing `make_node_for_his` helper function
- Use `DiagramDocument` with revision tracking
- Tests must be deterministic and not depend on external state
- Follow the existing test documentation style with `/// HIS-NNN:` comments

## Test Specifications

### HIS-014: Redo chain preserved after multiple undos
```rust
// Given: History with 4 states (A, B, C, D)
// When: Undo 3 times (back to A)
// Then: Redo stack has 3 entries (B, C, D) in correct order
// And: Each redo restores the correct state
```

### HIS-015: New action clears redo stack
```rust
// Given: History with redo entries after undo
// When: A new document state is pushed
// Then: Redo stack is completely empty
// And: Only the new push is in undo stack
```

### HIS-016: Undo across autosave boundary
```rust
// Given: Document with multiple revisions (simulating autosave intervals)
// When: Undo is performed
// Then: Document state is restored to previous revision
// And: Revision number matches the restored state
```

### HIS-017: Inverse property validation (move)
```rust
// Given: Node at position (x1, y1)
// When: Move to (x2, y2), push, then undo
// Then: Node position is exactly (x1, y1) (inverse of move)
```

### HIS-018: Inverse property validation (resize)
```rust
// Given: Node with dimensions (w1, h1)
// When: Resize to (w2, h2), push, then undo
// Then: Node dimensions are exactly (w1, h1) (inverse of resize)
```
