# Contract Specification: ai_conflict_state Signal

## Context
- **Feature**: Add an `ai_conflict_state` Signal<Option<String>> to the Dioxus app context to track AI conflict messages
- **Bead ID**: seshat-6jd
- **Domain terms**:
  - `ai_conflict_state`: A reactive signal that holds an optional conflict message string
  - Conflict message: Human-readable message describing why an AI operation was rejected due to concurrent human editing
  - Dioxus context: Application-level state provider using `use_context_provider`
- **Assumptions**:
  - This is a UI state signal, not persisted to backend
  - Multiple components may read from this signal
  - The signal is updated when an AI event is rejected due to human-AI conflict
- **Open questions**: None

## EARS Requirements
- **Ubiquitous**: System shall notify user of concurrent editing conflicts
- **Event-driven**: Rejected AI event triggers signal update
- **Unwanted**: No stale conflict messages persist after resolution

## Preconditions
- [P1] Signal initialization: The signal must be initialized with `None` (no conflict) at app startup
- [P2] Signal is accessible: Components must retrieve via `use_context::<Signal<Option<String>>>()`
- [P3] Valid message format: When set, message must be non-empty string

## Postconditions
- [Q1] Signal state after rejection: After AI event rejection, signal contains the rejection message
- [Q2] Signal state after resolution: After human releases edit, signal returns to `None`
- [Q3] Signal mutation contract: Only the conflict message value changes; no other app state is affected

## Invariants
- [I1] At any point, `ai_conflict_state` is either `None` or `Some(non_empty_string)`
- [I2] Signal is always accessible from any component within the app context
- [I3] No stale messages: Signal is cleared when conflict is resolved

## Error Taxonomy
- **ConflictError::SignalNotFound**: Returned when context provider is not available (should never happen in correct app setup)
- **ConflictError::InvalidMessage**: Returned when attempting to set an empty message (precondition violation)
- **ConflictError::ContextNotInitialized**: Returned when reading signal before provider initialization

## Contract Signatures
```rust
// Provider initialization (in app.rs)
fn init_ai_conflict_state() -> Signal<Option<String>> {
    use_context_provider(|| Signal::new(Option::<String>::None))
}

// Read conflict state (in components)
fn use_ai_conflict_state() -> ReadOnlySignal<Option<String>> {
    use_context::<Signal<Option<String>>>()
}

// Set conflict message
fn set_conflict_message(msg: &str) -> Result<(), ConflictError> {
    // P3: msg must be non-empty
}

// Clear conflict (resolve)
fn clear_conflict() -> Result<(), ConflictError> {
    // Q2: signal must become None
}
```

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Signal initialized to None | Compile-time | `Signal::new(Option::<String>::None)` |
| Context accessible | Compile-time | `use_context::<Signal<Option<String>>>()` |
| Message non-empty | Runtime-checked | `if msg.is_empty() { return Err(ConflictError::InvalidMessage) }` |
| Signal cleared on resolution | Runtime-checked | `signal.set(None)` with verification |

## Violation Examples
- VIOLATES P1: `use_context::<Signal<Option<String>>>()` called before `use_context_provider` initialization -- returns `Err(ConflictError::SignalNotFound)`
- VIOLATES P3: `set_conflict_message("")` -- should produce `Err(ConflictError::InvalidMessage)`
- VIOLATES Q2: After calling `clear_conflict()`, checking `signal.read()` returns `Some(_)` instead of `None` -- test fails

## Ownership Contracts
- **Signal ownership**: The signal is owned by the app context; components receive borrows
- **Clone policy**: Signals are cheaply cloneable (internal Arc); this is the intended pattern for Dioxus
- **Mutation**: Only the inner `Option<String>` value changes; no structural mutations

## Non-goals
- [ ] Backend persistence of conflict messages
- [ ] Conflict resolution logic (handled elsewhere)
- [ ] Multiple concurrent conflict tracking (single conflict at a time)

## Implementation Phases
1. **Phase 1**: Add `use_context_provider` for `Signal<Option<String>>` in app.rs
2. **Phase 2**: Export helper hook `use_ai_conflict_state()` for components
3. **Phase 3**: Integrate with AI event rejection handler to set message
4. **Phase 4**: Add UI component to display conflict message (optional display layer)
5. **Phase 5**: Clear signal when conflict is resolved

## Traceability
| EARS Requirement | Contract Clause |
|---|---|
| System shall notify user of concurrent editing conflicts | Q1: Signal contains rejection message |
| Rejected AI event triggers signal update | Q1: Postcondition on rejection handler |
| No stale messages persist after resolution | Q2: Invariant I3, Postcondition Q2 |
