# Test Defects - seshat-it8

## Critical Defects Found

### 1. BDD/Dan North Violations

| ID | Defect | Location |
|----|--------|----------|
| BDD-001 | **Malformed content** - Lines 22-24 have broken/mangled text: "Then: Returns Ok(NuOutput)test_value" followed by orphaned "with stdout containing" fragment | martin-fowler-tests.md:22-24 |
| BDD-002 | **Duplicate test behavior** - `test_returns_error_when_command_empty` (line 30) and `test_returns_error_when_command_whitespace_only` (line 35) test identical behavior with different inputs, but both belong in the same test as edge cases - not separate test cases | martin-fowler-tests.md:30-38 |
| BDD-003 | **Backtick typo** in test name breaks parsing | martin-fowler-tests.md:107 |

### 2. ATDD/Dave Farley Violations

| ID | Defect | Location |
|----|--------|----------|
| ATDD-001 | **No DSL separation** - Tests are written as implementation code rather than as a specification DSL. The contract.md has clauses but martin-fowler-tests.md does not translate them into an executable specification layer | Overall structure |
| ATDD-002 | **Missing executable specification** - The Given-When-Then scenarios (lines 140-173) read like documentation, not executable tests. No DSL is provided for stakeholders to write tests in business language | martin-fowler-tests.md:140-173 |

### 3. Testing Trophy/Dave Farley Violations

| ID | Defect | Location |
|----|--------|----------|
| TT-001 | **No E2E tests defined** - Evaluation Protocol (contract.md:65-71) mentions "Integration Tests" but no actual E2E test scenarios are described in martin-fowler-tests.md | Overall gap |
| TT-002 | **No real execution validation** - All tests are unit-level. No test validates the system actually works end-to-end with real Nushell binary | martin-fowler-tests.md |
| TT-003 | **Missing resource leak verification** - Q4 (No file descriptor leaks) has no corresponding test in the test plan | contract.md:31 vs martin-fowler-tests.md |

### 4. Combinatorial Coverage Violations

| ID | Defect | Location |
|----|--------|----------|
| CC-001 | **INVARIANT I3 MISALIGNED** - Contract says "Environment vars persist only for next command" (I3, line 39), but test `test_invariant_i3_env_vars_do_not_persist` (line 107) asserts they do NOT persist. This is contradictory. | contract.md:39 vs martin-fowler-tests.md:107-110 |
| CC-002 | **Missing property-based tests** - No tests verify invariants across random command sequences | Overall gap |
| CC-003 | **Missing fuzz tests** - No boundary/parser fuzzing for command strings | Overall gap |
| CC-004 | **Duplicate test content** - "Contract Violation Tests" section (lines 112-138) duplicates tests already in "Error Path Tests" (lines 29-53) | martin-fowler-tests.md |

### 5. Kent Beck/TDD Violations

| ID | Defect | Location |
|----|--------|----------|
| TDD-001 | **Multiple assertions per test** - Test `test_runner_reuses_for_multiple_commands` (line 14) asserts both commands succeed, which is two distinct behaviors | martin-fowler-tests.md:14-17 |

---

## Summary

**Total Critical Defects: 11**

The test plan fails to meet the Testing Trophy standard (no real E2E execution), violates ATDD by lacking DSL separation, contains a logical contradiction in invariant I3 testing, and has formatting corruption that would prevent execution.

**STATUS: REJECTED**
