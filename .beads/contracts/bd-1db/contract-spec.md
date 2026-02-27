# Contract Specification: Button Disabled State Tests

**Bead ID:** bd-1db
**Component:** `diagram_tool/e2e/` (Playwright tests)
**Target:** Toolbar button disabled state verification

---

## 1. Purpose

Verify that toolbar buttons correctly reflect their disabled/enabled state based on document state:
- Undo button: disabled on fresh document, enabled after edit
- Redo button: disabled when no redo history available
- Copy button: disabled with no selection, enabled with selection
- Paste button: disabled with empty clipboard, enabled after copy

---

## 2. Test Subject Contract

### 2.1 Undo Button

**Selector:** `page.getByTestId("toolbar-undo")`

**State Transitions:**
| Initial State | Trigger | Expected State |
|---------------|---------|----------------|
| Disabled | Fresh document load | Disabled |
| Disabled | Node created | Enabled |
| Enabled | Undo clicked (exhausts history) | Disabled |
| Enabled | New edit after undo | Enabled |

**Assertions:**
```typescript
expect(undoButton).toBeDisabled()  // Fresh document
expect(undoButton).toBeEnabled()   // After edit
```

---

### 2.2 Redo Button

**Selector:** `page.getByTestId("toolbar-redo")`

**State Transitions:**
| Initial State | Trigger | Expected State |
|---------------|---------|----------------|
| Disabled | Fresh document load | Disabled |
| Disabled | Node created | Disabled |
| Disabled | Undo performed | Enabled |
| Enabled | Redo clicked (exhausts stack) | Disabled |
| Enabled | New edit (invalidates redo) | Disabled |

**Assertions:**
```typescript
expect(redoButton).toBeDisabled()  // No redo history
expect(redoButton).toBeEnabled()   // After undo
```

---

### 2.3 Copy Button

**Selector:** `page.getByTestId("toolbar-copy")`

**State Transitions:**
| Initial State | Trigger | Expected State |
|---------------|---------|----------------|
| Disabled | Fresh document (no selection) | Disabled |
| Disabled | Node created (no selection) | Disabled |
| Disabled | Node clicked (selected) | Enabled |
| Enabled | Selection cleared | Disabled |

**Assertions:**
```typescript
expect(copyButton).toBeDisabled()  // No selection
expect(copyButton).toBeEnabled()   // With selection
```

---

### 2.4 Paste Button

**Selector:** `page.getByTestId("toolbar-paste")`

**State Transitions:**
| Initial State | Trigger | Expected State |
|---------------|---------|----------------|
| Disabled | Fresh document (empty clipboard) | Disabled |
| Disabled | Copy performed | Enabled |
| Enabled | Page reload | Disabled |

**Assertions:**
```typescript
expect(pasteButton).toBeDisabled()  // Empty clipboard
expect(pasteButton).toBeEnabled()   // After copy
```

---

## 3. Test Helper Requirements

### 3.1 Required Imports

```typescript
import { expect, test, type Page, type Locator } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  createTextNode,
  expectSelectedCount,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";
```

### 3.2 Button Locators

```typescript
const undoButton = page.getByTestId("toolbar-undo");
const redoButton = page.getByTestId("toolbar-redo");
const copyButton = page.getByTestId("toolbar-copy");
const pasteButton = page.getByTestId("toolbar-paste");
```

---

## 4. Test Scenarios

| Test Name | Buttons Tested | Initial State | Actions | Expected Final |
|-----------|---------------|---------------|---------|----------------|
| Undo disabled on fresh | Undo | Fresh doc | None | Disabled |
| Undo enabled after edit | Undo | Fresh doc | Create node | Enabled |
| Redo disabled initially | Redo | Fresh doc | None | Disabled |
| Redo enabled after undo | Redo, Undo | With node | Undo | Redo enabled |
| Copy disabled no selection | Copy | Fresh doc | None | Disabled |
| Copy enabled with selection | Copy | With node | Select node | Enabled |
| Paste disabled initially | Paste | Fresh doc | None | Disabled |
| Paste enabled after copy | Paste, Copy | With selection | Copy | Paste enabled |

---

## 5. Precondition Contracts

### 5.1 Page Setup

```typescript
async function setupFreshPage(page: Page): Promise<void> {
  await runEffectsSequential([
    () => page.goto("/"),
    () => waitForUiReady(page),
    () => clearCanvasOverlays(page),
  ]);
}
```

**Preconditions:**
- Dev server running on `/`
- No existing browser errors

**Postconditions:**
- Page loaded
- UI ready
- Canvas clear

---

## 6. Assertion Patterns

### 6.1 Disabled State

```typescript
await expect(button).toBeDisabled();
```

**Semantics:** Button has `disabled` attribute, not clickable, no `aria-disabled` only.

### 6.2 Enabled State

```typescript
await expect(button).toBeEnabled();
```

**Semantics:** Button has no `disabled` attribute, is clickable.

---

## 7. Test Isolation

Each test MUST:
1. Start from fresh page load
2. Not depend on other tests
3. Trap page errors and assert zero
4. Use `runEffectsSequential` for sequential operations

---

## 8. Violation Examples

### Invalid: Using role instead of testid

```typescript
// BAD: Unreliable if button text changes
page.getByRole("button", { name: "Undo" })
```

### Invalid: Not waiting for state

```typescript
// BAD: Race condition - state might not have updated
await createTextNode(page, canvas, 100, 100);
expect(undoButton).toBeEnabled();  // Missing await
```

### Valid: Proper testid and async

```typescript
// GOOD: Uses testid, proper async
await runEffect(() => createTextNode(page, canvas, 100, 100));
await expect(undoButton).toBeEnabled();
```

---

## 9. Error Taxonomy

| Error | Cause | Fix |
|-------|-------|-----|
| `expect(received).toBeEnabled()` but disabled | State not updated | Wait for reactivity |
| `expect(received).toBeDisabled()` but enabled | Missing disabled attr | Check button binding |
| Timeout on expect | State never reaches expected | Check state transition logic |
