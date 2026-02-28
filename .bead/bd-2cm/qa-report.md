# QA Report - bd-2cm

## Scope
- Bead: `bd-2cm`
- Title: `storage-sync: add atomic redb-plus-file persistence`
- Workspace: `/home/lewis/src/bd-2cm`

## Test Execution Evidence

### 1) `moon run :check`
- Command: `moon run :check`
- Exit code: `0`
- Expected: project checks succeed
- Actual: succeeded
- Key stdout:
  - `Finished dev profile`
  - `Tasks: 1 completed`
  - `EXIT_CODE:0`

### 2) `moon run :clippy`
- Command: `moon run :clippy`
- Exit code: `0`
- Expected: lint passes with denied warnings
- Actual: succeeded
- Key stdout:
  - `Finished dev profile`
  - `Tasks: 1 completed`
  - `EXIT_CODE:0`

### 3) `moon run :test-rust`
- Command: `moon run :test-rust`
- Exit code: `0`
- Expected: all Rust tests pass
- Actual: succeeded
- Key stdout:
  - `running 440 tests`
  - `test result: ok. 440 passed; 0 failed`
  - `running 8 tests`
  - `test result: ok. 8 passed; 0 failed`
  - `EXIT_CODE:0`

### 4) Atomic save verification (`save_workspace_atomic` path)
- Reproduction steps:
  1. Create test document at `.bead/bd-2cm/qa-fixtures/input.json`
  2. Run CLI layout command (calls `save_workspace_atomic`)
  3. Verify output file exists and JSON parses
- Command:
  - `./target/debug/diagram_tool layout --input .bead/bd-2cm/qa-fixtures/input.json --output .bead/bd-2cm/qa-fixtures/layout-output.json`
- Exit code: `0`
- Expected: persisted stage event, output file exists, valid JSON
- Actual: matched expectation
- Key stdout:
  - `{"event":"stage","name":"persisted","details":{"path":".bead/bd-2cm/qa-fixtures/layout-output.json","bytes_written":1537}}`
  - `EXIT_CODE:0`
- Verification command:
  - `python` JSON parse check for output file
- Verification output:
  - `exists: True`
  - `json_valid: True`
  - `top_keys: ['document', 'editor_state', 'revision', 'version']`
  - `has_temp_files: False`

### 5) JSONL event format verification
- Reproduction steps:
  1. Run validate command on test document
  2. Capture stdout/stderr
  3. Parse each stdout line as JSON
- Command:
  - `./target/debug/diagram_tool validate --input .bead/bd-2cm/qa-fixtures/input.json > .bead/bd-2cm/qa-fixtures/validate.stdout 2> .bead/bd-2cm/qa-fixtures/validate.stderr`
- Exit code: `0`
- Expected: JSONL events, one valid JSON object per line
- Actual: matched expectation
- Captured stdout (`.bead/bd-2cm/qa-fixtures/validate.stdout`):
  - `{"event":"start","command":"validate","ok":true,"code":"start","message":null}`
  - `{"event":"stage","name":"validating","details":{"path":".bead/bd-2cm/qa-fixtures/input.json"}}`
  - `{"event":"stage","name":"loaded","details":{"path":".bead/bd-2cm/qa-fixtures/input.json","fallback_used":false}}`
  - `{"event":"finish","command":"validate","ok":true,"code":"ok","message":null}`
- Captured stderr: empty
- JSONL verification command output:
  - `line_count: 4`
  - `all_lines_valid_json: True`
  - `single_line_objects: True`

### 6) LKG fallback verification
- Reproduction steps:
  1. Create invalid primary `.bead/bd-2cm/qa-fixtures/bad.json`
  2. Create valid fallback `.bead/bd-2cm/qa-fixtures/bad.json.lkg`
  3. Run validate against bad primary
- Command:
  - `./target/debug/diagram_tool validate --input .bead/bd-2cm/qa-fixtures/bad.json > .bead/bd-2cm/qa-fixtures/lkg.stdout 2> .bead/bd-2cm/qa-fixtures/lkg.stderr`
- Exit code: `0`
- Expected: primary load fails, fallback succeeds with `fallback_used:true`
- Actual: matched expectation
- Key stdout (`.bead/bd-2cm/qa-fixtures/lkg.stdout`):
  - `{"event":"stage","name":"validating","details":{"path":".bead/bd-2cm/qa-fixtures/bad.json","code":"validation_failed","message":"Failed to parse document: missing field \`revision\` at line 1 column 13"}}`
  - `{"event":"stage","name":"loaded","details":{"path":".bead/bd-2cm/qa-fixtures/bad.json.lkg","fallback_used":true}}`
  - `{"event":"finish","command":"validate","ok":true,"code":"ok","message":null}`
- Captured stderr: empty

## Issues Found
- None blocking.
- No crashes, no panics, no non-actionable errors observed in executed scope.

## Verdict
- **PASS** for requested QA scope.
- Recommendation: safe to proceed for bead `bd-2cm` based on executed checks above.
