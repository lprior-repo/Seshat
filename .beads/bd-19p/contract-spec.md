# Contract Specification: bd-19p Import/Export/Persistence

## Meta
- **Bead ID**: bd-19p
- **Title**: import-export: Fix import/export and persistence (IO-001 to IO-015)
- **Priority**: P2
- **Type**: feature
- **Created**: 2026-03-03
- **Author**: Claude

## Overview

This contract specifies the requirements for reliable JSON import/export and persistence operations in the Seshat diagram tool. The system must handle all edge cases gracefully with zero panics or unwraps.

## Contract: ImportExportManager

### Preconditions

| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | Input JSON is valid UTF-8 | Runtime check | ExportError::Serialization |
| P2 | Schema version is supported (<= current) | Runtime check | ExportError::InvalidSchema |
| P3 | Database connection is valid | Runtime check | ExportError::Sqlite |
| P4 | File path is writable (for save) | Runtime check | CliPersistenceError::IoError |
| P5 | File exists (for load) | Runtime check | CliPersistenceError::IoError |

### Postconditions

| # | Guarantee | Verification |
|---|-----------|--------------|
| Q1 | Export produces valid JSON | Round-trip through serde_json |
| Q2 | Export validates against schema | validate_schema() returns Ok |
| Q3 | Import creates valid document state | verify_invariants() passes |
| Q4 | Atomic save preserves original on failure | Original file unchanged after failed write |
| Q5 | LKG fallback recovers from corrupted primary | load_workspace_with_lkg() returns valid doc |

### Invariants

| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | No partial writes on crash | Atomic write pattern | Never |
| I2 | Export JSON is canonical | to_canonical_pretty_json() | Never |
| I3 | Import is idempotent | Same input = same state | Never |
| I4 | Unicode roundtrips correctly | UTF-8 everywhere | Never |

## Test Cases: IO-001 to IO-015

### IO-001: Malformed JSON Import
- **Given**: JSON input with syntax errors
- **When**: import_diagram_json() is called
- **Then**: Returns ExportError::Serialization with specific error message
- **No panic, no unwrap**

### IO-002: Empty Document Export
- **Given**: Empty database (no events)
- **When**: export_diagram_json() is called
- **Then**: Returns valid export with revision=0, empty nodes/edges

### IO-003: Invalid Schema Version
- **Given**: JSON with version > current supported version
- **When**: import_diagram_json() is called
- **Then**: Returns ExportError::InvalidSchema

### IO-004: Valid Round-Trip
- **Given**: Document with nodes and edges
- **When**: Export to JSON, then import to fresh database
- **Then**: Imported document equals original

### IO-005: Large Document Export Performance
- **Given**: Document with 1000+ nodes
- **When**: export_projection_json() is called
- **Then**: Completes within 5 seconds

### IO-006: Large Document Import Performance
- **Given**: Export with 100+ events
- **When**: import_diagram_json() is called
- **Then**: All events replay, import completes

### IO-007: Unicode Node Labels
- **Given**: Document with emoji/RTL/unicode labels
- **When**: Export and re-import
- **Then**: Labels preserved exactly

### IO-008: Atomic Save on Crash
- **Given**: Original file exists
- **When**: Save operation interrupted (simulated)
- **Then**: Original file unchanged, no temp files left

### IO-009: LKG Fallback
- **Given**: Corrupted primary file, valid .lkg file
- **When**: load_workspace_with_lkg() is called
- **Then**: Returns valid document from LKG

### IO-010: Schema Validation on Import
- **Given**: JSON that violates schema (negative dimensions, orphan edges)
- **When**: validate_export_schema() is called
- **Then**: Returns ExportError::InvalidSchema

### IO-011: Recovery Mode Export
- **Given**: Database opened in read-only recovery mode
- **When**: export_while_recovering() is called
- **Then**: Returns valid JSON (read operations only)

### IO-012: Version Backward Compatibility
- **Given**: Export with version 1 (older)
- **When**: validate_export_schema() is called
- **Then**: Accepts and processes successfully

### IO-013: Null in Required Field
- **Given**: JSON with null where string expected
- **When**: import_diagram_json() is called
- **Then**: Returns ExportError::Serialization

### IO-014: Truncated JSON
- **Given**: JSON cut off mid-string
- **When**: import_diagram_json() is called
- **Then**: Returns ExportError::Serialization (not panic)

### IO-015: Missing Required Field
- **Given**: JSON missing required field (version)
- **When**: import_diagram_json() is called
- **Then**: Returns ExportError::Serialization

## Implementation Requirements

### Error Handling
- All IO functions must return Result<T, Error>
- No unwrap() or expect() allowed in production code
- All errors must be specific and actionable

### File Operations
- Use atomic write pattern: write to temp, fsync, rename
- Clean up temp files on failure
- Support LKG fallback for recovery

### JSON Handling
- Use canonical JSON format for reproducibility
- Validate schema before persisting
- Handle all serde_json errors gracefully

## Acceptance Criteria

1. All 15 IO test cases pass
2. Zero clippy warnings for unwrap_used, expect_used, panic
3. Code coverage >= 90% for export.rs and cli_persistence.rs
4. All existing tests continue to pass
