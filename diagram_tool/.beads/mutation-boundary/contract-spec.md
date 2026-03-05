# Contract Specification: Centralized Mutation Boundary

## Context
- **Feature**: Enforce mutation invariants centrally - route all document write paths through a single mutation boundary that always applies schema+semantic validation
- **Domain terms**: 
  - `DiagramDocument` - the main document model
  - `Mutation` - transformation that produces a new document
  - Schema validation - structural validity (types, required fields, ranges)
  - Semantic validation - graph integrity (valid node references, DAG constraints)
  - `with_mut` - Dioxus signal's internal mutation API (bypasses validation)
- **Assumptions**:
  - The `run_mutation` pipeline already exists and works correctly
  - `ui_helpers.rs` provides `mutate_doc_signal` and related functions
  - The codebase uses Dioxus signals for state management

## Preconditions
- [P1] Any document mutation MUST go through `run_mutation` or its helpers
- [P2] All mutation functions MUST return `Result<T, MutationError>` 
- [P3] The mutation boundary MUST apply BOTH schema AND semantic validation
- [P4] No direct `with_mut` calls on document signals in production code paths

## Postconditions
- [Q1] All document mutations pass through validated mutation boundary
- [Q2] Invalid mutations are rejected with descriptive errors (not silent failures)
- [Q3] Revision is incremented on successful mutations (unless preserve policy)
- [Q4] History is correctly updated for undo/redo support

## Invariants
- [I1] Document must always be valid after any mutation completes
- [I2] No partial state - mutations are atomic (all-or-nothing)
- [I3] Revision monotonically increases (never decreases)

## Error Taxonomy
- `MutationError::Schema(String)` - structural validation failed (version, field types, ranges)
- `MutationError::Semantic(String)` - semantic validation failed (dangling refs, cycles, conflicts)
- `UiMutationError::Mutation(MutationError)` - UI wrapper for mutation errors

## Contract Signatures
```rust
// Core mutation pipeline (already exists)
pub fn run_mutation<F>(current: &DiagramDocument, transform: F) -> Result<DiagramDocument, MutationError>
where F: FnOnce(&DiagramDocument) -> Result<DiagramDocument, MutationError>

// UI helpers (already exist)
pub fn mutate_doc_signal<F>(doc_signal: &mut Signal<DiagramDocument>, transform: F) -> UiMutationResult<()>
pub fn mutate_doc_with_history(...) -> UiMutationResult<()>
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Mutations go through run_mutation | Compile-time | Private signal field / lint rule |
| P2: Returns Result | Compile-time | Return type `Result<T, E>` |
| P3: Schema + semantic validation | Compile-time | `run_mutation` always calls both |
| P4: No direct with_mut | Runtime | Deprecation warning + code review |

## Violation Examples
- VIOLATES P1: `doc_signal.with_mut(|doc| { doc.x = new_val })` -- bypasses validation, should use `mutate_doc_signal`
- VIOLATES Q1: `doc_signal.write()` directly assigns invalid state -- should fail validation first
- VIOLATES I1: Mutation leaves dangling edge reference -- should be caught by semantic validation

## Ownership Contracts
- `doc_signal: &mut Signal<DiagramDocument>` - exclusive borrow for mutation
- `transform: FnOnce(DiagramDocument) -> MutationResult<DiagramDocument>` - takes ownership, returns new document (immutable transform)
- The function does NOT mutate in place - it creates a new document (persistent data structures)

## Non-goals
- [ ] Remove all uses of `with_mut` in test code (tests may need direct access)
- [ ] Change the document model to use Rust types that enforce invariants at compile time (larger refactor)
- [ ] Add transactional semantics (out of scope for this task)
