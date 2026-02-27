# Contract Specification

## Context
- Feature: Add cleanup function to global keyboard hook's `use_effect`
- File: `diagram_tool/src/hooks/keyboard.rs`
- Domain terms:
  - `use_effect`: Dioxus reactive effect that runs on dependency changes
  - `document::eval`: Executes JavaScript in browser context, returns `EvalResult`
  - Event listener leak: When `use_effect` re-runs without cleanup, duplicate listeners accumulate
- Assumptions:
  - The hook runs in a WASM/browser environment (not native)
  - Multiple components may call `use_global_keyboard()` but only one should be active at a time
  - The cleanup pattern from `canvas.rs` is the canonical reference
- Open questions: None

## Preconditions

| ID | Precondition | Enforcement Level | Type / Pattern |
|---|---|---|---|
| P1 | `Signal<DiagramDocument>` exists in context | Compile-time | `use_context::<Signal<DiagramDocument>>()` panics if missing |
| P2 | `Signal<History>` exists in context | Compile-time | `use_context::<Signal<History>>()` panics if missing |
| P3 | JavaScript runtime available (browser environment) | Runtime-checked | `document::eval` returns `EvalResult`; recv fails gracefully |

## Postconditions

| ID | Postcondition | Enforcement Level |
|---|---|---|
| Q1 | Exactly one `keydown` listener is registered after effect runs | Debug-only (cannot verify from Rust) |
| Q2 | `window.__seshat_global_keyboard_cleanup` function exists after effect runs | Debug-only |
| Q3 | Calling `window.__seshat_global_keyboard_cleanup()` removes the `keydown` listener | Debug-only |
| Q4 | Previous listener (if any) is removed before new one is added | Debug-only |
| Q5 | `eval.recv()` channel remains open for the lifetime of the effect | Compile-time (owned by spawn) |

## Invariants

| ID | Invariant | Scope |
|---|---|---|
| I1 | No more than one global keyboard listener is active at any time | Global (window) |
| I2 | Listener only handles events when no input/textarea/contenteditable is focused | JavaScript-side |
| I3 | Cleanup function is idempotent (safe to call multiple times) | JavaScript-side |

## Error Taxonomy

No explicit `Result` type returned. The hook is fire-and-forget with implicit error handling:

| Error Mode | Behavior | Recovery |
|---|---|---|
| Missing context | Panic at `use_context` | None (programmer error) |
| JavaScript eval failure | Silent (returns `EvalResult` but we don't check) | N/A |
| Channel closed | `eval.recv()` returns error, loop exits | Spawned task terminates |

## Contract Signatures

```rust
pub fn use_global_keyboard()
// Returns: () (no value, side-effect only)
// Dependencies: Reads from Signal<DiagramDocument>, Signal<History>
// Mutates: window.__seshat_global_keyboard_cleanup (JavaScript global)
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| P1: Context exists | Compile-time (strongest) | `use_context::<Signal<T>>()` panics if not found |
| P2: Context exists | Compile-time (strongest) | `use_context::<Signal<T>>()` panics if not found |
| P3: JS runtime | Error variant (weakest) | `EvalResult::recv()` returns `Err` if channel fails |

## Violation Examples (REQUIRED -- one per precondition and postcondition)

### Precondition Violations

- VIOLATES P1: Component calls `use_global_keyboard()` without `<DiagramDocument>` in context tree -- produces `panic!("use_context failed")` at runtime
- VIOLATES P2: Component calls `use_global_keyboard()` without `<History>` in context tree -- produces `panic!("use_context failed")` at runtime

### Postcondition Violations (detectable via test)

- VIOLATES Q1: Call `use_global_keyboard()`, trigger effect re-run, press Ctrl+Z -- undo fires **twice** (duplicate listener)
- VIOLATES Q2: After effect runs, `window.__seshat_global_keyboard_cleanup` is `undefined` -- cleanup cannot be called
- VIOLATES Q3: Call cleanup, press Ctrl+Z -- undo still fires (listener not removed)
- VIOLATES Q4: Effect re-runs without calling prior cleanup -- listeners accumulate (memory leak)

## Ownership Contracts (Rust-specific)

| Function | Ownership | Mutation Contract |
|---|---|---|
| `use_global_keyboard()` | Borrows `Signal<DiagramDocument>` via `use_context` | Read-only for event dispatch |
| `use_global_keyboard()` | Borrows `Signal<History>` via `use_context` | Read-only for event dispatch |
| `spawn(async move { ... })` | Takes ownership of `eval` via move | Consumes `eval.recv()` channel |

- Clone policy: No cloning. Signals are `Copy`, `eval` is moved into spawn.
- Lifetime: Effect lifetime tied to component; spawned task lifetime tied to eval channel.

## JavaScript Cleanup Pattern (Reference Implementation)

```javascript
// At start of eval script:
if (window.__seshat_global_keyboard_cleanup) {
    window.__seshat_global_keyboard_cleanup();
}

const onKeyDown = (e) => { /* ... */ };

window.addEventListener('keydown', onKeyDown);

window.__seshat_global_keyboard_cleanup = () => {
    window.removeEventListener('keydown', onKeyDown);
};
```

## Non-goals

- Does not handle native/desktop keyboard events (WASM only)
- Does not provide cleanup verification from Rust (JavaScript-side concern)
- Does not change the keyboard shortcuts themselves
