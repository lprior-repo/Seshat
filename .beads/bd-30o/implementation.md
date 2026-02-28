# Implementation: schema: Two-way JSON schema sync for AI integration

bead_id: bd-30o
bead_title: schema: Two-way JSON schema sync for AI integration
phase: p2
updated_at: 2026-02-28T19:45:00Z

## Implementation Status: PARTIAL (2/3 features)

### Implemented Features

1. **JSON Export** (`diagram_tool/src/ui/toolbar/export_actions.rs:83-97`):
   ```rust
   pub fn export_json(doc_signal: Signal<DiagramDocument>) {
       let doc = doc_signal.read().clone();
       if let Ok(json) = serde_json::to_vec_pretty(&doc) {
           // Downloads as diagram.json
       }
   }
   ```
   - Exports full diagram state as JSON
   - Works in both WASM and CLI modes

2. **JSON Import** (`diagram_tool/src/ui/toolbar/persistence.rs:20-35`):
   ```rust
   fn prepare_import_transition(current: &DiagramDocument, contents: &str) -> Result<...>
   fn apply_import_contents(doc: &mut DiagramDocument, history: &mut History, contents: &str) -> Result<...>
   ```
   - Validates JSON before applying
   - Preserves history (undo stack)
   - Provides error messages for invalid JSON

3. **Import Button** (`diagram_tool/src/ui/toolbar/persistence.rs:161-194`):
   - File picker for .json files
   - Uses FileReader API for browser
   - Validates before applying

### Not Implemented

1. **Auto-save to localStorage on changes**:
   - Manual save via toolbar button only
   - No automatic persistence on state changes
   - Could be implemented using Dioxus effects

2. **Schema Versioning**:
   - No version field in document schema
   - Could break forward compatibility

## Verification Evidence

- Moon check: PASSED
- Moon test: 491 tests passed, 0 failed
- Moon clippy: PASSED
- Cargo fmt: PASSED
- Import tests exist in persistence.rs

## Contract Compliance

| Requirement | Status |
|-------------|--------|
| Export diagram as JSON | ✅ Implemented |
| Import JSON to update diagram | ✅ Implemented |
| Auto-save to localStorage on changes | ❌ Not implemented |
| Invalid import protection | ✅ Implemented |
| Schema versioning | ❌ Not implemented |
