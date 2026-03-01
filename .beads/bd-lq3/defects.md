---
bead_id: bd-lq3
bead_title: tests: Implement SUB subgraph tests 2/4
phase: p2
updated_at: 2026-03-01T22:52:00Z
---

# Defects: SUB Subgraph Tests 2/4

## Defect #1: Test Infrastructure - Rebuild Overlay Timeout

### Description
All e2e tests are failing because the dx serve dev server triggers a rebuild when test files are modified. The "Your app is being rebuilt" overlay appears and doesn't disappear within the 60-second timeout.

### Evidence
```
Error: expect(received).toBe(expected) // Object.is equality
Expected: true
Received: false
Call Log: Timeout 30000ms exceeded while waiting on the predicate
```

Error context shows:
```yaml
- heading [level=3]: Your app is being rebuilt.
```

### Impact
- All 5 new subgraph behavior tests fail
- Other existing tests also fail with the same issue
- Tests cannot be validated

### Root Cause
The dx serve process watches for file changes and rebuilds the WASM application. When test files are modified (even just touched), the dev server triggers a rebuild. The rebuild takes longer than the `waitForNoRebuildOverlay` timeout.

### Recommended Fix
1. Exclude test files from dx serve watch list
2. Use a pre-built static server for e2e tests
3. Increase the timeout in `waitForNoRebuildOverlay` to 120 seconds

### Status
**BLOCKING** - Tests are correctly implemented but cannot be validated due to infrastructure issue.
