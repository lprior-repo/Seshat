# TEST PROTECTION PROMPT

## Protected Test Files (NEVER overwrite)

Before modifying, deleting, or overwriting ANY test file, you MUST verify the existence of these contract tests. These are the authoritative test suites for their respective beads and MUST NOT be overwritten.

### 1. IO-001 to IO-015: Import/Export/Persistence Tests
- **File**: `diagram_tool/src/models/io_tests.rs`
- **Bead**: seshat-4uc
- **Tests**: IO-001 through IO-015 (15 test categories)
- **Purpose**: Contract tests for JSON import/export and persistence
- **Created**: 2026-03-06

### 2. Test Infrastructure Tests
- **File**: `diagram_tool/src/test_infrastructure_tests.rs`
- **Bead**: seshat-wcb
- **Tests**: P1-P4, Q1-Q3, plus helper function tests
- **Purpose**: Tests the test harness itself
- **Created**: 2026-03-06

### 3. Geometry Tests (GEO-001 to GEO-030)
- **File**: `diagram_tool/src/geometry/mod.rs` (tests section)
- **Bead**: seshat-pnn
- **Tests**: GEO-001 through GEO-030 (30 test categories, 225+ individual tests)
- **Purpose**: Contract tests for geometry math operations
- **Status**: Already implemented, protected from deletion

## Verification Checklist

Before ANY modification to test code, RUN THIS CHECK:

```bash
# Verify IO tests exist and have content
test -f diagram_tool/src/models/io_tests.rs && wc -l diagram_tool/src/models/io_tests.rs

# Verify test infrastructure tests exist
test -f diagram_tool/src/test_infrastructure_tests.rs && wc -l diagram_tool/src/test_infrastructure_tests.rs

# Verify geometry tests exist (look for GEO markers)
grep -c "GEO-0" diagram_tool/src/geometry/mod.rs
```

## Rule: DO NOT OVERWRITE

If a test file above exists, DO NOT:
- Delete it
- Replace its contents
- Merge it into another file
- "Clean up" or "refactor" it without explicit user permission

If you need to ADD new tests, APPEND them or create a NEW test module.

## Violation Consequence

Overwriting these contract tests invalidates the bead contracts and breaks the quality gate. ALWAYS check with the user before modifying protected test files.

## Protected Markers

Each protected test file contains this marker at the top:
```rust
//! Import/Export/Persistence Tests (IO-001 to IO-015)
//!
//! This module contains comprehensive tests for JSON import/export
//! and persistence operations per contract bd-19p.
```

When you see these markers, the file is PROTECTED.
