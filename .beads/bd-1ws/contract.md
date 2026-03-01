# Contract: bd-1ws - io-json-import

bead_id: bd-1ws
bead_title: io-json-import: import diagram json by generating canonical events
phase: p0
updated_at: 2026-03-01T19:20:00Z

## Overview

Import diagram JSON by generating canonical events that replay to the equivalent projection.

## Preconditions

- Valid JSON input matching diagram.schema.json
- Rust Contract Signature: `fn import_canonical_json(conn: &mut Connection, json: &str) -> Result<ImportResult, ExportError>`

## Postconditions

- Import generates canonical events
- Events can be replayed to reproduce the imported state

## Implementation Tasks

### Phase 2: Implementation
- Parse JSON into intermediate representation
- Generate canonical events from representation

### Phase 4: Verification
- Run moon run :ci
