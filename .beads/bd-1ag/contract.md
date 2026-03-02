bead_id: bd-1ag
bead_title: tests: Implement CLP clipboard tests 2/2
phase: p0
updated_at: 2026-03-02T02:28:00Z

# Contract: CLP Clipboard Tests 2/2

## System Under Test

Target: `diagram_tool/e2e/diagram.clipboard.spec.ts`
Test Suite: "CLP clipboard operations @clipboard"

## Preconditions

1. Existing clipboard tests (CLP-001 through CLP-012) are implemented and passing
2. Test infrastructure supports:
   - Playwright E2E testing framework
   - Helper functions: `freshStart`, `canvas`, `createTextNode`, `expectNodeCount`, etc.
   - Page error trapping via `trapPageErrors`
3. Application supports clipboard operations:
   - Copy (Ctrl/Cmd+C)
   - Paste (Ctrl/Cmd+V)
   - Duplicate (Ctrl/Cmd+D)
   - Undo/Redo (Ctrl/Cmd+Z/Y)

## Postconditions

1. **CLP-013: Paste into Container** - Test verifies that a pasted node can be assigned to a container (subgraph) parent when pasted at appropriate position

2. **CLP-014: Drag-Drop External Image** - Test verifies drag-drop event handling for external files (image/assets) with visual feedback or placeholder creation

3. **CLP-015: Clipboard Serialization No Internal Fields** - Test verifies that clipboard serialization does NOT expose internal Rust fields (ids, revision numbers, internal state) - only serializes user-facing data

4. **CLP-016: Paste Huge Payload 1000+ Items** - Test verifies application handles large clipboard paste operations (1000+ nodes) gracefully without crash or timeout

5. **CLP-017: Empty Clipboard Paste Does Nothing** - Test verifies that paste operation with empty clipboard does not create phantom nodes or errors

## Invariants

1. All new tests follow existing naming pattern: `CLP-NNN: description @tag`
2. All tests use `trapPageErrors` to catch console errors
3. All tests use `runEffect` and `runEffectsSequential` for operations
4. Test IDs continue sequence from CLP-012
5. Tests are tagged appropriately: @baseline, @behavior, @edge-case

## Coverage Requirements

| Test ID | Scenario | Tag | Verification |
|---------|----------|-----|--------------|
| CLP-013 | Paste into container | @behavior | Node parent assignment |
| CLP-014 | Drag-drop external image | @baseline | Event handling/visual feedback |
| CLP-015 | Clipboard serialization | @security | No internal fields exposed |
| CLP-016 | Huge payload (1000+) | @edge-case | No crash, graceful handling |
| CLP-017 | Empty clipboard paste | @baseline | No phantom nodes |

## Success Criteria

1. All 5 new tests pass with `pnpm test:e2e diagram.clipboard.spec.ts`
2. No page errors during test execution
3. Tests complete within reasonable time (< 30s each for normal tests, < 60s for CLP-016)
4. Tests are deterministic and pass consistently on re-run
