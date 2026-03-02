---
bead_id: bd-81l
bead_title: tests: Implement MUL multi-select tests 3/4
phase: p0
updated_at: 2026-03-02T05:51:00Z
---

# Contract: MUL Multi-Select Tests 3/4

## Scope
Add 5 multi-select resize tests to `diagram_tool/e2e/diagram.multi-select.spec.ts`.

## Required Tests

1. **MUL-011: Resize 2-point line endpoints** - Edge endpoints update when connected nodes are resized
2. **MUL-012: Resize curved arrow** - Edge routing updates when node position changes  
3. **MUL-013: Resize past minimum clamps** - Resize clamps to minimum dimensions
4. **MUL-014: Resize past inversion flips or clamps** - Resize past opposite edge clamps without inversion
5. **MUL-015: Resize container+children** - Subgraph resize scales children proportionally

## Acceptance Criteria

- All 5 tests implemented with @baseline marker
- Tests use proper helper functions (freshStart, trapPageErrors, etc.)
- Tests verify finite dimensions and no NaN values
- Tests verify edge preservation after resize operations
- Tests verify relative positioning preserved for container+children

## Dependencies

- Requires Dioxus WASM app running (blocked by rusqlite wasm32 issue)
- Tests are written but e2e execution requires WASM build fix

