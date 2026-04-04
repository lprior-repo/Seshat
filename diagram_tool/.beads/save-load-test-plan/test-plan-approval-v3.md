# Test Plan Review: save-load-test-plan

## STATUS: REJECTED

---

## Summary

The test plan demonstrates thorough coverage with 47 behaviors, 52 unit tests,
16 integration tests, 4 proptest invariants, 3 fuzz targets, and 2 Kani harnesses.
Trophy allocation is appropriate (5.2x ratio). All 10 public functions have BDD
scenarios. However, 3 MAJOR findings require resolution before approval.

---

## Axis 1 — Contract Parity: PASS (with reservation)

All 10 public functions have BDD scenarios. All error variants are referenced.

**Reservation**: OpenError::Io is marked as "unreachable from apply_open_document"
in coverage table but listed as covered — see MAJOR #1.

---

## Axis 2 — Assertion Sharpness: FAIL

Behaviors 54, 36 use `>=` for unchanged assertions. Should be `==`.

---

## Axis 3 — Trophy Allocation: PASS

52 unit / 16 integration / 2 e2e / 0 static. Ratio 5.2x ≥ 5x. ✓
4 proptest invariants for pure calc functions. ✓
3 fuzz targets for parsers. ✓
2 Kani harnesses for path traversal. ✓

---

## Axis 4 — Boundary Completeness: PASS

Empty documents, empty strings, WASM variants, path traversal, temp file
failures, rename failures — all explicitly named per function.

---

## Axis 5 — Mutation Survivability: PASS (on document)

Section 7 mutation checkpoints are comprehensive. 90% threshold specified.

---

## Axis 6 — Holzmann Plan Audit: PASS

Given-When-Then structure with explicit preconditions. No loops in test bodies.
Side effects advertised through toast/signal patterns.

---

## LETHAL FINDINGS

None.

---

## MAJOR FINDINGS (3 — REJECTION THRESHOLD MET)

### MAJOR #1: OpenError::Io — Coverage Table Inconsistency
- **Location**: test-plan.md:1186
- **Contract gap**: contract.md does not specify how CliPersistenceError::IoError
  maps to OpenError::Io
- **Required fix**: Either (a) add explicit mapping note to contract.md, OR
  (b) remove OpenError::Io from coverage table and mark it "action layer only"

### MAJOR #2: Toast Unchanged Assertions Use >= Instead of ==
- **Location**: test-plan.md:578, 602
- **Affected**: Behaviors 54, 36
- **Required fix**: Change `doc.nodes.len() >= 3` to `doc.nodes.len() == original_count`
  and `history` still has prior state to use exact equality checks:
  - Store `original_node_count = doc.nodes.len()` before call
  - Assert `doc.nodes.len() == original_node_count` after failed call
  - Same pattern for `history.can_undo()` — store prior state, compare exactly

### MAJOR #3: SaveError Contract Missing CliPersistenceError Mapping
- **Location**: contract.md:30-37
- **Gap**: 3-variant SaveError (NoFilePath, Serialize, Io) must accommodate
  TempFileError, AtomicRenameError, and PathTraversalDenied via mapping to Io,
  but this mapping is not documented in contract.md
- **Required fix**: Add mapping note to contract.md Section 4 or Error Enum section:
  "CliPersistenceError::TempFileError, CliPersistenceError::AtomicRenameError,
  and CliPersistenceError::PathTraversalDenied are mapped to SaveError::Io
  for the apply_save_document interface"

---

## MINOR FINDINGS (3/5 — below rejection threshold)

1. test-plan.md:588 — typo in invalid JSON example: `"{invalid json")`
   should be `"\"{invalid json\""` or similar
2. Section 9 — ParseError/ValidationError entries don't specify source function
3. Section 9 note references "Behavior 314" and "Behavior 320" which don't
   exist in the behavior inventory (should be renumbered to actual behavior numbers)

---

## MANDATE

Before resubmission, the following must be addressed:

1. **MAJOR #1**: Clarify OpenError::Io coverage — add mapping to contract.md
   OR fix coverage table attribution to `open_workspace` (action layer), not
   `apply_open_document` (calc layer)

2. **MAJOR #2**: Fix Behaviors 54 and 36 to use `==` for unchanged assertions:
   - Store `original_node_count = doc.nodes.len()` before call
   - Assert `doc.nodes.len() == original_node_count` after failed call
   - Same pattern for `history.can_undo()` — store prior state, compare exactly

3. **MAJOR #3**: Add error mapping documentation to contract.md:
   - Section 4 (WASM vs Native) or Error Enum section should note that
     CliPersistenceError variants (TempFileError, AtomicRenameError,
     PathTraversalDenied) map to SaveError::Io

4. **After fixes**: Re-run full Plan Inquisition from Axis 1

---

## Remaining Strengths

- 47 behaviors covering all 10 public functions
- Appropriate trophy distribution (unit/integration/e2e)
- 4 proptest invariants for pure calc functions with non-trivial inputs
- 3 fuzz targets for parse_diagram_document_with_compat and apply_import_contents
- 2 Kani harnesses for path traversal and atomicity
- Section 7 mutation checkpoints with named catching tests
- IO error wildcards (SaveError::Io(_)) correctly marked as intentional due to
  unpredictable OS messages
- String checks for validation/parse errors correctly noted as acceptable since
  our code generates those messages

---

*Inquisitor: test-reviewer skill | Mode 1 Plan Inquisition*
*Date: 2026-04-04*
