# Test Review Defects: seshat-t6u

STATUS: REJECTED

The test plan defined in `.beads/seshat-t6u/martin-fowler-tests.md` and the contract in `.beads/seshat-t6u/contract.md` have been reviewed against Testing Trophy, BDD, and ATDD doctrines.

## High Severity
1. **Inconsistency between Contract and Test Plan**: `martin-fowler-tests.md` line 16 expects "most recent common ancestor" behavior for the new subgraph's parent, but `contract.md` (Q1-Q6) fails to specify this postcondition. Without this, the implementation could default to root, breaking the user's expected mental model.
2. **Missing Boundary Tests for Nesting Limit**: `martin-fowler-tests.md` tests the violation at depth 5 (line 68), but lacks a "boundary success" test for a node at depth 4 being grouped (resulting in a node at depth 5, the maximum allowed).

## Medium Severity
3. **Ambiguity in Locked Node Error**: `contract.md` P3 (line 16) and `martin-fowler-tests.md` (line 60) don't specify if the operation should return *all* locked nodes or just the first one found. For better UX, returning a list of IDs might be preferable, or the contract should explicitly state it's a "fail-fast on first" approach.
4. **Missing Atomicity Implementation Detail**: `contract.md` Q6 (line 26) mentions a "single transaction" but doesn't reference the project's single-log WAL architecture (defined in `docs/12_SINGLE_LOG_ARCHITECTURE.md`). Testing should explicitly verify that this operation is logged and reconcilable.

## Low Severity
5. **E2E Integration Gap**: In the context of the Testing Trophy, while integration tests cover the model, there is no plan for a "real execution" test through the editor interface (Dioxus), even as a smoke test, to ensure the UI selection state stays in sync with the model after the grouping operation.
