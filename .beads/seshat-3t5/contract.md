# Contract Specification

## Context

- **Feature**: Toast/Badge UI Component for AI Conflict Notifications
- **Bead**: seshat-3t5
- **Domain Terms**:
  - `ai_conflict_state` - State representing a conflict between AI and human edits
  - `Toast` - UI component for displaying transient notifications
  - `ToastIntent` - Classification of toast message (Info, Success, Warning, Error)
  - `ConflictDecision::Reject` - Decision indicating AI operation was rejected due to human edit conflict
- **Assumptions**:
  - The Toast system is already implemented in `diagram_tool/src/ui/toast.rs`
  - The conflict resolution logic exists in `diagram_tool/src/models/conflict/`
  - This bead creates a Toast UI component that renders when AI conflict rejection occurs
- **Open Questions**:
  - What exact message text should display? (Will use "AI operation rejected - human has active edit")
  - Should the toast show conflicting entity names? (Yes, include in detail)

## EARS Requirements

| ID | Requirement | Type |
|----|-------------|------|
| EARS-1 | System shall notify user of concurrent editing conflicts | Ubiquitous |
| EARS-2 | Rejected AI event triggers Toast | Event-driven |
| EARS-3 | No toast after 3 seconds | Unwanted |

## Preconditions

- [P1] `conflict_state.is_some()` - Must have valid conflict state to display
- [P2] `toast_queue.is_available()` - Toast queue must accept new toasts (max 1)
- [P3] `conflict_state.reason.is_some()` - Rejection reason must be present

## Postconditions

- [Q1] `toast.id` is assigned and unique - New toast has valid ID
- [Q2] `toast.intent == ToastIntent::Warning` - Conflict toasts use Warning intent (yellow)
- [Q3] `toast.title == "Edit Conflict"` - Standard title for conflict notifications
- [Q4] `toast.detail.contains(conflict_state.reason)` - Detail includes rejection reason
- [Q5] `toast.auto_dismiss == true` - Toast auto-dismisses after 3 seconds
- [Q6] `toast_queue.items.len() <= MAX_TOASTS` - Queue respects max toast limit (1)

## Invariants

- [I1] Toast queue maintains at most 1 visible toast (enforced in code)
- [I2] Toast IDs are unique and monotonically increasing
- [I3] Dismissed toasts are marked but may remain in queue until removal

## Error Taxonomy

| Error Variant | Condition | Recovery |
|--------------|-----------|----------|
| `Error::NoConflictState` | Called with None conflict state | Return early, no toast |
| `Error::QueueFull` | Queue already has MAX_TOASTS toasts | Remove oldest dismissed first |
| `Error::InvalidReason` | Conflict reason is empty/missing | Show generic "Edit conflict" message |

## Contract Signatures

```rust
/// Display toast for AI conflict state
/// Returns: Result<ToastHandle, Error>
fn show_conflict_toast(
    conflict_state: &AiConflictState,
    toast_api: ToastApi,
) -> Result<ToastHandle, Error>;

/// Check if toast should be displayed for conflict
/// Returns: Result<bool, Error>
fn should_show_conflict_toast(
    conflict_state: Option<&AiConflictState>,
) -> Result<bool, Error>;
```

## Type Encoding

| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| conflict_state.is_some() | Runtime-checked | `Option<&AiConflictState>` - match on Some |
| toast_queue.available | Runtime-checked | `MAX_TOASTS = 1` check before add |
| reason.non_empty | Runtime-checked | `if reason.is_empty() { "Edit conflict" }` |

## Violation Examples

- VIOLATES P1: `show_conflict_toast(None, api)` -> returns `Err(Error::NoConflictState)`
- VIOLATES P2: `show_conflict_toast(state, full_queue)` -> returns `Err(Error::QueueFull)`
- VIOLATES P3: `show_conflict_toast(empty_reason_state, api)` -> returns `Err(Error::InvalidReason)`
- VIOLATES Q1: After call, `toast.id` must be valid non-zero ID
- VIOLATES Q5: After 3 seconds, `toast.dismissed == true`

## Ownership Contracts

- `toast_api: ToastApi` - Shared borrow via Signal, no ownership transfer
- `conflict_state: &AiConflictState` - Read-only borrow, no mutation
- Return `ToastHandle` - Caller gains ownership of handle for dismiss control

## Non-goals

- [ ] Persisting conflict history
- [ ] Conflict resolution UI (only notification)
- [ ] Multiple simultaneous conflict toasts (max 1)
- [ ] Customizable auto-dismiss duration (hardcoded 3s)
