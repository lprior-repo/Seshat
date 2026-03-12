# Contract Specification

## Context
- Feature: UI Visualization: Conflict Toast
- Bead: seshat-8tb
- Domain terms:
  - `AiConflictState` - struct holding conflict reason and conflicting entities
  - `show_conflict_toast` - function to display a toast for AI conflict
  - `ai_conflict_state` - Dioxus signal tracking current conflict
  - `ToastApi` - API for creating/managing toasts
- Assumptions:
  - The toast system (ToastQueue, ToastApi, Toaster component) is already working
  - The poller in app.rs already detects dropped AI events
  - The `AiConflictState` struct is already defined in toast.rs
- Open questions:
  - Should the conflict state include specific entity names for better UX?

## Preconditions
- [P1] ToastApi must be available from context to display the toast
- [P2] ai_conflict_state signal must be initialized in app.rs context
- [P3] Conflict detection must have identified dropped AI operations

## Postconditions
- [Q1] When conflict detected, toast with title "Edit Conflict" is displayed
- [Q2] The toast auto-dismisses after 3 seconds (CONFLICT_TOAST_DISMISS_MS)
- [Q3] ai_conflict_state is cleared when toast is dismissed
- [Q4] User can manually dismiss the toast, which also clears the conflict state

## Invariants
- [I1] Only one conflict toast can be displayed at a time
- [I2] ai_conflict_state is Some when conflict exists, None otherwise

## Error Taxonomy
- Error::NoConflictState - when no conflict state provided to show_conflict_toast
- Error::QueueFull - when toast queue is at capacity
- Error::InvalidReason - when conflict reason is empty or missing
- Error::SignalNotFound - when Dioxus signal not available in context

## Contract Signatures
- `pub fn show_conflict_toast(conflict_state: &AiConflictState, toast_api: ToastApi) -> Result<ToastHandle, Error>`
- `pub fn should_show_conflict_toast(conflict_state: Option<&AiConflictState>) -> Result<bool, Error>`
- `pub fn clear_ai_conflict_state(state: &mut Signal<Option<AiConflictState>>)`

## Type Encoding
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| ToastApi available | Runtime-checked constructor | use_context::<Signal<ToastQueue>>() |
| ai_conflict_state initialized | Runtime | context_provider in app.rs |
| Conflict detected | Runtime | DropDetectionResult.has_conflict |

## Violation Examples
- VIOLATES P1: Calling show_conflict_toast without ToastApi in context -> Error::SignalNotFound
- VIOLATES P3: Calling show_conflict_toast with empty AiConflictState -> Error::NoConflictState

## Ownership Contracts
- `show_conflict_toast` takes immutable reference to AiConflictState (read-only)
- `ToastApi` is copied/cloned into the function (shared access)
- `clear_ai_conflict_state` takes mutable reference to signal (interior mutability pattern)

## Non-goals
- Changing the conflict detection algorithm in ai_event_detection.rs
- Modifying the ToastQueue internal implementation
- Adding persistent storage for conflicts
