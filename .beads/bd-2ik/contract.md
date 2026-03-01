# Contract: bd-2ik - json-roundtrip

bead_id: bd-2ik
bead_title: json-roundtrip: implement schema-valid json import and export pipeline
phase: p0
updated_at: 2026-03-01T19:10:00Z

## Overview

Implement schema-valid JSON import and export pipeline.

## Preconditions

- SQLite connection is open with WAL enabled and synchronous FULL
- Rust Contract Signature: `fn export_diagram_json(conn: &Connection) -> Result<DiagramJsonExport, ExportError>`
- Rust Error Contract: `enum ExportError { InvalidSchema, Serialization, Sqlite, Validation }`

## Postconditions

- Rust Postcondition Signature: `fn import_diagram_json(conn: &mut Connection, input: &str, actor: Author) -> Result<ImportResult, ExportError>`
- Accepted operations increment revision monotonically by exactly one
- Rejected operations return structured error codes without side effects

## Invariants

- Event log remains append-only and replay deterministic
- Idempotent operation IDs never produce duplicate durable mutations
- Human-authored operations keep priority over conflicting AI operations

## Implementation Tasks

### Phase 2: Implementation
- Export current projection as schema-valid JSON plus optional event bundle
- Import JSON by generating canonical events that replay to equivalent projection

### Phase 4: Verification
- Run moon run :ci
