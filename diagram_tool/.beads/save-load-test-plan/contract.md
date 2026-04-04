# Contract: File Save/Load Persistence Feature

**Bead**: save-load-test-plan  
**Date**: 2026-04-03  
**Status**: authoritative source for public function signatures

---

## Public Function Inventory

All public functions in scope for this feature, with exact signatures and source locations.

| # | File | Function Signature | Notes |
|---|------|-------------------|-------|
| 1 | `save.rs:32` (non-WASM) | `pub fn apply_save_document(doc: &DiagramDocument, session: &DocumentSession, file_path: &PathBuf) -> Result<DocumentSession, SaveError>` | Pure calc layer — file I/O via `save_workspace_atomic` |
| 2 | `save.rs:54` (WASM) | `pub fn apply_save_document(doc: &DiagramDocument, session: &DocumentSession, file_path: &PathBuf) -> Result<DocumentSession, SaveError>` | Always returns `Err(SaveError::Io("Save not available in WASM"))` |
| 3 | `save.rs:62` | `pub fn save_workspace(doc_signal: Signal<DiagramDocument>, session_signal: Signal<DocumentSession>, toasts: Signal<ToastQueue>)` | Action — async, spawns task |
| 4 | `open.rs:56` | `pub fn apply_open_document(current_doc: &DiagramDocument, current_history: &History, contents: &str, file_path: PathBuf) -> Result<(DiagramDocument, History, DocumentSession), OpenError>` | Pure calc layer |
| 5 | `open.rs:296` | `pub fn open_workspace(signals: WorkspaceSignals, toasts: Signal<ToastQueue>, store_bridge: Option<Arc<StoreBridge>>)` (native) / same signature (WASM) | Action — async, spawns task |
| 6 | `common.rs:14` | `pub fn prepare_import_transition(current: &DiagramDocument, contents: &str) -> Result<(DiagramDocument, History), ImportTransitionError>` | Pure calc layer — migration |
| 7 | `common.rs:39` | `pub fn apply_import_contents(doc: &mut DiagramDocument, history: &mut History, contents: &str) -> Result<(), ImportTransitionError>` | Pure calc layer — atomic |
| 8 | `common.rs:55` | `pub fn update_load_save_success(toast_handle: ToastHandle, title: &str, detail: String)` | Side-effect only — updates toast |
| 9 | `common.rs:64` | `pub fn update_load_save_error(toast_handle: ToastHandle, title: &str, detail: String)` | Side-effect only — updates toast |
| 10 | `hooks/keyboard.rs:21` (non-WASM) / `:181` (WASM) | `pub fn use_global_keyboard(db_tx: Option<Coroutine<EventEnvelope>>)` | Keyboard hook — handles Ctrl+S, Ctrl+O |

---

## Error Enum Variants

### SaveError (`save.rs:15-19`)
```rust
pub enum SaveError {
    NoFilePath,
    Serialize(String),
    Io(String),
}
```

### OpenError (`open.rs:22-26`)
```rust
pub enum OpenError {
    Parse(String),
    Validation(String),
    Io(String),
}
```

### ImportTransitionError (`common.rs:9-12`)
```rust
pub enum ImportTransitionError {
    Parse(String),
    Validation(String),
}
```

### CliPersistenceError (`cli_persistence/mod.rs:30-51`)
```rust
pub enum CliPersistenceError {
    IoError(#[from] std::io::Error),
    ParseError(#[from] serde_json::Error),
    ValidationError(String),
    TempFileError(String),
    AtomicRenameError { from: String, to: String },
    NoValidDocument(String),
    PathTraversalDenied { path: String },
}
```

---

## WASM vs Native Differences

| Function | Native | WASM |
|----------|--------|------|
| `apply_save_document` | Real file I/O | Always `Err(SaveError::Io("Save not available in WASM"))` |
| `save_workspace` | Uses `rfd::FileDialog` | Shows "Save not available" toast |
| `open_workspace` | Uses `rfd::FileDialog` + store_bridge | Uses browser file picker or `__SESHAT_E2E_IMPORT_JSON` global |
| `use_global_keyboard` | Handles Ctrl+S + Ctrl+O | Handles Ctrl+S + Ctrl+O |

---

## Error Mapping (CliPersistenceError → SaveError)

The `apply_save_document()` function calls `save_workspace_atomic()` which returns `CliPersistenceError`. These errors are mapped to `SaveError` variants:

| CliPersistenceError | Maps To | Rationale |
|---------------------|---------|-----------|
| `IoError(_)` | `SaveError::Io(_)` | OS-level I/O failures (permission denied, disk full, etc.) |
| `ParseError(_)` | `SaveError::Serialize(_)` | JSON serialization failures |
| `ValidationError(_)` | `SaveError::Serialize(_)` | Document validation failures |
| `TempFileError(_)` | `SaveError::Io(_)` | Temp file creation failures are I/O at their core |
| `AtomicRenameError{..}` | `SaveError::Io(_)` | Atomic rename failures are I/O at their core |
| `NoValidDocument(_)` | Not reachable | Only used during load, not save |
| `PathTraversalDenied{..}` | `SaveError::Io(_)` | Path traversal is rejected before I/O |

---

*This contract is the authoritative source for public function signatures. Any discrepancy between this and implementation is a bug.*
