# Defects Found: SEL-002 Edge Selection Tests

## Status: REJECTED

---

## Defect 1: Missing Test Case - Bend Points

**Severity**: Medium

**Contract Clause**: Edge Case Tests (line 190-196 of contract.md)

**Issue**: The contract specifies `test_sel_002_given_edge_with_bend_points_when_clicking_on_bend_then_edge_selected` but this test is NOT implemented.

**Expected**: A test that creates an edge with bend points and verifies clicking on a bend selects the edge.

**Actual**: Only 3 edge case tests exist:
- Horizontal edge at endpoint
- Vertical edge
- Diagonal edge

Missing: Bend points test.

---

## Defect 2: Uses of `.unwrap()` in Test Code

**Severity**: Low

**Contract Clause**: Functional-Rust Rules (from AGENTS.md)

**Issue**: Multiple `.unwrap()` calls found in test module despite functional-rust skill explicitly forbidding `.unwrap()` and `.expect()` calls.

**Locations**:
- Line 953: `let edge_id = hit.unwrap();`
- Line 973: `hit.unwrap()`
- Line 1038: `hit.unwrap()`
- Line 1063: `hit.unwrap()`
- Line 1087: `hit.unwrap()`
- Line 1137: `hit.unwrap()`
- Line 1155: `hit.unwrap()`
- Line 1172: `hit.unwrap()`
- Line 1243: `hit.unwrap()`
- Line 1267: `hit.unwrap()`

**Note**: While these are guarded by assertions (which provides a better error message than a raw panic), the functional-rust rules require using `if let` or `match` instead.

**Recommendation**: Replace patterns like:
```rust
assert!(hit.is_some(), "message");
hit.unwrap()
```

With:
```rust
if let Some(edge_id) = hit {
    // test logic
} else {
    panic!("message");
}
```

Or better, use `match` for explicit handling.

---

## Summary

| Defect | Severity | Type |
|--------|----------|------|
| Missing bend points test | Medium | Missing coverage |
| Uses of `.unwrap()` | Low | Code style violation |

The implementation has 16 tests instead of the 17 specified in the contract (missing bend points test).
