# Contract Specification: Keyboard Shortcut E2E Tests

**Bead ID:** bd-2sa
**Title:** playwright: Add keyboard shortcut E2E tests
**Generated:** 2026-02-26

## Scope

Test keyboard shortcuts for diagram editor operations via Playwright E2E tests.

## Shortcuts Under Test

| Shortcut | Action | Primary Key | Alternate Key |
|----------|--------|-------------|---------------|
| Ctrl+Z | Undo | `ControlOrMeta+z` | - |
| Ctrl+Y | Redo | `ControlOrMeta+y` | `ControlOrMeta+Shift+z` |
| Ctrl+C | Copy | `ControlOrMeta+c` | - |
| Ctrl+V | Paste | `ControlOrMeta+v` | - |

## Behavioral Contracts

### Contract 1: Undo Shortcut (Ctrl+Z)

**Precondition:** Canvas has at least one node
**Action:** Press `ControlOrMeta+z`
**Postcondition:** Last operation is undone (node count decreases by 1 for node creation)

**State Changes:**
- `nodeCount` decreases by 1 (for node creation undo)
- Redo stack becomes available

### Contract 2: Redo Shortcut (Ctrl+Y)

**Precondition:** Undo has been performed, redo stack non-empty
**Action:** Press `ControlOrMeta+y`
**Postcondition:** Last undone operation is reapplied

**State Changes:**
- `nodeCount` increases by 1 (for node creation redo)
- Undo stack updated

### Contract 3: Redo Alternate Shortcut (Ctrl+Shift+Z)

**Precondition:** Undo has been performed, redo stack non-empty
**Action:** Press `ControlOrMeta+Shift+z`
**Postcondition:** Same as Contract 2 - last undone operation is reapplied

**Equivalence:** Must behave identically to Ctrl+Y

### Contract 4: Copy Shortcut (Ctrl+C)

**Precondition:** At least one node is selected
**Action:** Press `ControlOrMeta+c`
**Postcondition:** Selected nodes copied to clipboard (no visible state change)

**State Changes:**
- `nodeCount` unchanged
- `selectedCount` unchanged
- Internal clipboard state updated (not directly observable)

### Contract 5: Paste Shortcut (Ctrl+V)

**Precondition:** Clipboard contains copied nodes
**Action:** Press `ControlOrMeta+v`
**Postcondition:** Nodes duplicated from clipboard to canvas

**State Changes:**
- `nodeCount` increases by count of clipboard items
- New nodes appear on canvas

### Contract 6: Focus Guard - Input Elements

**Precondition:** An `<input>` element has focus
**Action:** Press any shortcut key (Ctrl+Z/Y/C/V)
**Postcondition:** Shortcut does NOT trigger diagram action

**State Changes:**
- `nodeCount` unchanged
- `selectedCount` unchanged
- Text input proceeds normally

### Contract 7: Focus Guard - Textarea Elements

**Precondition:** A `<textarea>` element has focus
**Action:** Press any shortcut key (Ctrl+Z/Y/C/V)
**Postcondition:** Shortcut does NOT trigger diagram action

**State Changes:**
- `nodeCount` unchanged
- `selectedCount` unchanged
- Text editing proceeds normally

## Error Handling Contract

**All tests must:**
- Use `trapPageErrors(page)` to capture console errors
- Assert `pageErrors` array is empty at test conclusion
- Fail on any uncaught page errors or console.error messages

## Test File Structure

```
diagram_tool/e2e/
├── diagram.keyboard-shortcuts.spec.ts  (NEW)
├── helpers.ts                          (existing)
├── diagram.undo-redo-history.spec.ts   (existing)
└── diagram.history-clipboard.spec.ts   (existing)
```

## Implementation Constraints

1. Use `page.keyboard.press('ControlOrMeta+Key')` for cross-platform compatibility
2. Use `runEffectsSequential` for setup sequences
3. Use `runEffect` for single async operations
4. Verify state with `expectNodeCount` and `expectSelectedCount` helpers
5. Follow existing test file import and structure patterns

## Acceptance Criteria

- [ ] All 4 shortcuts tested independently
- [ ] Ctrl+Shift+Z tested as redo alternate
- [ ] Focus guard tests for input and textarea
- [ ] State changes verified via counter helpers
- [ ] No console errors during test execution
