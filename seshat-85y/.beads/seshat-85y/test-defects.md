# Test Defects - seshat-85y

## Metadata
- bead_id: seshat-85y
- review_date: 2026-03-14
- status: REJECTED

---

## Critical Defects

### DEFECT-001: Testing Trophy Violation - No Real System Testing
- **Severity**: CRITICAL
- **Doctrine**: Testing Trophy (Dave Farley) - "Run the REAL thing first"
- **Location**: martin-fowler-tests.md lines 15-283 (entire file)
- **Defect**: All tests verify internal functions (`clamp_zoom`, `next_zoom_in`, `next_zoom_out`) instead of actual system behavior through UI commands
- **Evidence**: 
  - No tests call `apply_zoom_in`, `apply_zoom_out`, or `apply_zoom_reset` from `ui/commands.rs`
  - No tests verify real `DiagramDocument` state changes through the command layer
  - Tests only validate the internal `clamp_zoom` function in isolation
- **Remediation**: Add integration tests:
  ```
  #[test]
  fn apply_zoom_in_command_increases_zoom() {
      // Given: Document at zoom 1.0
      // When: apply_zoom_in(doc_signal, (800, 600))
      // Then: Document zoom is 1.25
  }
  ```

---

### DEFECT-002: ATDD Violation - No DSL Layer
- **Severity**: CRITICAL  
- **Doctrine**: Dave Farley ATDD - "Strict separation of WHAT from HOW"
- **Location**: martin-fowler-tests.md
- **Defect**: Test plan is written as implementation tests, not as user-facing specifications
- **Evidence**:
  - Test names focus on function names: `test_clamp_zoom_*`
  - No DSL expressing user behavior: "Given user at zoom 8.0, when zooms in, then zoom stays at 10.0"
- **Remediation**: Restructure tests around user behavior DSL:
  ```
  // DSL Layer: User Zoom Behavior
  scenario "user zooms in past maximum" {
      given document_at_zoom(8.0)
      when user_triggers_zoom_in()
      then zoom_is_clamped_to(10.0)
      and returns_false()
  }
  ```

---

### DEFECT-003: BDD Violation - Implementation-First Testing
- **Severity**: HIGH
- **Doctrine**: Dan North BDD - "Test behavior, not state"
- **Location**: All test names in martin-fowler-tests.md
- **Defect**: Tests verify internal function output instead of user-observable behavior
- **Evidence**:
  - ❌ `test_clamp_zoom_returns_same_value_when_within_bounds` - tests function
  - ❌ `test_clamp_zoom_returns_fallback_for_nan` - tests function
  - Should be: `test_user_provided_invalid_zoom_fallbacks_to_default`
- **Remediation**: Rename tests to express user behavior, not implementation

---

### DEFECT-004: Missing Property-Based Testing
- **Severity**: MEDIUM
- **Doctrine**: Advanced Paradigms - Exhaustive coverage
- **Location**: N/A
- **Defect**: No property-based tests for exhaustive bounds verification
- **Missing**: `proptest` tests generating random f64 values to verify:
  - `for all zoom in f64::MIN..f64::MAX, clamp_zoom(zoom) ∈ [0.1, 10.0]`
  - `for all zoom, clamp_zoom(zoom).is_finite() == true`
- **Remediation**: Add proptest:
  ```rust
  proptest! {
      #[test]
      fn clamp_zoom_always_in_bounds(zoom in f64::MIN..f64::MAX) {
          let result = clamp_zoom(zoom);
          prop_assert!(result >= 0.1 && result <= 10.0);
      }
  }
  ```

---

### DEFECT-005: Multi-Assertion Tests
- **Severity**: MEDIUM
- **Doctrine**: Kent Beck TDD - "One logical assertion per test"
- **Location**: martin-fowler-tests.md lines 224-263 (Scenarios)
- **Defect**: Scenario tests have multiple Then clauses checking separate outcomes
- **Evidence**:
  - Scenario 1 (lines 225-230): checks zoom value AND return value AND camera position
  - Scenario 4 (lines 248-254): checks all zoom values AND bounds AND camera
- **Remediation**: Split into separate tests:
  - `test_zoom_in_past_max_clamps_to_max`
  - `test_zoom_in_past_max_returns_false`
  - `test_zoom_in_past_max_preserves_camera`

---

### DEFECT-006: Missing Fuzzing
- **Severity**: LOW
- **Doctrine**: Advanced Paradigms
- **Location**: N/A
- **Defect**: No fuzzing tests for zoom input validation
- **Missing**: `cargo fuzz` or `rusfuzz` targeting zoom functions with invalid f64 values
- **Remediation**: Add fuzz target for clamp_zoom

---

## Coverage Assessment

| Doctrine | Coverage | Status |
|----------|----------|--------|
| Testing Trophy (Real Execution) | 0% | ❌ FAIL |
| ATDD (WHAT vs HOW) | 0% | ❌ FAIL |
| BDD (Behavior-first) | 20% | ⚠️ PARTIAL |
| TDD (Isolated/One-assertion) | 60% | ⚠️ PARTIAL |
| Combinatorial Permutations | 90% | ✅ GOOD |
| Advanced Paradigms | 0% | ❌ FAIL |

---

## Verdict

**STATUS: REJECTED**

The test plan must be rewritten to focus on:
1. Real system behavior through UI commands (Testing Trophy)
2. User-facing DSL specifications (ATDD)  
3. Behavior-first test names (BDD)
4. Property-based exhaustive testing (Advanced Paradigms)
