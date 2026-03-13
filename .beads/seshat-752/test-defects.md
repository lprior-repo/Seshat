# Test Defects - seshat-752

**Reviewer:** test-reviewer skill  
**Date:** 2026-03-12  
**Status:** REJECTED  
**Round:** 2 (Addressing round 1 defects)

---

## Previous Round Status

The following defects from Round 1 were addressed:
- ✅ Added DSL section with Gherkin syntax (lines 10-34)
- ✅ Fixed test names to be behavior-focused (`given_X_when_Y_then_Z`)
- ✅ Added fuzzing tests (lines 83-88)
- ✅ Added DomainOp variants coverage table (lines 116-127)

---

## New Defects Found (Round 2)

### D1: Missing Explicit Tests for NodeAdd and NodeDelete

**Severity:** HIGH  
**Category:** Combinatorial Permutations / Coverage Gap  
**Location:** martin-fowler-tests.md lines 40-48, 120-127

### Description
The "DomainOp Variants Coverage" table claims ✅ coverage for NodeAdd and NodeDelete, but the explicit "Happy Path Tests" table only contains UpdateLabel tests. No explicit test cases exist for:
- NodeAdd (node_id field)
- NodeDelete (node_id field)

### Evidence
- Lines 120-127: Coverage table shows "✅" for NodeAdd and NodeDelete
- Lines 40-48: Happy Path table only has UpdateLabel tests
- Line 45: Test name mentions "valid node JSON" but is under UpdateLabel row

### Required Fix
Add explicit test rows to Happy Path table:
```
| given_valid_nodeid_when_creating_node_then_nodeid_wraps_id | Valid node JSON with non-empty id | NodeAdd processed | Returns Ok(NodeAdd { node_id: NodeId(...) }) |
| given_valid_nodeid_when_deleting_node_then_nodeid_wrapped | Valid node id | NodeDelete processed | Returns Ok(NodeDelete { node_id: NodeId(...) }) |
```

---

### D2: Missing EdgeAdd Explicit Test

**Severity:** HIGH  
**Category:** Combinatorial Permutations / Coverage Gap  
**Location:** martin-fowler-tests.md lines 40-48, 120-127

### Description
EdgeAdd is marked as ✅ in coverage table but has no explicit test case in the Happy Path table.

### Evidence
- Line 125: Coverage table shows EdgeAdd with ✅
- Line 47: Test name exists `given_valid_source_and_target_when_adding_edge_then_edgeid_created` but is NOT in the explicit Happy Path table - only mentioned in passing

### Required Fix
Add explicit EdgeAdd row to Happy Path Tests table.

---

### D3: Whitespace Contract Mismatch

**Severity:** HIGH  
**Category:** ATDD / Contract Alignment  
**Location:** martin-fowler-tests.md line 63, contract.md lines 98-106

### Description
The test plan expects whitespace-only strings to be rejected, but the contract specification only checks `is_empty()`, not whitespace.

### Evidence
- martin-fowler-tests.md line 63:
  > `given_whitespace_only_when_creating_nodeid_then_error_returned`

- contract.md lines 100-102:
  ```rust
  if s.is_empty() {
      Err(ContractError::InvalidNodeId(s))
  }
  ```

### Root Cause
Contract implementation only checks `is_empty()` (empty string), NOT whitespace. Test expects whitespace rejection which is NOT specified in contract.

### Required Fix
**Option A (Test Fix - Recommended):** Remove the whitespace test since contract doesn't require it:
```
| given_empty_string_when_creating_nodeid_then_invalidnodeid_returned | "" | NodeId created | Returns Err(InvalidNodeId) |
```

**Option B (Contract Fix):** Update contract.md NodeId::new to also reject whitespace:
```rust
if s.trim().is_empty() {
    Err(ContractError::InvalidNodeId(s))
}
```
Then update test to match.

---

### D4: Property Test Not Matching Implementation

**Severity:** MEDIUM  
**Category:** ATDD / Specification Accuracy  
**Location:** martin-fowler-tests.md line 79

### Description
`property_all_valid_nodeids_are_nonempty` claims to verify non-empty NodeIds, but implementation only checks `is_empty()`, not whitespace.

### Evidence
- Line 79: Property claims "All successful NodeId creations return non-empty"
- contract.md: Implementation checks `is_empty()` only

### Required Fix
Either:
1. Update property test name to match: `property_all_valid_nodeids_are_nonempty_strings` 
2. Or update contract to reject whitespace and then update property

---

### D5: Vague Fuzzing Specification

**Severity:** MEDIUM  
**Category:** ATDD / Precise Specification  
**Location:** martin-fowler-tests.md lines 87-88

### Description
Fuzzing test specification is ambiguous - "either reject or sanitize" is not testable.

### Evidence
- Line 88: `fuzz_nodeid_creation_rejects_garbage`: "Random garbage strings either reject or sanitize"

### Required Fix
Specify exact expected behavior:
```
| fuzz_nodeid_creation_rejects_invalid_input | Random garbage/non-printable strings | NodeId::new() | Returns Err(InvalidNodeId) - never panics |
```

---

## Verification Commands

```bash
# Check for explicit NodeAdd/NodeDelete tests in test tables
grep -E "NodeAdd|NodeDelete" martin-fowler-tests.md

# Verify contract whitespace handling
grep -A5 "impl NodeId" contract.md

# Count explicit test rows in Happy Path table
grep -c "|" martin-fowler-tests.md | head -5
```

---

## Resolution Checklist

- [ ] D1: Add explicit NodeAdd test case to Happy Path table
- [ ] D1: Add explicit NodeDelete test case to Happy Path table  
- [ ] D2: Add explicit EdgeAdd test case to Happy Path table
- [ ] D3: Choose Option A (remove whitespace test) or Option B (update contract)
- [ ] D4: Update property test name to match contract behavior
- [ ] D5: Make fuzzing specification precise and testable

---

## Doctrine Compliance Score

| Doctrine | Previous Round | Current Round |
|----------|----------------|---------------|
| Dan North BDD | ✅ Fixed | ✅ Pass (GWT naming present) |
| Dave Farley ATDD | ✅ Fixed | ⚠️ Contract mismatch (D3) |
| Kent Beck TDD | N/A | ✅ Pass (isolated tests) |
| Testing Trophy | ✅ Fixed | ⚠️ Incomplete coverage (D1, D2) |
| Combinatorial Permutations | ✅ Fixed | ❌ Missing explicit tests |
| Advanced Paradigms | ✅ Fixed | ⚠️ Vague fuzzing (D5) |

---

## Summary

| Defect | Severity | Status |
|--------|----------|--------|
| No DSL defined (Round 1) | CRITICAL | ✅ RESOLVED |
| Implementation details in names (Round 1) | HIGH | ✅ RESOLVED |
| Missing fuzzing (Round 1) | MEDIUM | ✅ RESOLVED |
| Incomplete op_type coverage (Round 1) | MEDIUM | ✅ RESOLVED |
| Missing NodeAdd/NodeDelete explicit tests | HIGH | ❌ NEW |
| Missing EdgeAdd explicit test | HIGH | ❌ NEW |
| Whitespace contract mismatch | HIGH | ❌ NEW |
| Property test mismatch | MEDIUM | ❌ NEW |
| Vague fuzzing specification | MEDIUM | ❌ NEW |

**Recommendation:** Fix all HIGH severity defects before approval. D3 (whitespace) is the most critical ATDD violation - the test must match the contract.
