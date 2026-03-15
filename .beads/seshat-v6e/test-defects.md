# Test Plan Defects

### 1. Dan North BDD (Behavior vs. State) & Logical Contradiction
* **Defect**: The edge case specification `pasting_with_zero_offset_on_first_paste` directly contradicts the behavioral requirement in Q5, which states pasted nodes must be offset *"to visually distinguish them from the originals."*
* **Why it violates**: BDD focuses on user-facing behavior. If the first paste applies a zero offset, the pasted node will render exactly on top of the original node, providing zero visual feedback to the user and violating the explicit intent of Q5. The paste serial should intuitively provide an immediate offset (e.g., `(serial + 1) * constant`).

### 2. Combinatorial Permutations (Testing Trophy Exhaustiveness)
* **Defect**: The error path permutations are incomplete. The plan explicitly tests `pasting_edge_fails_when_target_node_is_in_document_but_not_in_clipboard`, but completely fails to specify the unhappy paths for when the `source` node is missing, or when *both* nodes are missing. 
* **Defect**: Missing invariants for internal clipboard corruption. There are no tests specifying what happens if the incoming `ClipboardData` contains duplicate Node IDs or Edge IDs *within itself* prior to insertion, which is a critical edge case when pasting serialized data from an OS clipboard.

### 3. Dave Farley ATDD & Functional-Rust Purity
* **Defect**: The contract signature `fn paste_contents(clipboard: &ClipboardData, doc: &mut DiagramDocument)` and its ownership rules enforce in-place mutation. 
* **Why it violates**: This violates ATDD's focus on pure, side-effect-free domain logic and explicitly breaks the project's `functional-rust` doctrine (Data → Calc → Actions, zero `mut` by default). A strictly testable domain calculation should take a shared reference `&DiagramDocument` and return a new state or a precise delta (`Result<PasteResult, Error>`), allowing tests to assert on the returned data without mocking or tracking mutations.

### 4. Advanced Paradigms (Missing Fuzzing & Mutation Testing)
* **Defect**: While the plan includes `proptest` for invariants, it completely omits **Mutation Testing** (vital for complex topological algorithms to ensure test suites catch logic inversions) and **Fuzzing**.
* **Why it violates**: Because `ClipboardData` crosses an external boundary, the test plan must include fuzzing specifications to bombard the parser with adversarial payloads (e.g., astronomical node counts to trigger OOM, infinitely recursive parent chains, or self-referential edge loops) to guarantee the `zero-panic` functional invariant holds under hostile conditions.