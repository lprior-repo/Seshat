bead_id: bd-1ag
bead_title: tests: Implement CLP clipboard tests 2/2
phase: p1
updated_at: 2026-03-02T02:30:00Z

# Implementation: CLP Clipboard Tests 2/2

## Target File

`diagram_tool/e2e/diagram.clipboard.spec.ts`

## Implementation Strategy

Add 5 new test cases (CLP-013 through CLP-017) to the existing clipboard test suite. Tests continue from CLP-012.

## Test Implementations

### CLP-013: Paste into Container with Explicit Parent Assignment

**Purpose**: Verify that a pasted node can be explicitly assigned to a container parent.

**Implementation**:
1. Create a subgraph container
2. Create a text node outside the container
3. Copy the external text node
4. Click inside the subgraph container to set context
5. Paste the node
6. Verify the pasted node is a child of the subgraph

**Verification**:
- Node count increases by 1
- Pasted node has correct parent assignment (verified via DOM structure or state)

### CLP-014: Drag-Drop External Image with File Input

**Purpose**: Test external file drop handling via programmatic file input.

**Implementation**:
1. Use Playwright's `.setInputFiles` on file input or drag-drop simulation
2. Trigger file selection for an image file
3. Verify image node creation or placeholder appears
4. Check for no page errors during file handling

**Note**: External file drop may not be fully implemented; test verifies basic file input handling.

**Verification**:
- No page errors
- File input event is processed (even if placeholder)

### CLP-015: Clipboard Serialization No Internal Fields

**Purpose**: Verify clipboard serialization does NOT expose internal Rust fields.

**Implementation**:
1. Create multiple nodes with edges
2. Select and copy to clipboard
3. Read clipboard content via `page.evaluate()` accessing clipboard API
4. Parse JSON if applicable
5. Verify NO internal fields are present:
   - No `__rust_field` prefixes
   - No raw memory addresses
   - No internal IDs exposed (only user-facing IDs)
   - No revision numbers in serialized data
   - No internal state markers

**Verification**:
- Clipboard content is clean JSON
- Only user-facing properties are present
- No internal implementation details leaked

### CLP-016: Paste Huge Payload 1000+ Items

**Purpose**: Stress test - verify app handles large clipboard paste without crash.

**Implementation**:
1. Programmatically create 1000+ nodes via `page.evaluate()`
2. Select all nodes
3. Copy to clipboard
4. Paste the huge payload
5. Verify app doesn't crash or timeout
6. Check that operation completes within 60 seconds
7. Verify node count increased appropriately

**Note**: This is a stress test; some lag is acceptable but no crash.

**Verification**:
- No page errors
- Operation completes within timeout
- Node count reflects the paste operation

### CLP-017: Empty Clipboard Paste Does Nothing

**Purpose**: Verify paste with empty clipboard doesn't create phantom nodes.

**Implementation**:
1. Start with empty canvas
2. Ensure clipboard is empty (no prior copy)
3. Trigger paste operation (Ctrl/Cmd+V)
4. Verify node count remains 0
5. Verify no page errors
6. Verify no selection changes

**Verification**:
- Node count: 0
- Selected count: 0
- No page errors

## Code Pattern

All tests follow existing pattern:
```typescript
test("CLP-NNN: description @tag", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  const canvasEl = await setupCanvas(page);

  // ... test implementation ...

  expect(pageErrors).toHaveLength(0);
});
```

## Integration Points

- Uses existing helpers: `setupCanvas`, `createTextNode`, `runEffect`, `runEffectsSequential`
- Uses existing helper: `createSubgraphWithChild` for container tests
- May need new helper for programmatic node creation (CLP-016)

## Files Modified

- `diagram_tool/e2e/diagram.clipboard.spec.ts` (append 5 new tests)

## Success Criteria

1. All 5 new tests pass
2. No TypeScript compilation errors
3. Tests complete within timeout limits
4. Tests are deterministic and reproducible
