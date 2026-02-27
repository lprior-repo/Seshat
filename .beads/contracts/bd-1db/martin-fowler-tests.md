# Martin Fowler Given-When-Then Tests: Button Disabled States

**Bead ID:** bd-1db
**Test File:** `diagram_tool/e2e/diagram.button-disabled-states.spec.ts`

---

## Test 1: Undo Disabled on Fresh Document

```
GIVEN a fresh page load
  AND the UI is ready
  AND the canvas is clear
WHEN I locate the Undo button by testid "toolbar-undo"
THEN the Undo button is disabled
```

---

## Test 2: Undo Enabled After Edit

```
GIVEN a fresh page load
  AND the UI is ready
  AND the canvas is clear
WHEN I create a text node at position (560, 220)
  AND I locate the Undo button by testid "toolbar-undo"
THEN the Undo button is enabled
```

---

## Test 3: Redo Disabled on Fresh Document

```
GIVEN a fresh page load
  AND the UI is ready
  AND the canvas is clear
WHEN I locate the Redo button by testid "toolbar-redo"
THEN the Redo button is disabled
```

---

## Test 4: Redo Disabled After All Redos Exhausted

```
GIVEN a fresh page load
  AND I have created a text node
  AND I have clicked Undo
  AND the Redo button is enabled
WHEN I click the Redo button
  AND the redo stack is now empty
THEN the Redo button is disabled
```

---

## Test 5: Redo Enabled After Undo

```
GIVEN a fresh page load
  AND I have created a text node
  AND the Undo button is enabled
WHEN I click the Undo button
  AND I locate the Redo button by testid "toolbar-redo"
THEN the Redo button is enabled
```

---

## Test 6: Copy Disabled With No Selection

```
GIVEN a fresh page load
  AND the UI is ready
  AND the canvas is clear
WHEN I locate the Copy button by testid "toolbar-copy"
THEN the Copy button is disabled
```

---

## Test 7: Copy Enabled With Selection

```
GIVEN a fresh page load
  AND I have created a text node at position (560, 220)
WHEN I click on the node to select it
  AND I verify selection count is 1
  AND I locate the Copy button by testid "toolbar-copy"
THEN the Copy button is enabled
```

---

## Test 8: Copy Disabled After Selection Cleared

```
GIVEN a fresh page load
  AND I have created a text node
  AND I have selected the node
  AND the Copy button is enabled
WHEN I click on empty canvas to deselect
  AND I verify selection count is 0
THEN the Copy button is disabled
```

---

## Test 9: Paste Disabled With Empty Clipboard

```
GIVEN a fresh page load
  AND the UI is ready
  AND the canvas is clear
WHEN I locate the Paste button by testid "toolbar-paste"
THEN the Paste button is disabled
```

---

## Test 10: Paste Enabled After Copy

```
GIVEN a fresh page load
  AND I have created a text node at position (560, 220)
  AND I have selected the node
WHEN I trigger copy action (Ctrl/Cmd+C or toolbar-copy click)
  AND I locate the Paste button by testid "toolbar-paste"
THEN the Paste button is enabled
```

---

## Test 11: All Buttons Disabled Initially

```
GIVEN a fresh page load
  AND the UI is ready
  AND the canvas is clear
WHEN I locate all four toolbar buttons
  - toolbar-undo
  - toolbar-redo
  - toolbar-copy
  - toolbar-paste
THEN all four buttons are disabled
```

---

## Test 12: State Transitions After Edit Cycle

```
GIVEN a fresh page load
  AND I have created a text node
  AND I have selected the node
WHEN I copy the selection
THEN the Copy button is enabled (selection still exists)
  AND the Paste button is enabled (clipboard has content)
  AND the Undo button is enabled (history exists)
  AND the Redo button is disabled (no redo history)
```

---

## Implementation Notes

### Locators

```typescript
const undoButton = page.getByTestId("toolbar-undo");
const redoButton = page.getByTestId("toolbar-redo");
const copyButton = page.getByTestId("toolbar-copy");
const pasteButton = page.getByTestId("toolbar-paste");
```

### Assertion Patterns

```typescript
await expect(button).toBeDisabled();
await expect(button).toBeEnabled();
```

### Selection Helper

```typescript
async function selectNode(page: Page): Promise<void> {
  const node = canvas(page).getByTestId("node").first();
  await runEffect(() => node.click());
  await expectSelectedCount(page, 1);
}
```

### Copy Helper

```typescript
async function performCopy(page: Page): Promise<void> {
  await runEffect(() => page.keyboard.press("ControlOrMeta+c"));
}
```
