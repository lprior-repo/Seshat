bead_id: bd-1db
bead_title: playwright: Add button disabled state tests
phase: p0
updated_at: 2026-03-02T01:12:00Z

# Contract: Button Disabled State Tests

## Overview

Add Playwright E2E tests to verify that toolbar buttons (Undo, Redo, Copy, Paste) correctly reflect their disabled/enabled states based on application state.

## Preconditions

- Playwright is configured
- Buttons have disabled attributes
- Test infrastructure includes helpers for fresh page setup

## Postconditions

### State Changes
- Tests verify Undo disabled state
- Tests verify Redo disabled state
- Tests verify Copy disabled state
- Tests verify Paste disabled state

### Invariants
- Tests check both disabled attribute and visual state

## Event-Driven Requirements

| Trigger | Expected Behavior |
|---------|-------------------|
| WHEN undo history is empty | THE SYSTEM SHALL disable Undo button |
| WHEN redo history is empty | THE SYSTEM SHALL disable Redo button |
| WHEN no selection exists | THE SYSTEM SHALL disable Copy button |
| WHEN clipboard is empty | THE SYSTEM SHALL disable Paste button |

## Unwanted Behaviors

| Condition | Unwanted Behavior | Rationale |
|-----------|-------------------|-----------|
| IF action is not available | THE SYSTEM SHALL NOT show enabled button | Users should not be able to click unavailable actions |

## Test Coverage Requirements

1. **Undo Button Tests**
   - Disabled on fresh document
   - Enabled after edit

2. **Redo Button Tests**
   - Disabled on fresh document
   - Enabled after undo
   - Disabled after all redos exhausted

3. **Copy Button Tests**
   - Disabled with no selection
   - Enabled with selection
   - Disabled after selection cleared

4. **Paste Button Tests**
   - Disabled with empty clipboard
   - Enabled after copy

5. **Combined State Tests**
   - All buttons disabled initially
   - State transitions after edit cycle

## Verification

- All tests tagged with @baseline
- Tests use freshStart() for consistent state
- Page errors trapped and verified to be empty
