# Martin Fowler Test Plan

## Happy Path Tests
- `test_can_undo_returns_true_when_undo_stack_has_elements`
- `test_can_redo_returns_true_when_redo_stack_has_elements`
- `test_can_undo_returns_false_when_undo_stack_empty`
- `test_can_redo_returns_false_when_redo_stack_empty`

## Error Path Tests
None - these are infallible methods.

## Edge Case Tests
- `test_can_undo_on_fresh_history_returns_false`
- `test_can_redo_on_fresh_history_returns_false`
- `test_can_undo_after_single_push_returns_true`
- `test_can_redo_after_single_undo_returns_true`
- `test_can_redo_after_push_clears_stack_returns_false`
- `test_can_undo_at_history_boundary_returns_correct_value`

## Contract Verification Tests
- `test_invariant_can_undo_is_o1`
- `test_invariant_can_redo_is_o1`
- `test_postcondition_can_undo_no_mutation`
- `test_postcondition_can_redo_no_mutation`

## Contract Violation Tests

(These test that implementation correctly satisfies postconditions)

- `test_postcondition_q1_can_undo_true_iff_undo_stack_nonempty`
  Given: History with 3 elements pushed
  When: `can_undo()` is called
  Then: returns `true` (undo_stack.len() == 3)

- `test_postcondition_q1_can_undo_false_iff_undo_stack_empty`
  Given: Fresh `History::new()` with empty undo_stack
  When: `can_undo()` is called
  Then: returns `false` (undo_stack.is_empty())

- `test_postcondition_q2_can_redo_true_iff_redo_stack_nonempty`
  Given: History after undo operation (redo_stack has 1 element)
  When: `can_redo()` is called
  Then: returns `true` (redo_stack.len() == 1)

- `test_postcondition_q2_can_redo_false_iff_redo_stack_empty`
  Given: Fresh history after push (redo_stack cleared)
  When: `can_redo()` is called
  Then: returns `false` (redo_stack.is_empty())

- `test_postcondition_q3_can_undo_does_not_mutate`
  Given: History with known undo_stack and redo_stack contents
  When: `can_undo()` is called
  Then: undo_stack and redo_stack remain unchanged

- `test_postcondition_q3_can_redo_does_not_mutate`
  Given: History with known undo_stack and redo_stack contents
  When: `can_redo()` is called
  Then: undo_stack and redo_stack remain unchanged

## Given-When-Then Scenarios

### Scenario 1: Fresh history has no undo available
Given: A newly created `History::new()`
When: `can_undo()` is called
Then:
- Returns `false`
- No state change occurs

### Scenario 2: Fresh history has no redo available
Given: A newly created `History::new()`
When: `can_redo()` is called
Then:
- Returns `false`
- No state change occurs

### Scenario 3: After push, undo becomes available
Given: A fresh history
When: `push(doc)` is called
Then:
- `can_undo()` returns `true`
- `can_redo()` returns `false`

### Scenario 4: After undo, redo becomes available
Given: History with one push
When: `undo(current)` is called successfully
Then:
- On resulting history, `can_redo()` returns `true`
- On resulting history, `can_undo()` returns `false`

### Scenario 5: After push following undo, redo is cleared
Given: History with undo performed (redo available)
When: `push(new_doc)` is called
Then:
- On resulting history, `can_redo()` returns `false`

### Scenario 6: Multiple undos tracked correctly
Given: History with 5 pushes
When: Checking `can_undo()`
Then:
- Returns `true`
- After 5 undos, `can_undo()` returns `false`

### Scenario 7: Boundary at MAX_HISTORY (100)
Given: History with exactly 100 pushes
When: Checking `can_undo()`
Then:
- Returns `true` (at capacity boundary)
- After 100 undos, `can_undo()` returns `false`

### Scenario 8: Both stacks can have elements simultaneously
Given: History with 3 pushes, then 1 undo
When: Checking both methods
Then:
- `can_undo()` returns `true` (2 elements in undo_stack)
- `can_redo()` returns `true` (1 element in redo_stack)

## Test Implementation Notes

```rust
#[test]
fn test_can_undo_returns_true_when_undo_stack_has_elements() {
    let history = History::new().push(DiagramDocument::default());
    assert!(history.can_undo());
}

#[test]
fn test_can_redo_returns_true_when_redo_stack_has_elements() {
    let history = History::new().push(DiagramDocument::default());
    let (_, after_undo) = history.undo(DiagramDocument::default()).unwrap();
    assert!(after_undo.can_redo());
}

#[test]
fn test_can_undo_returns_false_when_undo_stack_empty() {
    let history = History::new();
    assert!(!history.can_undo());
}

#[test]
fn test_can_redo_returns_false_when_redo_stack_empty() {
    let history = History::new().push(DiagramDocument::default());
    assert!(!history.can_redo());
}

#[test]
fn test_postcondition_q3_can_undo_does_not_mutate() {
    let history = History::new()
        .push(doc_with_revision(1))
        .push(doc_with_revision(2));
    let undo_len_before = history.undo_stack.len();
    let redo_len_before = history.redo_stack.len();
    
    let _ = history.can_undo();
    
    assert_eq!(history.undo_stack.len(), undo_len_before);
    assert_eq!(history.redo_stack.len(), redo_len_before);
}

#[test]
fn test_postcondition_q3_can_redo_does_not_mutate() {
    let history = History::new()
        .push(doc_with_revision(1))
        .push(doc_with_revision(2));
    let Some((_, after_undo)) = history.undo(doc_with_revision(100)) else {
        panic!("undo should succeed");
    };
    
    let undo_len_before = after_undo.undo_stack.len();
    let redo_len_before = after_undo.redo_stack.len();
    
    let _ = after_undo.can_redo();
    
    assert_eq!(after_undo.undo_stack.len(), undo_len_before);
    assert_eq!(after_undo.redo_stack.len(), redo_len_before);
}
```
