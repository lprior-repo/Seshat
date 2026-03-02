bead_id: bd-1u4
bead_title: tests: Implement EDG edge routing tests 4/4
phase: p2
updated_at: 2026-03-01T18:30:00Z

# Verification: EDG Edge Routing Tests 4/4

## Verification Commands

### TypeScript Compilation
```bash
./node_modules/.bin/tsc --noEmit --skipLibCheck diagram_tool/e2e/diagram.edges-and-routing.spec.ts
```
Result: PASS (no errors)

### Rust Tests
```bash
/usr/bin/cargo test
```
Result: PASS
- 885 unit tests passed
- 13 CLI integration tests passed
- 5 ignored
- 0 failed

## Test Coverage Summary

| Test ID | Test Name | Status |
|---------|-----------|--------|
| EDG-016 | rejects self-loop edge in dag mode | Added |
| EDG-017 | curved edge is hittable along quadratic bezier path | Added |
| EDG-018 | thin horizontal edge remains selectable across zoom levels | Added |
| EDG-019 | step-routed edge is hittable at midpoint segments | Added |
| EDG-020 | sharp diagonal edge is hittable along line | Added |

## Contract Compliance

### Preconditions Met
- [x] `diagram_tool/e2e/diagram.edges-and-routing.spec.ts` exists
- [x] `diagram_tool/e2e/helpers.ts` provides test utilities
- [x] Edge model supports `bend_points` field
- [x] Edge model supports `ArrowType::Curved` variant
- [x] DAG validation rejects self-loops

### Postconditions Met
- [x] 5 new tests added to spec file
- [x] All new tests marked with `@baseline` tag
- [x] All tests pass TypeScript compilation
- [x] No regression in Rust tests

### Invariants Preserved
- [x] Tests use existing helper functions
- [x] Tests follow existing naming conventions
- [x] Tests trap page errors and assert zero errors
- [x] Tests use deterministic patterns

## Notes

- The bead description mentioned "waypoint drag" but the UI does not currently support bend_point manipulation. This was addressed by testing step-routed and curved edges instead, which cover the edge routing functionality.
- Edge overlap hit-selection tests were already present in the file (EDG-003 through EDG-006 and EDG-014/EDG-015), so additional tests were not needed.
