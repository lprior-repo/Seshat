# Contract Specification

## Context
- **Feature**: UI Conflict Auto-dismiss
- **Bead ID**: seshat-fsa
- **Description**: Wire the Toast component to automatically dismiss and clear the ai_conflict_state after 3 seconds.
- **Domain terms**:
  - `ai_conflict_state`: A Dioxus Signal<Option<AiConflictInfo>> that holds transient AI conflict information
  - `Toast`: UI notification component with intent (Info, Success, Warning, Error)
  - `ToastApi`: API for creating/managing toasts
  - `Toaster`: Component that renders and manages toast lifecycle
  - Auto-dismiss: Automatic toast dismissal triggered by timer (3 seconds)
- **Assumptions**:
  - The Toast system is already initialized (ToastQueue provided via context)
  - Dioxus signals work as expected in the WASM environment
  - JavaScript setTimeout is available for delayed execution
- **Open Questions**:
  - Should the 3-second timer start when the toast is created or when it first renders?
  - Is there a need to support cancellation of the auto-dismiss timer?
  - What exact data should be stored in ai_conflict_state?

## Preconditions
- **P1**: ToastQueue signal must be available in the Dioxus context
- **P2**: ai_conflict_state signal must be initialized (can be None initially)
- **P3**: The conflict toast must be of ToastIntent::Error or ToastIntent::Warning type
- **P4**: The timer system (document::eval with setTimeout) must be functional

## Postconditions
- **Q1**: After exactly 3 seconds (3000ms) from toast creation, the conflict toast is automatically dismissed (dismissed flag set to true)
- **Q2**: After dismissal completes, ai_conflict_state is set to None (cleared)
- **Q3**: The ToastQueue remains in a valid state after auto-dismiss (no items corrupted)
- **Q4**: If user manually dismisses before 3 seconds, the timer is cancelled and ai_conflict_state is still cleared

## Invariants
- **I1**: ToastQueue.items always contains valid Toast structs (no corrupted entries)
- **I2**: At most one conflict toast exists at any given time
- **I3**: The auto-dismiss timer does not cause memory leaks (timers are properly cleaned up)
- **I4**: ai_conflict_state is Some(_) only when a conflict toast is visible

## Error Taxonomy
- **Error::JsTimeoutFailure**: The JavaScript setTimeout call failed or returned an error
- **Error::SignalNotFound**: The required Dioxus signal (ToastQueue or ai_conflict_state) is not available in context
- **Error::ToastNotFound**: Attempted to dismiss a toast that no longer exists in the queue
- **Error::TimerCancelled**: The auto-dismiss timer was cancelled (e.g., manual dismiss before 3s)

## Contract Signatures

### New Function: show_conflict_toast
```rust
/// Shows a conflict toast and starts the 3-second auto-dismiss timer.
/// 
/// # Preconditions
/// - P1: ToastQueue signal must be available
/// - P2: ai_conflict_state signal must be available
/// 
/// # Postconditions
/// - Q1: Toast is created with the given options
/// - Q2: ai_conflict_state is set to Some(conflict_info)
/// - Q3: A 3-second timer is started for auto-dismiss
/// 
/// # Returns
/// - Ok(ToastId): The ID of the created toast
/// - Err(Error::SignalNotFound): If required signals are not available
/// - Err(Error::JsTimeoutFailure): If timer creation fails
pub fn show_conflict_toast(
    api: ToastApi, 
    conflict_info: AiConflictInfo
) -> Result<ToastId, Error>
```

### New Function: clear_ai_conflict_state
```rust
/// Clears the ai_conflict_state signal.
/// 
/// # Postconditions
/// - ai_conflict_state is set to None
pub fn clear_ai_conflict_state(state: &mut Signal<Option<AiConflictInfo>>)
```

### Modified: Toaster component
```rust
/// Modified Toaster component to include auto-dismiss logic for conflict toasts.
/// 
/// # Preconditions
/// - P1, P2, P4
/// 
/// # Postconditions
/// - Q1, Q2, Q3, Q4
#[component]
pub fn Toaster() -> Element
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| ToastQueue available | Runtime-checked | `use_context::<Signal<ToastQueue>>()` returns Some |
| ai_conflict_state available | Runtime-checked | `use_context::<Signal<Option<AiConflictInfo>>>()` returns Some |
| Timer functional | Runtime-checked | `document::eval()` returns Ok |
| Conflict toast valid | Compile-time | ToastIntent enum is non-exhaustive but validated at creation |

## Violation Examples (REQUIRED)

### Precondition Violations

- **VIOLATES P1** (ToastQueue not available):
  ```rust
  // When ToastApi is created without available ToastQueue context
  let api = ToastApi::from_signal(None); // or use outside of provider
  api.show(ToastIntent::Error, "Conflict", Some("AI edit rejected"));
  // Should produce: Err(Error::SignalNotFound)
  ```

- **VIOLATES P2** (ai_conflict_state not available):
  ```rust
  // When trying to set conflict state without signal availability
  let mut conflict_signal = None; // simulate unavailable signal
  conflict_signal.write().replace(AiConflictInfo { ... });
  // Should produce: Err(Error::SignalNotFound)
  ```

- **VIOLATES P3** (Non-conflict toast intent):
  ```rust
  // When creating a non-conflict toast (Info, Success)
  let options = ToastOptions::new(ToastIntent::Success, "All good");
  // Auto-dismiss should NOT apply to non-conflict toasts
  // The timer should NOT be started for these intents
  ```

### Postcondition Violations

- **VIOLATES Q1** (Toast not dismissed after 3 seconds):
  ```rust
  // After 3 seconds + 100ms, check if toast is dismissed
  let toast_id = show_conflict_toast(...);
  std::thread::sleep(std::time::Duration::from_millis(3100));
  let queue = toast_api.queue.read();
  let toast = queue.items().iter().find(|t| t.id == toast_id);
  // Should produce: toast.dismissed == true (VIOLATION if false)
  ```

- **VIOLATES Q2** (ai_conflict_state not cleared):
  ```rust
  // After auto-dismiss, check if state is cleared
  let state = use_context::<Signal<Option<AiConflictInfo>>>();
  // Should produce: state.read().is_none() (VIOLATION if Some)
  ```

- **VIOLATES Q4** (Manual dismiss doesn't clear state):
  ```rust
  // When user manually dismisses before 3 seconds
  toast_handle.dismiss();
  let state = use_context::<Signal<Option<AiConflictInfo>>>();
  // Should produce: state.read().is_none() (VIOLATION if Some)
  ```

## Ownership Contracts (Rust-specific)

### AiConflictInfo
```rust
#[derive(Clone, Debug)]
pub struct AiConflictInfo {
    pub message: String,
    pub timestamp: i64,
    pub conflicting_element: Option<String>,
}
```
- **Clone policy**: Intentionally clonable - Signal<Option<T>> requires T to be Clone for read operations
- **Ownership**: Created at the call site, ownership transferred to the signal

### ToastApi
- **Ownership**: Copy (derive(Clone, Copy)) - lightweight handle to the Signal
- **Mutation**: Uses `with_mut` on internal Signal, no direct mutation of self

### Timer Handle
- **Ownership**: Dropped after spawn - the async closure captures needed signals
- **Lifetime**: Must complete before component unmounts to avoid dangling references

## Non-goals
- Persisting ai_conflict_state across page refreshes
- Supporting custom dismiss durations (hardcoded to 3 seconds per spec)
- Adding conflict toast to desktop CLI (WASM-only feature)
- Implementing conflict resolution UI (accept/reject buttons) - separate bead
- Adding sound notifications for conflict toasts
