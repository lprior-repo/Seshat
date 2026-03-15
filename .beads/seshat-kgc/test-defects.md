# Test Plan Defects

**Status:** 🔴 **FLAWED**

While the contract and test plan demonstrate a strong understanding of Testing Trophy principles (real execution, fuzzing), they contain critical defects regarding determinism, implementation coupling, and BDD semantics. 

Here is the detailed breakdown of the flaws according to the requested doctrines:

### 1. Kent Beck (TDD): Determinism Violations
- **Flaw - Non-Deterministic Performance Assertions:** Asserting that the "operation completes within the 16ms performance budget" (`should_scale_selection_to_3000_nodes_within_time_limit` and Scenario 1) is a severe TDD violation. Wall-clock time assertions in automated test suites are highly hardware-dependent and will cause flaky tests on shared CI runners. Performance budgets should be enforced via dedicated micro-benchmarking tools (e.g., `criterion`), not pass/fail unit/integration tests.
- **Flaw - Uncontrolled Randomness:** `Scenario 1` specifies "3000 randomly distributed nodes." While randomness is correct for the Property-Based Test (where the fuzzer framework handles seed tracking and shrinking), using uncontrolled randomness in a standard Given-When-Then scenario violates the TDD mandate for predictable, deterministic execution. Standard scenarios require fixed, intentional setups.

### 2. Dave Farley (ATDD): Separation of WHAT from HOW
- **Flaw - Tight Coupling to Internal State:** The test `should_verify_postcondition_only_selection_field_is_mutated` explicitly asserts against internal struct mechanics ("only the selection field is mutated"). ATDD strictly demands testing observable behavior through a DSL (the WHAT), rather than inspecting internal data structures or field-level mutations (the HOW). If you want to ensure the rest of the document is unaltered, you should assert that the observable document state (aside from selection) remains equivalent to the start state using the DSL.

### 3. Dan North (BDD): Behavior vs. Mechanics
- **Flaw - Mechanics-Infested Naming:** Test names like `test_precondition_negative_dimensions_returns_error` and `should_verify_postcondition_contain_mode_strictness` leak contract jargon and implementation mechanics into the test suite. BDD demands expressive, domain-focused naming (e.g., `should_reject_selection_when_dimensions_are_negative` or `should_strictly_exclude_partially_intersecting_nodes_in_contain_mode`).
- **Flaw - State-Centric Language:** Asserting that "The selection state is durably updated" focuses on internal state. BDD requires focusing on behavior—for instance, "The diagram correctly reports the enclosed nodes as selected."

### 4. Combinatorial Permutations
- **Flaw - Flat Permutations Instead of Cross-Products:** The "Edge Case & Combinatorial Tests" section lists permutations linearly (e.g., node inside, node outside, node on edge, node rotated 90 degrees). True combinatorial rigor requires testing the cross-product of these states. For example, the plan is missing tests that explicitly verify an **arbitrarily rotated node** that falls **exactly on a boundary corner** while in **Contain mode**. 

## Required Remediation

To elevate this test plan to flawless status, the following corrections must be made:
1. **Remove CI-bound performance assertions:** Move the 16ms budget to a benchmarking suite. Replace the test with a deterministic stress test that simply ensures 3000 nodes execute correctly without crashing or timing out excessively.
2. **Fix Determinism:** Replace "randomly distributed nodes" in Scenario 1 with a deterministically generated layout (e.g., a predictable grid or mathematically fixed scatter).
3. **Decouple from State:** Remove tests specifically checking internal struct fields. Validate state changes strictly through observable properties in the ATDD DSL.
4. **Rewrite Test Names:** Strip out all mechanical words (`precondition`, `postcondition`, `test_`) and replace them with pure domain behavior language.
5. **Expand the Combinatorial Matrix:** Explicitly mandate parameterized tests that multiply `[Rotation] x [Boundary Intersection Position] x [Selection Mode]`.