bead_id: bd-2sa
bead_title: playwright: Add keyboard shortcut E2E tests
phase: p0
updated_at: 2026-03-01T00:00:00Z

# Contract: bd-2sa - Keyboard Shortcut E2E Tests

## Preconditions

- Playwright is configured
- Test helpers exist in e2e/helpers.ts

## Requirements

### EARS Requirements

**Ubiquitous:**
- THE SYSTEM SHALL have E2E tests for keyboard shortcuts

**Event-Driven:**
1. WHEN user presses Ctrl+Z, THE SYSTEM SHALL undo last action
2. WHEN user presses Ctrl+Y, THE SYSTEM SHALL redo last undone action
3. WHEN user presses Ctrl+C, THE SYSTEM SHALL copy selection
4. WHEN user presses Ctrl+V, THE SYSTEM SHALL paste from clipboard

**Unwanted Behavior:**
- IF focus is in input field, THE SYSTEM SHALL NOT trigger diagram shortcuts, because input fields need native shortcuts

## Postconditions

### State Changes
- New spec file `diagram.keyboard-shortcuts.spec.ts` exists
- Tests cover all 4 shortcuts (Ctrl+Z, Ctrl+Y, Ctrl+C, Ctrl+V)
- All tests pass

## Invariants

- Tests use `page.keyboard.press(ControlOrMeta+key)`
- Tests verify state changes, not just no errors

## Implementation Tasks

### Phase 0: Research
- Read existing Playwright test patterns in e2e/

### Phase 1: Tests First
- Create `diagram.keyboard-shortcuts.spec.ts`
- Add test for Ctrl+Z undo
- Add test for Ctrl+Y redo
- Add test for Ctrl+C copy
- Add test for Ctrl+V paste

### Phase 2: Implementation
- Run Playwright tests
- Fix any failures

### Phase 4: Verification
- Run `moon run :ci`

## Acceptance Criteria

- [ ] All acceptance tests written and passing
- [ ] All error path tests written and passing
- [ ] E2E pipeline test passing with real data
- [ ] No mocks or fake data in any test
- [ ] `moon run :ci` passes

## Technical Constraints

- Use `page.keyboard.press(ControlOrMeta+key)` for cross-platform keyboard shortcuts
- Tests must verify state changes (undo/redo stack, clipboard contents)
- Tests must handle input field focus edge cases
