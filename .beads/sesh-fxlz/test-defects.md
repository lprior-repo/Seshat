# Test Defects

STATUS: REJECTED

The test plan and contract specification violate core principles of Testing Trophy, Dan North's BDD, and Dave Farley's ATDD doctrines:

1. **Dave Farley ATDD / Clean Architecture**: Contradiction in domain purity. The contract describes `apply_edge_label_edit` as a "pure domain function", yet includes `UpdateFailed` for "persistence or other underlying operations fail" in its Error Taxonomy and Scenarios. Pure domain functions must not know about or handle I/O and persistence.

2. **Contract Signature Validity (Data→Calc→Actions)**: The signature `fn apply_edge_label_edit(edge_id: EdgeId, new_label: String) -> Result<(), EditError>` has no context or target (e.g., `&mut Document` or `self`). It is impossible for a standalone function with this signature to perform "state mutation of the edge label" in Rust without global state, which violates functional pure domain principles.

3. **Dan North BDD (Infrastructure Detail)**: Scenario 4 exposes infrastructure details ("The underlying storage system is in a failed state"). BDD scenarios must use business-readable Ubiquitous Language, abstracting away technical implementation details like specific storage systems.

4. **Testing Trophy / Dave Farley ATDD (Encapsulation)**: The integration tests state they drive the UI but verify "the actual underlying domain document state". Testing Trophy and ATDD emphasize testing behaviors from the outside-in (e.g., verifying the UI reflects the change). Reaching directly into internal domain state pierces encapsulation and creates brittle tests.

5. **Domain vs. UI State Mismatch**: Scenario 2 tests "canceling an edit". The contract explicitly states UI state and edit sessions are non-goals and unknown to the domain. Including a UI-only non-event in a domain contract test plan dilutes the behavioral focus. Acceptance tests should be independent of UI presentation layer mechanisms like "drafting" and "canceling" if testing the domain contract.