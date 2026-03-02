---
bead_id: bd-2df
bead_title: tests: Implement EDG edge routing tests 3/4
phase: p2
updated_at: 2026-03-02T00:22:00Z
---

# Verification: EDG Edge Routing Tests 3/4

## Validation Results

### TypeScript Type Check
```
$ npx tsc --noEmit --project diagram_tool/e2e/tsconfig.json
(exit code: 0)
```
PASS - No type errors.

### Playwright Test List
```
$ npx playwright test --list diagram_tool/e2e/diagram.edges-and-routing.spec.ts
Total: 24 tests in 1 file
```
PASS - All 5 new tests are recognized:
1. `edge between nodes in same container @baseline` (EDG-011)
2. `edge crossing container boundary @baseline` (EDG-012)
3. `reparent node with connected edge produces valid state @baseline` (EDG-013)
4. `horizontal edge overlap hit-selection is deterministic across repeated clicks @baseline` (EDG-014)
5. `vertical edge overlap hit-selection is deterministic across repeated clicks @baseline` (EDG-015)

### Contract Coverage

| Test ID | Requirement | Status |
|---------|-------------|--------|
| EDG-011 | Edge between nodes in same container | Implemented |
| EDG-012 | Edge crossing container boundary | Implemented |
| EDG-013 | Reparent node with edges | Implemented |
| EDG-014 | Edge routing stable on overlapping nodes (horizontal) | Implemented |
| EDG-015 | Edge routing stable on overlapping nodes (vertical) | Implemented |

### Code Quality Checks
- [x] Uses `freshStart()` for clean state
- [x] Uses `runEffect()`/`runEffectsSequential()` for deterministic operations
- [x] Uses `trapPageErrors()` for error detection
- [x] Uses `expectEdgeCount()` and `expectNodeCount()` for assertions
- [x] Follows existing test patterns in the file
- [x] All tests marked with `@baseline` tag

## Notes
- E2E runtime tests require a running server and are not executed in this verification phase
- Static analysis (TypeScript type check + Playwright test list) confirms tests are syntactically correct
