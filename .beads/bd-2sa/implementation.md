bead_id: bd-2sa
bead_title: playwright: Add keyboard shortcut E2E tests
phase: p1
updated_at: 2026-03-01T00:00:00Z

# Implementation: bd-2sa - Keyboard Shortcut E2E Tests

## Summary

The keyboard shortcut E2E tests already existed in `diagram_tool/e2e/diagram.keyboard-shortcuts.spec.ts` but were missing the `@baseline` tags required by the Playwright configuration to be included in the baseline test project.

## Changes Made

### File: `/home/lewis/src/seshat/diagram_tool/e2e/diagram.keyboard-shortcuts.spec.ts`

Added `@baseline` tags to:
1. The `test.describe()` block: `"keyboard shortcuts @baseline"`
2. All 10 individual test names

## Tests Coverage

The tests cover all 4 required keyboard shortcuts per the contract:

| Shortcut | Test Coverage |
|----------|---------------|
| Ctrl+Z (Undo) | `Ctrl+Z undoes node creation @baseline`, `full undo-redo keyboard workflow @baseline` |
| Ctrl+Y (Redo) | `Ctrl+Y redoes undone action @baseline`, `full undo-redo keyboard workflow @baseline` |
| Ctrl+C (Copy) | `Ctrl+C copies selected nodes @baseline` |
| Ctrl+V (Paste) | `Ctrl+V pastes copied nodes @baseline`, `multiple paste operations stack correctly @baseline` |

Additional tests:
- `Ctrl+Shift+Z also triggers redo @baseline` - Alternative redo shortcut
- `shortcuts do not fire when input has focus @baseline` - Input field edge case
- `shortcuts blocked when textarea has focus @baseline` - Textarea edge case
- `undo after paste removes pasted nodes @baseline` - Complex workflow test
- `multiple paste operations stack correctly @baseline` - Stacking behavior test
- `full undo-redo keyboard workflow @baseline` - Complete workflow test

## Technical Implementation

- Uses `page.keyboard.press("ControlOrMeta+key")` for cross-platform keyboard shortcuts (works on both Windows/Linux with Ctrl and Mac with Meta)
- Tests verify state changes (node counts, selection counts) rather than just checking for no errors
- Uses helper functions from `helpers.ts`: `freshStart`, `clearCanvasOverlays`, `createTextNode`, `expectNodeCount`, `expectSelectedCount`, `runEffect`, `runEffectsSequential`, `trapPageErrors`
- All tests follow the existing patterns in the codebase

## Contract Compliance

### Preconditions Met
- Playwright is configured: Yes (`playwright.config.ts`)
- Test helpers exist in e2e/helpers.ts: Yes

### Postconditions Met
- New spec file `diagram.keyboard-shortcuts.spec.ts` exists: Yes (already existed)
- Tests cover all 4 shortcuts: Yes
- All tests pass: Cannot verify due to WASM build environment issue (missing sqlite3 library)

### Invariants Met
- Tests use `page.keyboard.press(ControlOrMeta+key)`: Yes
- Tests verify state changes, not just no errors: Yes
