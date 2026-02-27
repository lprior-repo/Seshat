# Martin Fowler Given-When-Then Test Scenarios

**Bead ID:** bd-2sa
**Title:** playwright: Add keyboard shortcut E2E tests
**Generated:** 2026-02-26

---

## Scenario 1: Undo via Ctrl+Z removes last created node

**Given** the canvas is empty
**And** I create 2 text nodes at positions (560, 220) and (790, 320)
**And** the node count is 2

**When** I press `ControlOrMeta+z`

**Then** the node count is 1
**And** the canvas has 1 node element

---

## Scenario 2: Redo via Ctrl+Y restores undone node

**Given** the canvas has 1 node (after undo of 2-node creation)
**And** the redo stack has 1 entry

**When** I press `ControlOrMeta+y`

**Then** the node count is 2
**And** the canvas has 2 node elements

---

## Scenario 3: Redo via Ctrl+Shift+Z restores undone node

**Given** the canvas has 1 node (after undo of 2-node creation)
**And** the redo stack has 1 entry

**When** I press `ControlOrMeta+Shift+z`

**Then** the node count is 2
**And** the canvas has 2 node elements

---

## Scenario 4: Copy via Ctrl+C does not change node count

**Given** the canvas has 2 text nodes
**And** I select both nodes (selected count is 2)

**When** I press `ControlOrMeta+c`

**Then** the node count is still 2
**And** the selected count is still 2

---

## Scenario 5: Paste via Ctrl+V duplicates copied nodes

**Given** the canvas has 2 text nodes
**And** both nodes are selected
**And** I have pressed `ControlOrMeta+c` (nodes copied)

**When** I press `ControlOrMeta+v`

**Then** the node count is 4
**And** the canvas has 4 node elements

---

## Scenario 6: Multiple paste operations stack correctly

**Given** the canvas has 2 text nodes
**And** both nodes are copied to clipboard

**When** I press `ControlOrMeta+v` once
**Then** the node count is 4

**When** I press `ControlOrMeta+v` again
**Then** the node count is 6

---

## Scenario 7: Undo after paste removes pasted nodes

**Given** the canvas has 4 nodes (2 original + 2 pasted)

**When** I press `ControlOrMeta+z`

**Then** the node count is 2

---

## Scenario 8: Shortcuts blocked when input has focus

**Given** the canvas has 2 text nodes
**And** I click on a text node to edit it
**And** an `<input>` element has focus

**When** I press `ControlOrMeta+z`

**Then** the node count is still 2 (undo did NOT trigger)
**And** the input handles the keystroke natively

---

## Scenario 9: Shortcuts blocked when textarea has focus

**Given** the canvas has 2 text nodes
**And** a `<textarea>` element has focus

**When** I press `ControlOrMeta+y`

**Then** the node count is still 2 (redo did NOT trigger)
**And** the textarea handles the keystroke natively

---

## Scenario 10: Copy shortcut blocked when input has focus

**Given** the canvas has 2 text nodes with one selected
**And** an `<input>` element has focus

**When** I press `ControlOrMeta+c`

**Then** the selected count is still 1 (copy did NOT trigger)

---

## Scenario 11: Paste shortcut blocked when textarea has focus

**Given** the clipboard contains 2 nodes
**And** the canvas has 2 text nodes
**And** a `<textarea>` element has focus

**When** I press `ControlOrMeta+v`

**Then** the node count is still 2 (paste did NOT trigger)

---

## Scenario 12: Full undo-redo keyboard workflow

**Given** the canvas is empty

**When** I create 1 text node at (500, 200)
**Then** the node count is 1

**When** I press `ControlOrMeta+z`
**Then** the node count is 0

**When** I press `ControlOrMeta+Shift+z` (alternate redo)
**Then** the node count is 1

**When** I press `ControlOrMeta+z` again
**Then** the node count is 0

**When** I press `ControlOrMeta+y`
**Then** the node count is 1

---

## Scenario 13: No console errors during keyboard operations

**Given** the error trap is active via `trapPageErrors(page)`

**When** I perform any keyboard shortcut test

**Then** the `pageErrors` array has length 0
**And** no console.error messages were logged
**And** no page errors were thrown

---

## Test Implementation Mapping

| Scenario | Test Name | Primary Assertion |
|----------|-----------|-------------------|
| 1 | `undo via Ctrl+Z removes last created node` | `expectNodeCount(page, 1)` |
| 2 | `redo via Ctrl+Y restores undone node` | `expectNodeCount(page, 2)` |
| 3 | `redo via Ctrl+Shift+Z restores undone node` | `expectNodeCount(page, 2)` |
| 4 | `copy via Ctrl+C does not change node count` | `expectNodeCount(page, 2)` |
| 5 | `paste via Ctrl+V duplicates copied nodes` | `expectNodeCount(page, 4)` |
| 6 | `multiple paste operations stack correctly` | `expectNodeCount(page, 6)` |
| 7 | `undo after paste removes pasted nodes` | `expectNodeCount(page, 2)` |
| 8 | `shortcuts blocked when input has focus` | `expectNodeCount(page, 2)` |
| 9 | `shortcuts blocked when textarea has focus` | `expectNodeCount(page, 2)` |
| 10 | `copy shortcut blocked when input has focus` | `expectSelectedCount(page, 1)` |
| 11 | `paste shortcut blocked when textarea has focus` | `expectNodeCount(page, 2)` |
| 12 | `full undo-redo keyboard workflow` | Multi-step assertions |
| 13 | `no console errors during keyboard operations` | `expect(pageErrors).toHaveLength(0)` |

---

## Helper Functions Required

From `helpers.ts`:
- `trapPageErrors(page)` - Error capture
- `runEffectsSequential(steps)` - Sequential async setup
- `runEffect(thunk)` - Single async operation
- `waitForUiReady(page)` - UI readiness check
- `clearCanvasOverlays(page)` - Clear modals
- `canvas(page)` - Canvas locator
- `createTextNode(page, canvas, x, y)` - Node creation
- `expectNodeCount(page, count)` - Node count assertion
- `expectSelectedCount(page, count)` - Selection count assertion
