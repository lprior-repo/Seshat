# ADR-016: Persistence Strategy

## Status

**Accepted** (2026-03-23)

## Context

Seshat has two persistence mechanisms: the filesystem stores JSON files that users can save and load, while SQLite provides a database layer for application state. We need to define which is the source of truth and how they interact, especially when users save and load files, when AI agents modify diagrams via CLI, and when the app crashes with unsaved changes.

We already have a solid foundation. The `DiagramDocument` type with serde serialization is defined in `diagram_models/src/document/mod.rs`. SQLite persistence exists per ADR-003. The Single-Log WAL architecture via Restate is documented in `docs/12_SINGLE_LOG_ARCHITECTURE.md`.

## Decision

### The Persistence Hierarchy

The filesystem is the primary source of truth. The JSON file on disk is authoritative — it's what users explicitly save and what they expect to persist. SQLite is a recovery cache only, used for crash recovery and as a substrate for AI synchronization via the WAL.

This creates a three-tier hierarchy. At the top is the filesystem (`.seshat.json` files), which users explicitly save to and load from, and which is version-controllable and shareable. In the middle is in-memory state (Dioxus Signals), which represents what's currently rendered on screen and tracks the file path, last saved revision, and dirty flag. At the bottom is SQLite, which serves only for crash recovery and as the AI sync substrate — it is never user-visible.

### Dirty Flag Tracking

The `DocumentSession` struct tracks the relationship between in-memory state and the persisted file. It contains the `DiagramDocument` itself, an optional `file_path` (None means "Untitled"), and the `last_saved_revision`. The `is_dirty()` method returns true when the document's current revision differs from the last saved revision.

The UI indicates dirty state by showing a dot in the tab title: `● my-diagram.seshat.json`. This follows the convention used by most modern editors.

### File Operations

When a user creates a new document, we create an empty `DiagramDocument`, set `file_path` to None, and set `dirty` to false.

When a user opens a file, we load the JSON from the file, validate it per ADR-014, replace the in-memory state, set `file_path` to the chosen path, set `dirty` to false, and critically, we **overwrite SQLite** with the file contents. This ensures the cache always matches the source of truth.

When a user saves (Cmd+S or Save menu), if a file path exists, we write the JSON to the file and set `dirty` to false. If no file path exists, we trigger Save As. Save As prompts the user for a location, writes the JSON there, sets `file_path`, and clears the dirty flag.

Auto-save runs on a 30-second timer and writes to **SQLite only**, not the file. This preserves the user's work for crash recovery without modifying the file they explicitly saved.

### SQLite Cache Rules

The cache follows strict rules to maintain consistency with the file. When a user opens a file, we overwrite SQLite with the file contents — the cache must match the source of truth. When a user saves a file, we update SQLite (redundant but keeps the cache fresh). When a user makes changes, auto-save updates SQLite every 30 seconds.

On app crash and restart, if the SQLite revision is greater than the file revision, we offer a recovery prompt. If the user accepts recovery, we load from SQLite and mark the document dirty (the user should save to file to persist). If the user rejects recovery, we delete the SQLite cache entry and load from the file.

### AI CLI Path

AI agents interact through a specific path that respects the persistence hierarchy. Per `10_AI_CLI_CONTRACT.md`, the AI calls `seshat propose --input changes.json --base-revision 42`. The CLI validates the schema, revision, and referential integrity. If valid, the proposal submits to the WAL (Restate). The UI receives the proposal via WAL subscription and displays a ghost diff (per ADR-015). The user reviews and accepts or rejects. Only then is the document updated, and auto-save updates SQLite.

The key constraint is that AI never writes directly to the file. The file is only updated on explicit user save. This ensures users maintain full control over what gets persisted.

### Error Handling

When disk is full on save, we show a toast "Save failed: disk full", keep the dirty flag true, and offer Save As to a different location. When the file is read-only, we show a toast and offer Save As elsewhere. When a network drive is disconnected, we show "Cannot reach save location. Changes saved to local cache" and auto-retry when reconnected.

When JSON is corrupted on load, we reject the load and offer recovery from cache. When the schema is an old version, we run the migration and show "File upgraded from v1 to v2". When the schema is a newer version, we load with warnings, preserve unknown fields, and show "File from newer Seshat version".

### Concurrent Access

When multiple browser tabs are open, each has its own in-memory state. On save conflict, we show "File changed since opened" with options to overwrite, keep the current version, or show a diff.

When AI proposes while the user is editing, a ghost diff appears per ADR-015, and the user decides. User always wins.

When both user and AI try to save simultaneously, the user always wins. The AI's conditional append fails if the revision has advanced.

### Cache Cleanup

To prevent cache accumulation, we clean up in three scenarios: on successful file save (the cache is now redundant), on explicit new document creation (the old cache is invalid), and periodically for entries older than 7 days with no matching file.

### Contracts

The system maintains several invariants: the file on disk is the source of truth, SQLite is a recovery cache only and not authoritative, SQLite is always overwritten on file load, and the dirty flag accurately reflects unsaved changes.

For load operations, preconditions are: file exists at the path, file is readable, file contains valid UTF-8, and file content is valid JSON per ADR-014. Postconditions are: in-memory state matches file content, SQLite is overwritten with file content, dirty is false, and file_path is set.

For save operations, preconditions are: parent directory exists and is writable, document has no NaN/Infinity floats, and document passes structural validation. Postconditions are: file written successfully, dirty is false, and SQLite is updated.

### Module Structure

The `diagram_models/src/physical_io/` directory contains `mod.rs` (load_document, save_document), `builder.rs` (DiagramBuilder), and `migration.rs` (migrate_schema). A new `diagram_tool/src/document_session.rs` file will contain `DocumentSession` and dirty tracking. A new `diagram_tool/src/autosave.rs` file will handle auto-save timing and SQLite writes. In the UI layer, `tab_title.rs` displays the dirty indicator, and `recovery_modal.rs` handles crash recovery prompts.

## Consequences

### Positive

The mental model is simple: file equals truth. Crash recovery via SQLite provides a safety net. The `.seshat.json` files are Git-friendly and can be versioned, diffed, and merged. Files are portable — users can email or share them.

### Negative

Having two sources of truth requires careful synchronization, though clear rules mitigate this. Crash recovery adds UI complexity with the recovery prompt.

### Risks

If the rules aren't followed, cache drift could occur where SQLite and the file diverge. This is mitigated by always overwriting SQLite on file load. Large caches could accumulate over time, mitigated by periodic cleanup of old entries.

---

## Machine-Readable Spec (JSONL)

```jsonl
{"type":"adr","id":"adr-016","title":"Persistence Strategy","status":"accepted","date":"2026-03-23","context":{"problem":"Seshat has two persistence mechanisms - filesystem (JSON files) and SQLite (database). Need to define source of truth and interaction rules for user saves, AI modifications, and crash recovery.","existing":["DiagramDocument with serde","SQLite per ADR-003","Single-Log WAL per docs/12_SINGLE_LOG_ARCHITECTURE.md"]},"decision":{"primary_source":"filesystem","hierarchy":[{"level":1,"name":"filesystem","description":"PRIMARY SOURCE OF TRUTH - User explicitly saves here","format":".seshat.json","role":"authoritative"},{"level":2,"name":"in_memory","description":"WORKING STATE - What's rendered on screen","role":"ephemeral","tracks":["file_path","last_saved_revision","dirty_flag"]},{"level":3,"name":"sqlite","description":"RECOVERY CACHE ONLY","role":"subsidiary","purposes":["crash_recovery","ai_sync_substrate"]}]},"dirty_flag":{"rust_type":"DocumentSession","fields":[{"name":"doc","type":"DiagramDocument"},{"name":"file_path","type":"Option<PathBuf>","description":"None = Untitled"},{"name":"last_saved_revision","type":"Revision"}],"computed":{"is_dirty":"doc.revision != last_saved_revision"},"ui_indicator":"Tab title shows ● when dirty: ● my-diagram.seshat.json"},"file_operations":[{"name":"new","trigger":"User clicks New","steps":["Create empty DiagramDocument","file_path = None","dirty = false"]},{"name":"open","trigger":"User selects file","steps":["Load JSON from file","Validate per ADR-014","Replace in-memory state","file_path = Some(path)","dirty = false","OVERWRITE SQLite with file contents"]},{"name":"save","trigger":"Cmd+S or Save menu","precondition":"file_path.is_some()","steps":["Write JSON to file","dirty = false"],"fallback":"If no file_path, trigger Save As"},{"name":"save_as","trigger":"Menu or first save on new doc","steps":["Prompt user for location","Write JSON to chosen path","file_path = Some(path)","dirty = false"]},{"name":"auto_save","trigger":"Timer every 30 seconds","target":"SQLite ONLY","file_unchanged":true,"purpose":"Crash recovery cache"}],"sqlite_cache_rules":[{"scenario":"user_opens_file","action":"OVERWRITE SQLite with file contents","rationale":"Cache must match source of truth"},{"scenario":"user_saves_file","action":"Update SQLite (redundant but keeps cache fresh)","rationale":"Sync cache with persisted state"},{"scenario":"user_makes_changes","action":"Auto-save to SQLite every 30s","rationale":"Recovery safety net"},{"scenario":"app_crashes_then_restarts","action":"If SQLite revision > file revision, offer recovery prompt","rationale":"User may have unsaved work"},{"scenario":"recovery_accepted","action":"Load from SQLite, mark dirty","rationale":"User should save to file to persist"},{"scenario":"recovery_rejected","action":"Delete SQLite cache entry, load from file","rationale":"User chose to discard unsaved work"}],"ai_cli_path":{"source":"10_AI_CLI_CONTRACT.md","flow":["AI CLI calls: seshat propose changes.json --base-revision 42","CLI validates: schema, revision, referential integrity","If valid: submit to WAL (Restate)","WAL performs conditional append","UI receives via WAL subscription","Ghost diff appears (per ADR-015)","User reviews and accepts/rejects","Document updated","Auto-save updates SQLite"],"constraint":"AI never writes directly to file. File is only updated on explicit user save."},"error_handling":[{"failure":"disk_full_on_save","behavior":["Toast: Save failed: disk full","Keep dirty = true","Offer Save As to different location"]},{"failure":"read_only_file","behavior":["Toast: File is read-only","Offer Save As to different location"]},{"failure":"network_drive_disconnected","behavior":["Toast: Cannot reach save location. Changes saved to local cache.","Auto-retry when reconnected"]},{"failure":"corrupted_json_on_load","behavior":["Reject load","Offer: File corrupted. Recover from cache?","SQLite may have last good state"]},{"failure":"old_schema_version","behavior":["Run migration","Toast: File upgraded from v1 to v2"]},{"failure":"newer_schema_version","behavior":["Load with warnings","Preserve unknown fields","Toast: File from newer Seshat version"]}],"concurrent_access":[{"scenario":"multiple_browser_tabs","behavior":"Each has own in-memory state","conflict_resolution":"On save conflict: File changed since opened. Show options: Overwrite / Keep Mine / Show Diff"},{"scenario":"ai_proposes_while_user_editing","behavior":"Ghost diff appears per ADR-015","resolution":"User decides. User always wins."},{"scenario":"user_and_ai_both_try_to_save","behavior":"User always wins","mechanism":"AI conditional append fails if revision advanced"}],"cache_cleanup":[{"trigger":"successful_file_save","action":"Remove cache entry (now redundant)"},{"trigger":"explicit_new_document","action":"Remove old cache entry (invalid)"},{"trigger":"periodic_cleanup","condition":"Entries older than 7 days with no matching file","action":"Remove stale entries"}],"contracts":{"invariants":["I1: File on disk = source of truth","I2: SQLite = recovery cache only, not authoritative","I3: On file load, SQLite is overwritten","I4: Dirty flag accurately reflects unsaved changes"],"preconditions_load":["PL1: File exists at path","PL2: File is readable","PL3: File contains valid UTF-8","PL4: File content is valid JSON per ADR-014"],"postconditions_load":["QL1: In-memory state matches file content","QL2: SQLite overwritten with file content","QL3: dirty = false","QL4: file_path = Some(path)"],"preconditions_save":["PS1: Parent directory exists and is writable","PS2: Document has no NaN/Infinity floats","PS3: Document passes structural validation"],"postconditions_save":["QS1: File written successfully","QS2: dirty = false","QS3: SQLite updated (if auto-save not already done)"]},"module_structure":{"diagram_models/src/physical_io/":["mod.rs - load_document, save_document","builder.rs - DiagramBuilder","migration.rs - migrate_schema"],"diagram_tool/src/":["document_session.rs - NEW: DocumentSession, dirty tracking","autosave.rs - NEW: Auto-save timer, SQLite writes"],"diagram_tool/src/ui/":["tab_title.rs - Dirty indicator display","recovery_modal.rs - Crash recovery prompt"]},"consequences":{"positive":["Simple mental model: File = truth","Crash recovery via SQLite","Git-friendly .seshat.json files","Portable: users can email/share files"],"negative":["Two sources of truth require careful sync","Recovery adds UI complexity"],"risks":[{"risk":"Cache drift if rules not followed","mitigation":"Always overwrite SQLite on file load"},{"risk":"Large cache accumulation","mitigation":"Periodic cleanup of old entries"}]},"related":["ADR-003: SQLite Persistence","ADR-014: Diagram JSON Schema","ADR-015: Ghost Diff System","12_SINGLE_LOG_ARCHITECTURE.md","10_AI_CLI_CONTRACT.md"]}
```
