# Martin Fowler Test Plan

## Feature: Disabled states for Undo/Redo toolbar buttons

### Given-When-Then Scenarios

---

## Scenario 1: Undo button disabled on fresh document

**Given** a new diagram document with no history
**When** the toolbar renders
**Then** the Undo button has `disabled="true"` attribute
**And** the Undo button has `opacity: 0.4`
**And** the Undo button has `cursor: not-allowed`
**And** clicking the Undo button does nothing

**Test command:**
```bash
# E2E: Fresh document, verify undo disabled
moon run :e2e-smoke -- --grep "undo button disabled on fresh document"
```

---

## Scenario 2: Redo button disabled on fresh document

**Given** a new diagram document with no history
**When** the toolbar renders
**Then** the Redo button has `disabled="true"` attribute
**And** the Redo button has `opacity: 0.4`
**And** the Redo button has `cursor: not-allowed`

**Test command:**
```bash
moon run :e2e-smoke -- --grep "redo button disabled on fresh document"
```

---

## Scenario 3: Undo button enabled after action

**Given** a diagram document with one node added
**When** the toolbar renders
**Then** the Undo button has `disabled="false"` or no disabled attribute
**And** the Undo button has `opacity: 1.0`
**And** the Undo button has `cursor: pointer`

**Test command:**
```bash
# Add node, verify undo enabled
moon run :e2e-smoke -- --grep "undo button enabled after action"
```

---

## Scenario 4: Redo button enabled after undo

**Given** a diagram document with one node added
**And** the user clicks Undo
**When** the toolbar renders
**Then** the Redo button has `disabled="false"` or no disabled attribute
**And** the Redo button has `opacity: 1.0`

**Test command:**
```bash
# Add node, undo, verify redo enabled
moon run :e2e-smoke -- --grep "redo button enabled after undo"
```

---

## Scenario 5: Both buttons disabled after undo/redo cycle

**Given** a diagram document with one node added
**And** the user clicks Undo
**And** the user clicks Redo
**When** the toolbar renders
**Then** the Undo button is enabled (can undo the add)
**And** the Redo button is disabled (redo_stack empty)

**Test command:**
```bash
moon run :e2e-smoke -- --grep "buttons state after undo redo cycle"
```

---

## Scenario 6: New action clears redo availability

**Given** a diagram document with one node added
**And** the user clicks Undo (Redo now available)
**And** the user adds another node
**When** the toolbar renders
**Then** the Redo button is disabled (new action cleared redo_stack)

**Test command:**
```bash
moon run :e2e-smoke -- --grep "new action clears redo availability"
```

---

## Scenario 7: Clicking disabled button does nothing

**Given** a fresh diagram document (Undo disabled)
**When** the user attempts to click the Undo button
**Then** no undo action is dispatched
**And** the document state remains unchanged

**Test command:**
```bash
moon run :e2e-smoke -- --grep "disabled button click does nothing"
```

---

## Scenario 8: Visual consistency with Delete button pattern

**Given** any document state
**When** the toolbar renders
**Then** disabled Undo/Redo buttons use same opacity (0.4) as disabled Delete button
**And** disabled Undo/Redo buttons use same cursor (not-allowed) as disabled Delete button

**Test command:**
```bash
moon run :e2e-smoke -- --grep "visual consistency disabled buttons"
```

---

## Unit Test Contract

```rust
#[test]
fn undo_button_respects_can_undo() {
    // Verify disabled attribute binds to !can_undo()
}

#[test]
fn redo_button_respects_can_redo() {
    // Verify disabled attribute binds to !can_redo()
}

#[test]
fn disabled_opacity_applied() {
    // Verify opacity: 0.4 when disabled
}

#[test]
fn disabled_cursor_applied() {
    // Verify cursor: not-allowed when disabled
}
```

## Boundary Cases

| Case | Expected Undo | Expected Redo |
|------|---------------|---------------|
| Fresh document | Disabled | Disabled |
| 1 action, 0 undos | Enabled | Disabled |
| 1 action, 1 undo | Disabled | Enabled |
| 5 actions, 3 undos | Enabled | Enabled |
| 5 actions, 5 undos | Disabled | Enabled |

## Acceptance Criteria Checklist

- [ ] Undo button has `disabled` attribute when `!can_undo()`
- [ ] Redo button has `disabled` attribute when `!can_redo()`
- [ ] Disabled buttons have `opacity: 0.4`
- [ ] Disabled buttons have `cursor: not-allowed`
- [ ] Enabled buttons have `opacity: 1.0`
- [ ] Enabled buttons have `cursor: pointer`
- [ ] Clicking disabled button does not dispatch action
- [ ] Pattern matches existing Delete button implementation
