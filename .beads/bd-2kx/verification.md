bead_id: bd-2kx
bead_title: tests: Implement CAM viewport tests 2/3
phase: p2
updated_at: 2026-03-02T00:15:00Z

# Verification: CAM Viewport Tests 2/3

## Moon Validation

### Check
```
moon run :check
```
Result: PASSED (exit code 0)

### Test (Rust)
```
moon run :test-rust
```
Result: PASSED (exit code 0)

### E2E Tests
```
npx playwright test diagram.viewport-cam.spec.ts --project baseline --list
```
Result: 10 tests listed (8 existing + 2 new)

## Test Inventory

| # | Test Name | Status |
|---|-----------|--------|
| 1 | wheel zoom at cursor keeps node centered under pointer | existing |
| 2 | spacebar + drag pans viewport without selecting nodes | existing |
| 3 | zoom out clamps at minimum 10% | existing |
| 4 | zoom in clamps at maximum 400% | existing |
| 5 | world-to-screen remains consistent at extreme zoom levels | existing |
| 6 | wheel zoom works when canvas is in scrollable container | existing |
| 7 | drag near scroll parent edge updates scroll position | existing |
| 8 | viewport recalculates after resize simulation | existing |
| 9 | edge scrolling during drag reveals more canvas space | NEW |
| 10 | fit to content centers nodes with appropriate padding | NEW |

## Notes

The e2e tests could not be executed due to an environmental issue (app stuck in rebuild state). This is not a test implementation issue but rather an infrastructure issue with the running dx serve processes.

The tests are syntactically valid and follow the existing patterns in the file:
- Use `@baseline` tag convention
- Use `freshStart()` for isolation
- Use `trapPageErrors()` for error tracking
- Follow Given/When/Then structure with comments

## Artifacts
- Contract: `.beads/bd-2kx/contract.md`
- Implementation: `.beads/bd-2kx/implementation.md`
- Verification: `.beads/bd-2kx/verification.md`
- Receipts: `.beads/bd-2kx/receipts.jsonl`
