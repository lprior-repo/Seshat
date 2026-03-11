# Test Defects - seshat-c0j

## Critical Defects

### DEFECT-001: Missing Test Plan File
- **Severity**: CRITICAL
- **File**: `/home/lewis/src/seshat/.beads/seshat-c0j/martin-fowler-tests.md`
- **Issue**: The `martin-fowler-tests.md` file does not exist. Only `contract.md` is present.
- **Impact**: No executable specifications exist for this feature. This violates the core ATDD/BDD workflow where test plans must be authored BEFORE implementation.
- **Required Action**: Create `martin-fowler-tests.md` with Given-When-Then test cases derived from the contract.

---

## Contract Analysis (contract.md exists but is untested)

### Positive Observations
- EARS requirements are well-defined (EARS-1 through EARS-4)
- Preconditions P1-P4 cover node existence and dimension validation
- Postconditions Q1-Q6 cover width/height updates and non-mutation of other fields
- Violation examples are provided (lines 82-91)
- Error taxonomy is clear

### Missing Test Coverage Requirements
The contract specifies these test scenarios that MUST be in martin-fowler-tests.md:

1. **Happy Path Tests** (per Q1, Q2):
   - Apply NodeResize with valid dimensions → width/height updated correctly

2. **Unhappy Path Tests** (per P2, P3, P4):
   - Node not found → ProjectionError::NodeNotFound
   - NaN width → ProjectionError::InvalidDimensions
   - Infinity width → ProjectionError::InvalidDimensions
   - Negative width → ProjectionError::InvalidDimensions
   - NaN height → ProjectionError::InvalidDimensions
   - Infinity height → ProjectionError::InvalidDimensions
   - Negative height → ProjectionError::InvalidDimensions

3. **Edge Cases** (per Q3, Q4, Q5, Q6):
   - Node.x unchanged after resize
   - Node.y unchanged after resize
   - Node.label unchanged after resize
   - Other nodes in document unchanged after resize
   - Document revision incremented after resize

4. **Invariant Tests** (per INV-1, INV-2, INV-3):
   - Document remains valid (all nodes have positive finite dimensions)
   - No nodes deleted/added
   - Edges unaffected

---

## Summary

**Status**: REJECTED - Missing test plan file entirely. Cannot review tests against North/Farley/Beck/Testing Trophy doctrines when the test plan does not exist.
