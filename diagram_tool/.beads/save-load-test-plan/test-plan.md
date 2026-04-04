# Test Plan: File Save/Load Persistence Feature

## Summary

- **Feature**: Workspace save/load via toolbar UI, keyboard shortcuts (Ctrl+S/Ctrl+O), and Export/Import buttons
- **Bead**: save-load-test-plan
- **Behaviors identified**: 46
- **Trophy allocation**: ~52 unit / ~16 integration / ~2 e2e / ~0 static
- **Proptest invariants**: 4
- **Fuzz targets**: 3
- **Kani harnesses**: 2
- **Mutation threshold**: ≥90%

---

## 0. Contract Reference

**`contract.md`** in this bead directory lists all 10 public functions with exact signatures, error enum variants, and WASM/native differences. All tests below refer to functions defined in that contract.

---

## 1. Behavior Inventory

### 1.1 Save Operations (`diagram_tool/src/ui/toolbar/persistence/save.rs`)

1. **[Save] save_workspace() shows info toast when saving with known path**
2. **[Save] save_workspace() shows Save As dialog when no file path set**
3. **[Save] save_workspace() updates session_signal with saved session on success**
4. **[Save] save_workspace() shows success toast with file path on success**
5. **[Save] save_workspace() shows error toast on IO failure**
6. **[Save] save_workspace() shows error toast on serialization failure**
7. **[Save] save_workspace() dismisses toast when user cancels Save As dialog**
8. **[Save] save_workspace() returns error toast in WASM build**
9. **[Save] apply_save_document() returns new session with cleared dirty flag on success**
10. **[Save] apply_save_document() preserves file path in session (not save path)**
11. **[Save] apply_save_document() syncs revision from saved document**
12. **[Save] apply_save_document() returns Io error for invalid path**
13. **[Save] apply_save_document() returns NoFilePath error when session has no path**
14. **[Save] apply_save_document() returns Serialize error when document validation fails**
15. **[Save] apply_save_document() returns PathTraversalDenied error when path contains ".."**
16. **[Save] apply_save_document() WASM variant always returns Io error**

### 1.2 Open Operations (`diagram_tool/src/ui/toolbar/persistence/open.rs`)

17. **[Open] open_workspace() shows info toast while loading**
18. **[Open] open_workspace() opens native file picker on native**
19. **[Open] open_workspace() opens browser file picker on WASM**
20. **[Open] open_workspace() uses E2E import JSON global when available (WASM)**
21. **[Open] open_workspace() resets revision to INITIAL on load**
22. **[Open] open_workspace() pushes current doc to history before loading**
23. **[Open] open_workspace() resets store bridge on native after successful load**
24. **[Open] open_workspace() shows error toast when file picker cancelled**
25. **[Open] open_workspace() shows error toast on load failure (native IO error)**
26. **[Open] open_workspace() shows error when both primary and LKG fail**
27. **[Open] open_workspace() uses LKG fallback when primary file corrupt**
28. **[Open] apply_open_document() creates session with correct file path**
29. **[Open] apply_open_document() happy path: returns doc with correct node and edge count**
30. **[Open] apply_open_document() returns Parse error for invalid JSON**
31. **[Open] apply_open_document() returns Validation error for schema violations**

### 1.3 Toast Helper Operations (`diagram_tool/src/ui/toolbar/persistence/common.rs`)

32. **[Toast] update_load_save_success() updates toast with Success intent and correct title/detail**
33. **[Toast] update_load_save_error() updates toast with Error intent and correct title/detail**

### 1.4 Import/Transition Operations (`common.rs`, `persistence_compat/`)

34. **[Import] apply_import_contents() updates doc and history atomically on success**
35. **[Import] apply_import_contents() leaves doc and history unchanged on parse error**
36. **[Import] apply_import_contents() leaves doc and history unchanged on validation error**
37. **[Import] prepare_import_transition() migrates legacy field names (camelCase to snake_case)**
38. **[Import] prepare_import_transition() migrates icon_data_url to icon_url for base64 data**
39. **[Import] prepare_import_transition() preserves existing icon_url when both exist**
40. **[Import] prepare_import_transition() normalizes arrow types from legacy values**
41. **[Import] prepare_import_transition() happy path: valid v2 JSON without migration needed**
42. **[Import] parse_diagram_document_with_compat() rejects documents without version field**

### 1.5 Keyboard Shortcuts (`hooks/keyboard.rs`)

43. **[Keyboard] Ctrl+S triggers save_workspace() when not editing input (native)**
44. **[Keyboard] Ctrl+S triggers save_workspace() when not editing input (WASM)**
45. **[Keyboard] Ctrl+O triggers open_workspace() when not editing input (native only)**

### 1.6 Toolbar Buttons (`toolbar.rs`)

46. **[Toolbar] Export button triggers save_workspace()**
47. **[Toolbar] Import button triggers open_workspace() (async-db feature, native only)**

---

## 2. Trophy Allocation

| Layer | Count | Rationale |
|-------|-------|-----------|
| **Unit** | 52 | Calc layer: 9 functions × 5+ tests each for boundary/exhaustive coverage. Functions: `apply_save_document` (6 variants), `apply_open_document` (4 variants), `apply_import_contents` (4 variants), `prepare_import_transition` (6 variants), `update_load_save_success` (1), `update_load_save_error` (1), error Display impls (3), revision sync logic (2), path traversal (1). Target 5x ratio. |
| **Integration** | 16 | Component interactions: `save_workspace()` with FileDialog, `open_workspace()` with FileDialog + store_bridge, LKG fallback chain, atomic write + load round-trip, keyboard hook → toolbar action, toast notifications |
| **E2E** | 2 | Full workflow: Ctrl+S saves and verify file exists, Ctrl+O opens and verify document loaded |
| **Static** | 0 | Covered by clippy/cargo-deny in CI |

**Rationale**: Save/load is I/O-heavy. The calc layer (parse, validate, transform) has pure functions that benefit from exhaustive unit tests. Integration tests cover component boundaries (FileDialog, store_bridge, signals). E2E is minimal because keyboard shortcuts and file dialogs require browser/native environment.

---

## 3. BDD Scenarios

### 3.1 Save Scenarios

---

#### Behavior: save_workspace() shows info toast when saving with known path

**Given**: AppState with document signal and session signal where session has a `file_path` set to `"/path/to/diagram.json"`

**When**: `save_workspace(doc_signal, session_signal, toasts)` is called

**Then**: Info toast appears with title `"Saving workspace"` and detail `"Preparing data..."`

**Test name**: `fn save_workspace_shows_info_toast_when_saving_with_known_path()`

---

#### Behavior: save_workspace() shows Save As dialog when no file path set

**Given**: AppState with session signal where session has `file_path() == None`

**When**: `save_workspace(doc_signal, session_signal, toasts)` is called

**Then**: File dialog is shown with filter `"Seshat Diagram"` and extension `["json"]`

**Test name**: `fn save_workspace_shows_save_as_dialog_when_no_file_path_set()`

---

#### Behavior: save_workspace() updates session_signal with saved session on success

**Given**: Dirty session with file path `"/path/to/diagram.json"`, doc with revision 10

**When**: `save_workspace()` completes successfully

**Then**: `session_signal.read().is_dirty() == false` and `session_signal.read().last_saved_revision() == Revision(10)`

**Test name**: `fn save_workspace_updates_session_signal_with_cleared_dirty_flag_on_success()`

---

#### Behavior: save_workspace() shows success toast with file path on success

**Given**: Session with file path `"/path/to/diagram.json"`

**When**: `save_workspace()` completes successfully

**Then**: Success toast shows title `"Workspace saved"` and detail containing `"Saved to /path/to/diagram.json"`

**Test name**: `fn save_workspace_shows_success_toast_with_file_path_on_success()`

---

#### Behavior: save_workspace() shows error toast on IO failure

**Given**: Session with file_path set to path in non-writable directory

**When**: `save_workspace()` fails with IO error

**Then**: Error toast shows title `"Save failed"` and detail containing `"Save error:"`

**Test name**: `fn save_workspace_shows_error_toast_when_io_fails()`

---

#### Behavior: save_workspace() shows error toast on serialization failure

**Given**: Session with file_path set to valid path, but document has invalid state causing serialization to fail

**When**: `save_workspace()` fails with Serialize error

**Then**: Error toast shows title `"Save failed"` and detail containing `"Serialize error:"`

**Test name**: `fn save_workspace_shows_error_toast_when_serialization_fails()`

---

#### Behavior: save_workspace() dismisses toast when user cancels Save As dialog

**Given**: Session with `file_path() == None`

**When**: User cancels the Save As file dialog

**Then**: The info toast is dismissed, no error toast is shown

**Test name**: `fn save_workspace_dismisses_toast_when_user_cancels_save_as_dialog()`

---

#### Behavior: save_workspace() returns error toast in WASM build

**Given**: WASM target architecture (`#[cfg(target_arch = "wasm32")]`)

**When**: `save_workspace()` is called

**Then**: Error toast shows title `"Save not available"` with detail `"Backend has been decommissioned"`

**Test name**: `fn save_workspace_returns_error_toast_in_wasm_build()`

---

#### Behavior: apply_save_document() returns new session with cleared dirty flag

**Given**: Dirty session and valid document

**When**: `apply_save_document(&doc, &session, &path)` succeeds

**Then**: Returned session has `is_dirty() == false`

**Test name**: `fn apply_save_document_clears_dirty_flag_on_success()`

---

#### Behavior: apply_save_document() succeeds with empty document

**Given**: Dirty session and valid document with empty nodes `{}` and empty edges `{}`

**When**: `apply_save_document(&empty_doc, &session, &path)` succeeds

**Then**: Returned session has `is_dirty() == false`

**Test name**: `fn apply_save_document_succeeds_with_empty_document()`

---

#### Behavior: apply_save_document() preserves file path in session

**Given**: Session created with `DocumentSession::from_file(doc, PathBuf::from("/original.json"))`

**When**: `apply_save_document(&doc, &session, &temp_path)` succeeds where temp_path differs

**Then**: Returned session's `file_path() == Some(PathBuf::from("/original.json"))`

**Test name**: `fn apply_save_document_preserves_original_file_path_not_save_path()`

---

#### Behavior: apply_save_document() syncs revision from saved document

**Given**: Session created from doc with revision 5, but doc's revision is now 10

**When**: `apply_save_document(&current_doc, &session, &path)` succeeds

**Then**: Returned session's `last_saved_revision() == Revision(10)`

**Test name**: `fn apply_save_document_syncs_revision_from_current_document()`

---

#### Behavior: apply_save_document() returns Io error for invalid path

**Given**: Session with dirty document

**When**: `apply_save_document(&doc, &session, &PathBuf::from("/nonexistent/dir/file.json"))` is called

**Then**: Returns `Err(SaveError::Io(_))`

**Test name**: `fn apply_save_document_returns_io_error_for_nonexistent_directory()`

---

#### Behavior: apply_save_document() returns NoFilePath error when session has no path

**Given**: Session created without file path (using `DocumentSession::new()`)

**When**: `apply_save_document(&doc, &session, &path)` is called

**Then**: Returns `Err(SaveError::NoFilePath)`

**Test name**: `fn apply_save_document_returns_no_file_path_error_when_session_lacks_path()`

---

#### Behavior: apply_save_document() returns Serialize error when document validation fails

**Given**: Document that fails schema validation (e.g., invalid edge reference)

**When**: `apply_save_document(&doc, &session, &valid_path)` is called

**Then**: Returns `Err(SaveError::Serialize(_))`

**Test name**: `fn apply_save_document_returns_serialize_error_when_document_validation_fails()`

---

#### Behavior: apply_save_document() returns PathTraversalDenied error when path contains ".."

**Given**: Session with dirty document, valid file path that contains `".."`

**When**: `apply_save_document(&doc, &session, &PathBuf::from("../escape.json"))` is called

**Then**: Returns `Err(SaveError::Io(s))` where `s` contains `"Path traversal denied"`

**Test name**: `fn apply_save_document_returns_path_traversal_error_when_path_contains_double_dot()`

---

#### Behavior: apply_save_document() WASM variant always returns Io error

**Given**: WASM build, any document and session

**When**: `apply_save_document(&doc, &session, &path)` is called

**Then**: Returns `Err(SaveError::Io(s))` where `s` contains `"Save not available in WASM"`

**Test name**: `fn apply_save_document_returns_error_on_wasm_target()`

---

#### Behavior: apply_save_document() returns Io error when atomic write temp file creation fails

**Given**: Valid document, session with file path, but temp file creation fails (e.g., path too long for temp directory, permission issue in temp directory)

**When**: `apply_save_document(&doc, &session, &path)` is called

**Then**: Returns `Err(SaveError::Io(_))`

> **Note**: OS error messages are unpredictable. `Err(SaveError::Io(_))` is ACCEPTABLE here because we only verify the error variant, not the specific message. The mapping from `CliPersistenceError::TempFileError` → `SaveError::Io` is intentional.

**Test name**: `fn apply_save_document_returns_io_error_when_temp_file_creation_fails()`

---

#### Behavior: apply_save_document() returns Io error when atomic rename fails

**Given**: Valid document, session with file path, but atomic rename operation fails (e.g., target directory becomes read-only after temp write, disk full during rename)

**When**: `apply_save_document(&doc, &session, &path)` is called

**Then**: Returns `Err(SaveError::Io(_))`

> **Note**: OS error messages are unpredictable. `Err(SaveError::Io(_))` is ACCEPTABLE here because we only verify the error variant, not the specific message. The mapping from `CliPersistenceError::AtomicRenameError` → `SaveError::Io` is intentional.

**Test name**: `fn apply_save_document_returns_io_error_when_atomic_rename_fails()`

---

### 3.2 Open Scenarios

---

#### Behavior: open_workspace() shows info toast while loading

**Given**: Any app state

**When**: `open_workspace(signals, toasts, store_bridge)` is called

**Then**: Info toast appears with title `"Loading workspace"` and detail `"Reading persisted document..."`

**Test name**: `fn open_workspace_shows_info_toast_while_loading()`

---

#### Behavior: open_workspace() opens native file picker on native

**Given**: Native build (non-WASM)

**When**: `open_workspace()` is called

**Then**: `rfd::FileDialog::new().add_filter("Seshat Diagram", &["json"]).pick_file()` is invoked

**Test name**: `fn open_workspace_opens_native_file_picker_on_native()`

---

#### Behavior: open_workspace() opens browser file picker on WASM

**Given**: WASM build, no `__SESHAT_E2E_IMPORT_JSON` global set

**When**: `open_workspace()` is called

**Then**: Browser `<input type="file">` element is created and clicked

**Test name**: `fn open_workspace_opens_browser_file_picker_on_wasm()`

---

#### Behavior: open_workspace() uses E2E import JSON global when available (WASM)

**Given**: WASM build with `window.__SESHAT_E2E_IMPORT_JSON` set to valid JSON string

**When**: `open_workspace()` is called

**Then**: Document is loaded from the global variable, not from file picker

**Test name**: `fn open_workspace_uses_e2e_import_json_global_when_available()`

---

#### Behavior: open_workspace() resets revision to INITIAL on load

**Given**: Current document with revision 50

**When**: `open_workspace()` successfully loads a file

**Then**: Loaded document has `revision == Revision::INITIAL`

**Test name**: `fn open_workspace_resets_revision_to_initial_on_load()`

---

#### Behavior: open_workspace() pushes current doc to history before loading

**Given**: Current document with nodes, empty history

**When**: `open_workspace()` successfully loads a different file

**Then**: History can undo to previous document state

**Test name**: `fn open_workspace_pushes_current_doc_to_history_before_loading()`

---

#### Behavior: open_workspace() resets store bridge on native after successful load

**Given**: Native build with store_bridge configured

**When**: `open_workspace()` successfully loads a file

**Then**: `store_bridge.reset_store_sync()` is called

**Test name**: `fn open_workspace_resets_store_bridge_on_native_after_load()`

---

#### Behavior: open_workspace() shows error toast when file picker cancelled

**Given**: User cancels the native file picker

**When**: `open_workspace()` receives `None` from file dialog

**Then**: Info toast is dismissed, no error toast shown

**Test name**: `fn open_workspace_shows_error_toast_when_file_picker_cancelled()`

---

#### Behavior: open_workspace() shows error toast on load failure (native IO error)

**Given**: Native build, user selects a file, but file cannot be read (permission denied)

**When**: `open_workspace()` fails with IO error from `load_workspace_with_lkg()`

**Then**: Error toast shows title `"Load failed"` and detail containing `"IO error:"` or `"Cannot open file:"`

**Test name**: `fn open_workspace_shows_error_toast_on_native_io_failure()`

---

#### Behavior: open_workspace() shows error when both primary and LKG fail

**Given**: Primary file corrupt, LKG file also corrupt or missing

**When**: `open_workspace()` is called

**Then**: Error toast shows title `"Load failed"` with detail containing `"Cannot open file"` and `"Backup also unavailable"`

**Test name**: `fn open_workspace_shows_error_when_both_primary_and_lkg_fail()`

---

#### Behavior: open_workspace() uses LKG fallback when primary file corrupt

**Given**: Primary file contains invalid JSON, `.lkg/<filename>.lkg` contains valid document

**When**: `open_workspace()` is called

**Then**: Document loads successfully from LKG file, success toast shows detail containing `"Loaded from..."` with LKG path

**Test name**: `fn open_workspace_uses_lkg_fallback_when_primary_file_corrupt()`

---

#### Behavior: apply_open_document() creates session with correct file path

**Given**: Valid JSON content, file path `"/path/to/diagram.json"`

**When**: `apply_open_document(&current_doc, &history, valid_json, PathBuf::from("/path/to/diagram.json"))` succeeds

**Then**: Returned session's `file_path() == Some(PathBuf::from("/path/to/diagram.json"))`

**Test name**: `fn apply_open_document_creates_session_with_correct_file_path()`

---

#### Behavior: apply_open_document() happy path: returns doc with correct node and edge count

**Given**: Valid v2 JSON with 3 nodes and 2 edges, current doc is empty, empty history

**When**: `apply_open_document(&current_doc, &history, valid_json, path)` succeeds

**Then**: Returned `(next_doc, next_history, session)` where `next_doc.nodes.len() == 3`, `next_doc.edges.len() == 2`, and `session.file_path() == Some(path)`

**Test name**: `fn apply_open_document_returns_doc_with_correct_node_and_edge_count_on_happy_path()`

---

#### Behavior: apply_open_document() returns Parse error for invalid JSON

**Given**: Valid current document, invalid JSON string `"not valid json"`

**When**: `apply_open_document(&current_doc, &history, "not valid json", path)` is called

**Then**: Returns `Err(OpenError::Parse(s))` where `s.len() > 0`

**Test name**: `fn apply_open_document_returns_parse_error_for_invalid_json()`

---

#### Behavior: apply_open_document() returns Parse error for missing version field

**Given**: Valid current document, JSON missing required `"version"` field

**When**: `apply_open_document()` is called

**Then**: Returns `Err(OpenError::Parse(s))` where `s.contains("version")` (serde deserialize error for missing required field)

**Test name**: `fn apply_open_document_returns_parse_error_for_missing_version()`

---

#### Behavior: apply_open_document() returns Validation error for schema violations

**Given**: Valid current document, JSON that is syntactically valid but fails schema validation (e.g., node references non-existent node ID, edge has invalid arrow_type value)

**When**: `apply_open_document(&current_doc, &history, invalid_schema_json, path)` is called

**Then**: Returns `Err(OpenError::Validation(s))` where `s.contains("validation")`

> **Note**: String check is acceptable because validation error messages are generated by our code, not the OS.

**Test name**: `fn apply_open_document_returns_validation_error_for_schema_violations()`

---

### 3.3 Toast Helper Scenarios

---

#### Behavior: update_load_save_success() updates toast with Success intent and correct title/detail

**Given**: A `ToastHandle` from an existing toast, title `"Workspace saved"`, detail `"Saved to /path.json"`

**When**: `update_load_save_success(toast_handle, "Workspace saved", String::from("Saved to /path.json"))` is called

**Then**: The toast is updated with intent `ToastIntent::Success`, title `"Workspace saved"`, and detail `Some("Saved to /path.json")`

**Test name**: `fn update_load_save_success_updates_toast_with_success_intent_and_correct_content()`

---

#### Behavior: update_load_save_error() updates toast with Error intent and correct title/detail

**Given**: A `ToastHandle` from an existing toast, title `"Save failed"`, detail `"IO error: permission denied"`

**When**: `update_load_save_error(toast_handle, "Save failed", String::from("IO error: permission denied"))` is called

**Then**: The toast is updated with intent `ToastIntent::Error`, title `"Save failed"`, and detail `Some("IO error: permission denied")`

**Test name**: `fn update_load_save_error_updates_toast_with_error_intent_and_correct_content()`

---

### 3.4 Import/Transition Scenarios

---

#### Behavior: apply_import_contents() updates doc and history atomically on success

**Given**: Current document `"DocA"`, empty history, valid JSON for `"DocB"` with 2 nodes

**When**: `apply_import_contents(&mut doc, &mut history, valid_json)` succeeds

**Then**: `doc.nodes.len() == 2`, `history.can_undo() == true`

**Test name**: `fn apply_import_contents_updates_doc_and_history_atomically()`

---

#### Behavior: apply_import_contents() leaves doc and history unchanged on parse error

**Given**: Current document `"DocA"` with 3 nodes, history with prior state

**When**: `apply_import_contents(&mut doc, &mut history, "{invalid json")` fails

**Then**: `doc.nodes.len() == 3` and `history.can_undo() == true` (unchanged from before call)

> **Note**: Store original count before call: `let original_node_count = doc.nodes.len();` then assert `assert_eq!(doc.nodes.len(), original_node_count);`

**Test name**: `fn apply_import_contents_leaves_doc_and_history_unchanged_on_parse_error()`

---

#### Behavior: apply_import_contents() leaves doc and history unchanged on validation error

**Given**: Current document `"DocA"` with 3 nodes, history with prior state

**When**: `apply_import_contents(&mut doc, &mut history, valid_json_with_invalid_edges)` fails validation

**Then**: `doc.nodes.len() == 3` and `history.can_undo() == true` (unchanged from before call)

> **Note**: Store original count before call: `let original_node_count = doc.nodes.len();` then assert `assert_eq!(doc.nodes.len(), original_node_count);`

**Test name**: `fn apply_import_contents_leaves_doc_and_history_unchanged_on_validation_error()`

---

#### Behavior: prepare_import_transition() migrates legacy field names

**Given**: JSON with `font_size: 14` (camelCase) and `dagRank: "same"` (camelCase)

**When**: `prepare_import_transition(&current, json)` is called

**Then**: Parsed document has `nodes[].properties.font_size == 14` and `nodes[].dag_rank == 7` (normalized)

**Test name**: `fn prepare_import_transition_migrates_legacy_camelcase_fields_to_snakecase()`

---

#### Behavior: prepare_import_transition() migrates icon_data_url to icon_url for base64 data

**Given**: JSON with node containing `icon_data_url: "data:image/png;base64,SGVsbG8="`

**When**: `prepare_import_transition()` is called

**Then**: Parsed node has `metadata.icon_url == "/resources/{icon}"` and no `icon_data_url` field

**Test name**: `fn prepare_import_transition_migrates_base64_icon_data_url_to_icon_url()`

---

#### Behavior: prepare_import_transition() preserves existing icon_url when both exist

**Given**: JSON with node containing both `icon_url: "/custom/path"` and `icon_data_url: "data:..."`

**When**: `prepare_import_transition()` is called

**Then**: Parsed node has `metadata.icon_url == "/custom/path"` (preserved, not overwritten)

**Test name**: `fn prepare_import_transition_preserves_existing_icon_url_when_both_keys_exist()`

---

#### Behavior: prepare_import_transition() normalizes arrow types from legacy values

**Given**: JSON with edge having `arrowType: "diamond"` or `arrow_type: "open"`

**When**: `prepare_import_transition()` is called

**Then**: Parsed edge has normalized `arrow_type` (`ArrowType::Step` for "diamond", `ArrowType::Straight` for "open")

**Test name**: `fn prepare_import_transition_normalizes_legacy_arrow_type_values()`

---

#### Behavior: prepare_import_transition() happy path: valid v2 JSON without migration needed

**Given**: Valid v2 JSON with proper snake_case fields, no legacy fields

**When**: `prepare_import_transition(&current, json)` is called

**Then**: Returns `Ok((doc, history))` where `doc` has correct structure and `doc.revision == Revision::INITIAL`

**Test name**: `fn prepare_import_transition_parses_valid_v2_json_without_migration_errors()`

---

#### Behavior: parse_diagram_document_with_compat() rejects documents without version field

**Given**: JSON string missing the `"version"` field

**When**: `parse_diagram_document_with_compat(json)` is called

**Then**: Returns `Err` containing `"version"`

**Test name**: `fn parse_diagram_document_with_compat_rejects_documents_without_version()`

---

### 3.5 Keyboard Shortcut Scenarios

---

#### Behavior: Ctrl+S triggers save_workspace() when not editing input (native)

**Given**: Native app with focus on canvas (not input/textarea), Ctrl+S pressed

**When**: `use_global_keyboard(db_tx)` receives the keyboard event

**Then**: `save_workspace(doc_signal, session_signal, toasts)` is called with correct signals from app_state

**Test name**: `fn keyboard_ctrl_s_triggers_save_workspace_when_not_editing_input_on_native()`

---

#### Behavior: Ctrl+S triggers save_workspace() when not editing input (WASM)

**Given**: WASM app with focus on canvas (not input/textarea), Ctrl+S pressed

**When**: `use_global_keyboard(db_tx)` receives the keyboard event

**Then**: `save_workspace(doc_signal, session_signal, toasts)` is called with correct signals from app_state

**Note**: WASM keyboard handler (line 296) calls `save_workspace` directly without `store_bridge`

**Test name**: `fn keyboard_ctrl_s_triggers_save_workspace_when_not_editing_input_on_wasm()`

---

#### Behavior: Ctrl+O triggers open_workspace() when not editing input (native only)

**Given**: Native app with focus on canvas, Ctrl+O pressed

**When**: `use_global_keyboard(db_tx)` receives the keyboard event

**Then**: `open_workspace(signals, toasts, store_bridge)` is called with correct signals and store_bridge

**Note**: WASM build does NOT handle Ctrl+O (line 148-160 in keyboard.rs shows 'o' only handled in non-WASM cfg)

**Test name**: `fn keyboard_ctrl_o_triggers_open_workspace_on_native_only()`

---

### 3.6 Toolbar Button Scenarios

---

#### Behavior: Export button triggers save_workspace()

**Given**: Toolbar rendered with Export button visible, dirty session signal, doc signal, toast signal

**When**: Export button is clicked

**Then**: `save_workspace(doc_signal, session_signal, toasts)` is called

**Test name**: `fn toolbar_export_button_triggers_save_workspace()`

---

#### Behavior: Import button triggers open_workspace() (async-db, native only)

**Given**: Toolbar rendered with async-db feature enabled, Import button visible, WorkspaceSignals, toast signal, store_bridge

**When**: Import button is clicked

**Then**: `open_workspace(signals, toasts, Some(store_bridge))` is called

**Test name**: `fn toolbar_import_button_triggers_open_workspace()`

---

## 4. Proptest Invariants

### 4.1 apply_save_document() Revision Sync Invariant

**Invariant**: After successful `apply_save_document()`, the returned session's `last_saved_revision()` equals the revision of the document that was saved.

**Strategy**: 
- Generate arbitrary revision values (u64) using `any::<u64>()`
- Generate arbitrary valid `DiagramDocument` with that revision
- Generate arbitrary `DocumentSession`
- Call `apply_save_document()` and verify `last_saved_revision()` matches document revision

**Anti-invariant**: Passing a document with different revision than what was saved

**Implementation hint**:
```rust
proptest! {
    #[test]
    fn apply_save_document_revision_sync_invariant(doc in any::<DiagramDocument>(), session in any::<DocumentSession>()) {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();
        let result = apply_save_document(&doc, &session, &path);
        prop_assert!(result.is_ok());
        let saved_session = result.unwrap();
        prop_assert_eq!(saved_session.last_saved_revision(), doc.revision);
    }
}
```

---

### 4.2 apply_open_document() File Path Preservation Invariant

**Invariant**: `apply_open_document()` always returns a session whose `file_path()` equals exactly the `file_path` parameter passed in.

**Strategy**:
- Generate arbitrary `PathBuf` for file_path
- Generate arbitrary valid JSON content for v2 document
- Generate arbitrary current document and history
- Verify returned session's `file_path()` equals input `file_path`

**Anti-invariant**: None (should always hold)

**Implementation hint**:
```rust
proptest! {
    #[test]
    fn apply_open_document_file_path_preservation_invariant(
        path in "**",
        json_content in 'a'..'z',
    ) {
        let current = DiagramDocument::default();
        let history = History::new();
        let valid_json = make_valid_v2_json();
        let result = apply_open_document(&current, &history, &valid_json, PathBuf::from(&path));
        prop_assert!(result.is_ok());
        let session = result.unwrap().2;
        prop_assert_eq!(session.file_path(), Some(PathBuf::from(&path)));
    }
}
```

---

### 4.3 apply_import_contents() Atomicity Invariant

**Invariant**: On `apply_import_contents()` error, the `doc` and `history` parameters are completely unchanged (bit-for-bit identical to pre-call state).

**Strategy**:
- Generate arbitrary doc and history states
- Generate invalid JSON strings (syntax errors, schema violations)
- Verify doc and history are identical after call (clone before, compare after)

**Anti-invariant**: Mutation of doc or history on error path

**Implementation hint**:
```rust
proptest! {
    #[test]
    fn apply_import_contents_atomicity_on_error_invariant(
        mut doc in any::<DiagramDocument>(),
        mut history in any::<History>(),
        invalid_json in "**",
    ) {
        let doc_clone = doc.clone();
        let history_clone = history.clone();
        let result = apply_import_contents(&mut doc, &mut history, &invalid_json);
        prop_assert!(result.is_err());
        prop_assert_eq!(doc, doc_clone);
        prop_assert_eq!(history, history_clone);
    }
}
```

---

### 4.4 Serialization Round-Trip Invariant

**Invariant**: A document that serializes via `to_canonical_pretty_json()` and is then parsed via `parse_diagram_document_with_compat()` produces a document with identical structural content (nodes, edges, revision).

**Strategy**:
- Generate arbitrary valid `DiagramDocument`
- Serialize to JSON string via `to_canonical_pretty_json()`
- Parse back via `parse_diagram_document_with_compat()`
- Compare node count, edge count, revision

**Anti-invariant**: Data loss or transformation during serialization/deserialization

**Implementation hint**:
```rust
proptest! {
    #[test]
    fn serialization_roundtrip_invariant(doc in any::<DiagramDocument>()) {
        let json = doc.to_canonical_pretty_json();
        let parsed = parse_diagram_document_with_compat(&json);
        prop_assert!(parsed.is_ok());
        let parsed_doc = parsed.unwrap();
        prop_assert_eq!(doc.nodes.len(), parsed_doc.nodes.len());
        prop_assert_eq!(doc.edges.len(), parsed_doc.edges.len());
        prop_assert_eq!(doc.revision, parsed_doc.revision);
    }
}
```

---

## 5. Fuzz Targets

### 5.1 parse_diagram_document_with_compat()

**Input type**: `&str` (raw JSON bytes as string)

**Risk**: 
- Panic on malformed JSON
- Panic on invalid UTF-8
- Resource exhaustion (OOM on huge allocations)
- Logic errors in migration code with unexpected field combinations

**Corpus seeds**:
- Valid v2 document with nodes and edges
- Document with legacy camelCase fields
- Document with `icon_data_url` base64 data
- Document missing optional fields
- Empty document `{"version": 2, "document": {"nodes": {}, "edges": {}}}`
- Document with version 1

**Fuzz directive**: `cargo fuzz run parse_diagram_document`

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use diagram_tool::persistence_compat::parse_diagram_document_with_compat;

fuzz_target(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_diagram_document_with_compat(s);
    }
});
```

---

### 5.2 save_workspace_atomic()

**Input type**: Arbitrary `DiagramDocument` struct

**Risk**:
- Panic on invalid document state
- Panic on JSON serialization failure
- Filesystem errors not handled gracefully
- Temp file left behind on crash

**Corpus seeds**:
- Default document
- Document with unicode labels
- Document with very long label strings
- Document with special characters in node IDs

**Note**: This is already partially covered by `cli_persistence/tests.rs` but should have dedicated fuzz target.

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use diagram_tool::cli_persistence::save_workspace_atomic;
use diagram_models::document::DiagramDocument;

fuzz_target(|doc: DiagramDocument| {
    let temp_dir = std::env::temp_dir();
    let target = temp_dir.join("fuzz_output.json");
    let _ = save_workspace_atomic(&doc, &target);
    let _ = std::fs::remove_file(target);
});
```

---

### 5.3 apply_import_contents()

**Input type**: Arbitrary `(DiagramDocument, History, &str)` — current state + JSON string

**Risk**:
- Panic on deeply nested JSON
- Panic on JSON with wrong types for expected fields
- State corruption on error path (violates atomicity invariant)
- Denial of service via huge JSON payload

**Corpus seeds**:
- Valid JSON with single node
- Valid JSON with many nodes
- JSON with deeply nested objects
- JSON with extremely long strings
- JSON with unexpected field types

**Implementation**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use diagram_tool::toolbar::persistence::common::apply_import_contents;
use diagram_models::document::DiagramDocument;
use diagram_tool::history::History;

fuzz_target |(mut doc, mut history, json)| {
    let _ = apply_import_contents(&mut doc, &mut history, json);
};
```

---

## 6. Kani Harnesses

### 6.1 apply_save_document() — Path Traversal Prevention

**Property**: `apply_save_document()` with any `PathBuf` argument that contains `".."` will return `Err(SaveError::Io(s))` where `s` contains `"Path traversal denied"`, for any document and session.

**Bound**: Path strings up to 1024 characters, documents with up to 100 nodes

**Rationale**: Path traversal is a security-critical bug. Property testing cannot exhaustively verify path safety because it depends on string parsing logic. Kani can symbolically execute the path validation to prove the `".."` check is comprehensive.

**Implementation hint**:
```rust
use diagram_tool::ui::toolbar::persistence::save::apply_save_document;
use diagram_tool::SaveError;
use diagram_models::document::{DiagramDocument, DocumentSession};

#[kani::proof]
fn apply_save_document_rejects_path_traversal() {
    // Symbolic path containing ".."
    let path: PathBuf = kani::any();
    let doc: DiagramDocument = kani::any();
    let session: DocumentSession = kani::any();
    
    // Only test if path contains ".."
    if path.to_string_lossy().contains("..") {
        let result = apply_save_document(&doc, &session, &path);
        assert!(matches!(result, Err(SaveError::Io(s)) if s.contains("Path traversal denied")));
    }
}
```

---

### 6.2 apply_import_contents() — Atomicity on Error

**Property**: When `apply_import_contents()` returns an `Err`, the `doc` and `history` arguments are bit-for-bit identical to their pre-call values, for ANY invalid JSON input.

**Bound**: JSON strings up to 1MB, arbitrary current document state with up to 100 nodes

**Rationale**: This is a critical invariant. Property tests with generated inputs can catch many cases but cannot prove absence of partial mutation. Kani can symbolically execute the error path to verify no mutation occurs before the error is returned.

**Implementation hint**:
```rust
use diagram_tool::ui::toolbar::persistence::common::apply_import_contents;
use diagram_models::document::DiagramDocument;
use diagram_tool::history::History;

#[kani::proof]
fn apply_import_contents_preserves_state_on_error() {
    let mut doc: DiagramDocument = kani::any();
    let mut history: History = kani::any();
    let invalid_json: String = kani::any();
    
    // Get pre-state
    let doc_pre = doc.clone();
    let history_pre = history.clone();
    
    let result = apply_import_contents(&mut doc, &mut history, &invalid_json);
    
    if result.is_err() {
        assert!(doc_pre == doc);
        assert!(history_pre == history);
    }
}
```

---

## 7. Mutation Checkpoints

### 7.1 Critical Mutations for save_workspace()

| Mutation | Must Be Caught By |
|----------|------------------|
| Remove `is_dirty()` check before clearing flag | `apply_save_document_clears_dirty_flag_on_success` |
| Skip `mark_saved()` call | `apply_save_document_syncs_revision_from_current_document` |
| Omit `fsync()` call | `save_workspace_atomic_persists_data_to_disk` (existing integration test) |
| Skip temp file cleanup on error | `given_atomic_save_when_complete_then_no_temp_files_remain` (existing) |
| Wrong revision assignment | `apply_save_document_syncs_revision_from_current_document` |
| Map `CliPersistenceError::PathTraversalDenied` to wrong variant | `apply_save_document_returns_path_traversal_error_when_path_contains_double_dot` |

---

### 7.2 Critical Mutations for open_workspace()

| Mutation | Must Be Caught By |
|----------|------------------|
| Skip `push()` to history | `open_workspace_pushes_current_doc_to_history_before_loading` |
| Skip `Revision::INITIAL` reset | `open_workspace_resets_revision_to_initial_on_load` |
| Skip LKG fallback attempt | `open_workspace_uses_lkg_fallback_when_primary_file_corrupt` |
| Skip store_bridge reset | `open_workspace_resets_store_bridge_on_native_after_load` |
| Wrong error mapping for IO error | `open_workspace_shows_error_toast_on_native_io_failure` |

---

### 7.3 Critical Mutations for apply_import_contents()

| Mutation | Must Be Caught By |
|----------|------------------|
| Mutate doc BEFORE validate (move order) | `apply_import_contents_leaves_doc_and_history_unchanged_on_validation_error` |
| Skip history push on success | `apply_import_contents_updates_doc_and_history_atomically` |
| Skip error rollback | `apply_import_contents_leaves_doc_and_history_unchanged_on_parse_error` |

---

### 7.4 Critical Mutations for prepare_import_transition()

| Mutation | Must Be Caught By |
|----------|------------------|
| Skip revision reset to INITIAL | `prepare_import_transition_parses_valid_v2_json_without_migration_errors` |
| Skip legacy field migration | `prepare_import_transition_migrates_legacy_camelcase_fields_to_snakecase` |
| Overwrite existing icon_url with icon_data_url | `prepare_import_transition_preserves_existing_icon_url_when_both_keys_exist` |

---

### 7.5 Mutation Kill Rate Target

**Threshold**: ≥90% of mutations must be caught by existing tests.

**Coverage Plan**: The `cli_persistence/tests.rs`, `open_tests.rs`, `save.rs` tests, and `tests_import.rs` combined with the new tests above should achieve ≥90% kill rate.

---

## 8. Combinatorial Coverage Matrix

### 8.1 apply_save_document()

| Scenario | Input Class | Expected Output | Test Layer |
|----------|-------------|----------------|------------|
| Happy path | Valid doc + session + valid path | Ok(session with cleared dirty flag) | unit |
| Io error: no directory | Valid doc + session + path to nonexistent dir | Err(SaveError::Io(_)) | unit |
| Io error: permission denied | Valid doc + session + path to unwritable location | Err(SaveError::Io(_)) | unit |
| Serialize error: validation fails | Valid doc + session + valid path but doc fails schema | Err(SaveError::Serialize(_)) | unit |
| Serialize error: path traversal | Valid doc + session + path with ".." | Err(SaveError::Io(contains "Path traversal denied")) | unit |
| No file path | Valid doc + session created via new() | Err(SaveError::NoFilePath) | unit |
| Atomic rename failure | Valid doc + session + path causing rename failure | Err(SaveError::Io(contains "Atomic rename failed")) | unit |
| Revision sync | Doc rev 10, session rev 5 | Ok with last_saved_rev=10 | unit |
| File path preservation | Session from "/original.json", save to "/tmp/new.json" | Ok session file_path="/original.json" | unit |
| WASM always error | Any inputs on wasm32 target | Err(SaveError::Io(contains "Save not available in WASM")) | unit |

---

### 8.2 apply_open_document()

| Scenario | Input Class | Expected Output | Test Layer |
|----------|-------------|----------------|------------|
| Happy path: valid v2 | Valid JSON v2 doc + current + history | Ok((doc, history, session)) with correct node/edge count | unit |
| Happy path: file path | Valid JSON + path "/test.json" | session.file_path() == "/test.json" | unit |
| Happy path: revision reset | Valid JSON with rev 999 | next_doc.revision == Revision::INITIAL | unit |
| Parse error: invalid JSON | "{not json}" + current + history | Err(OpenError::Parse(_)) | unit |
| Parse error: missing version | JSON without version field | Err(OpenError::Parse(contains "version")) | unit |
| Validation error: bad schema | JSON with invalid structure | Err(OpenError::Validation(_)) | unit |
| History push | Valid JSON + current doc | next_history.can_undo() == true | unit |

---

### 8.3 prepare_import_transition()

| Scenario | Input Class | Expected Output | Test Layer |
|----------|-------------|----------------|------------|
| Happy path: valid v2 | Valid v2 JSON | Ok((doc, history)) | unit |
| Happy path: revision reset | Valid v2 JSON | doc.revision == Revision::INITIAL | unit |
| Parse error | Malformed JSON | Err(ImportTransitionError::Parse(_)) | unit |
| Validation error | Valid JSON but invalid schema | Err(ImportTransitionError::Validation(_)) | unit |
| Font size migration | JSON with `font_size` | doc.nodes[].font_size populated | unit |
| Dag rank migration | JSON with `dagRank` | doc.nodes[].dag_rank == 7 | unit |
| Arrow type: diamond | JSON with `arrowType: "diamond"` | edge.arrow_type == ArrowType::Step | unit |
| Arrow type: open | JSON with `arrow_type: "open"` | edge.arrow_type == ArrowType::Straight | unit |
| Icon data URL base64 | JSON with base64 icon_data_url | metadata.icon_url == "/resources/{icon}" | unit |
| Icon URL preserved | JSON with both icon_url and icon_data_url | metadata.icon_url preserved | unit |
| Version 1 doc | JSON with version 1 | Ok (migrates successfully) | unit |

---

### 8.4 load_workspace_with_lkg()

| Scenario | Input Class | Expected Output | Test Layer |
|----------|-------------|----------------|------------|
| Happy path | Valid file at path | Ok(doc) | integration |
| Missing file | File doesn't exist | Err(CliPersistenceError::NoValidDocument(_)) | integration |
| Invalid JSON | File contains "not json" | Err(CliPersistenceError::NoValidDocument(_)) | integration |
| LKG fallback | Primary corrupt, LKG valid | Ok(doc from LKG) | integration |
| Both fail | Primary corrupt, LKG corrupt | Err(CliPersistenceError::NoValidDocument(_)) | integration |
| Schema validation fail | Valid JSON but invalid schema | Err(CliPersistenceError::NoValidDocument(_)) | integration |
| IO error | File exists but cannot read | Err(CliPersistenceError::IoError(_)) | integration |

---

## 9. Error Enum Coverage

### SaveError

| Variant | Covered By | Test Name |
|---------|------------|-----------|
| NoFilePath | Behavior 13 | `apply_save_document_returns_no_file_path_error_when_session_lacks_path` |
| Serialize(String) | Behavior 14 | `apply_save_document_returns_serialize_error_when_document_validation_fails` |
| Io(String) | Behaviors 12, 15 | `apply_save_document_returns_io_error_for_nonexistent_directory`, `apply_save_document_returns_path_traversal_error_when_path_contains_double_dot` |

### OpenError

| Variant | Covered By | Test Name |
|---------|------------|-----------|
| Parse(String) | Behaviors 30, 31 | `apply_open_document_returns_parse_error_for_invalid_json`, `apply_open_document_returns_parse_error_for_missing_version` |
| Validation(String) | (other schema violations) | `apply_open_document_returns_validation_error_for_schema_violations` |
| Io(String) | Not applicable | `apply_open_document` takes `&str` content, not a file path — IO errors arise only at the `open_workspace` action layer |

### ImportTransitionError

| Variant | Covered By | Test Name |
|---------|------------|-----------|
| Parse(String) | Behavior 35 | `apply_import_contents_leaves_doc_and_history_unchanged_on_parse_error` |
| Validation(String) | Behavior 36 | `apply_import_contents_leaves_doc_and_history_unchanged_on_validation_error` |

### CliPersistenceError

| Variant | Covered By | Test Name |
|---------|------------|-----------|
| IoError | `cli_persistence/tests.rs` | existing |
| ParseError | `cli_persistence/tests.rs` | existing |
| ValidationError | `cli_persistence/tests.rs` | existing |
| TempFileError | Behavior 314 | `apply_save_document_returns_io_error_when_temp_file_creation_fails` |
| AtomicRenameError | Behavior 320 | `apply_save_document_returns_io_error_when_atomic_rename_fails` |
| NoValidDocument | Behaviors 26, integration | `open_workspace_shows_error_when_both_primary_and_lkg_fail`, existing |
| PathTraversalDenied | Behavior 15 | `apply_save_document_returns_path_traversal_error_when_path_contains_double_dot` |

> **Note**: Behaviors 314 and 320 show SOURCE error types (TempFileError, AtomicRenameError) in this table because they document what `CliPersistenceError` variants are caught. The BDD "Then:" assertions show DESTINATION error type (SaveError::Io after mapping). This IS correct — the mapping converts `CliPersistenceError → SaveError`.

---

## 10. Open Questions

1. **Auto-save interaction**: Does `auto_save.rs` interact with `save_workspace()`? Are there race conditions between manual and auto-save?
2. **Large file handling**: Is there a maximum document size? What happens when a document exceeds available memory?
3. **Concurrent access**: What happens if the same file is opened from two instances simultaneously? Is there any file locking?
4. **Export button vs Save**: The toolbar has "Export" button that calls `save_workspace()`. Is there a difference between Export and Save As semantics?
5. **Import button visibility**: The Import button requires `async-db` feature. Is this intentional for WASM builds?
6. **Max document size**: ~~Is there a hard limit on document size?~~ Closed — bounded by available memory; stress testing not in scope.
7. **Revision overflow**: What happens when document revision exceeds u64::MAX? No explicit test scenarios for extreme revision values.
8. **Empty string handling**: How does the system handle empty file paths, empty node IDs, or empty edge labels? Some edge cases may not be validated.
9. **Max JSON payload**: Is there a limit on JSON payload size during import? Large payloads could cause OOM before validation runs.
10. **Empty JSON string boundary for apply_import_contents**: Converted to BDD scenario — see Behavior 35b below.

#### Behavior: apply_import_contents() returns Parse error on empty string

**Given**: Current document `"DocA"` with 3 nodes, empty history

**When**: `apply_import_contents(&mut doc, &mut history, "")` is called

**Then**: Returns `Err(ImportTransitionError::Parse(s))` where `s.len() > 0`

> **Note**: String check is acceptable because parse error messages are generated by our code (serde), not the OS.

**Test name**: `fn apply_import_contents_returns_parse_error_on_empty_string()`

---

## 11. Test Implementation Notes

### 11.1 Unit Test Files
- `diagram_tool/src/ui/toolbar/persistence/save.rs` — existing tests at bottom, add new tests for Serialize/PathTraversal
- `diagram_tool/src/ui/toolbar/persistence/open_tests.rs` — existing tests, add IO error test
- `diagram_tool/src/ui/toolbar/persistence/common.rs` — add tests for `update_load_save_success` and `update_load_save_error`
- `diagram_tool/src/ui/toolbar/persistence/tests_import.rs` — import tests, add happy path test
- `diagram_tool/src/cli_persistence/tests.rs` — atomic write tests (existing)

### 11.2 Integration Test Targets
- `diagram_tool/tests/save_load_integration_tests.rs` — new file for integration tests
  - `test_open_workspace_io_error_on_native`
  - `test_lkg_fallback_chain`
  - `test_keyboard_shortcut_triggers_save`

### 11.3 Proptest Implementation
- Add `proptest` dependency to `Cargo.toml` if not present
- Add proptest configurations in `diagram_tool/src/ui/toolbar/persistence/mod.rs` or separate module
- Run with: `cargo test --lib -- proptest`

### 11.4 Fuzz Implementation
- Add `cargo-fuzz` crate and `libfuzzer-sys` dependency
- Initialize with: `cargo +nightly fuzz init`
- Fuzz targets go in `diagram_tool/fuzz/fuzz_targets/`

### 11.5 Kani Implementation
- Install Kani: `cargo install kani`
- Harnesses go in `diagram_tool/kani/` directory
- Run with: `cargo kani`

### 11.6 Protected Test Files
Per AGENTS.md, these are CONTRACT TESTS and MUST NOT be overwritten:
- `diagram_models/src/io_tests.rs` — IO-001 to IO-015
- `diagram_tool/src/test_infrastructure_tests.rs` — P1-P4, Q1-Q3
- `diagram_tool/src/geometry/**/*.rs` — GEO-001 to GEO-030

---

*Test plan generated by test-planner skill. Implementation requires: unit tests in save.rs/open_tests.rs/common.rs, integration tests in tests/ directory, E2E tests via Playwright, proptest invariants in dedicated module, fuzz targets in fuzz/ directory, and Kani harnesses in kani/ directory.*
