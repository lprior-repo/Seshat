bead_id: bd-1u1
bead_title: tests: Implement IO import/export tests 2/2
phase: p0
updated_at: 2026-03-02T02:27:15Z

# Contract: IO Import/Export Tests (Part 2/2)

## Summary

Implement 5 IO tests covering export image bounds, export with rotated items, save/reopen geometry preservation, import large coordinates, and older version migration.

## Requirements

### Test 1: Export Image Bounds Match
- **Given**: A document with nodes at specific positions
- **When**: Exporting to PNG/SVG
- **Then**: The exported image bounds match the calculated document bounds (with margin)

**Acceptance Criteria**:
- PNG/SVG width and height reflect document extent plus margin
- viewBox (SVG) or dimensions (PNG) encompass all nodes
- Empty document uses default bounds (800x600)

### Test 2: Export with Rotated Items
- **Given**: A document containing nodes with rotation metadata
- **When**: Exporting to PNG/SVG
- **Then**: The export completes without crash and includes rotated items

**Acceptance Criteria**:
- Nodes with `metadata.rotation` field export correctly
- Rotation values are preserved in export (if applicable to format)
- No panic or error on rotated items

### Test 3: Save/Reopen Exact Geometry
- **Given**: A document with nodes at precise fractional coordinates
- **When**: Saving to JSON and reopening
- **Then**: All geometry values are exactly preserved

**Acceptance Criteria**:
- x, y, width, height values match exactly after round-trip
- OrderedFloat precision is maintained
- No floating-point drift or truncation

### Test 4: Import Large Coordinates No Float Crash
- **Given**: A JSON document with very large coordinate values (e.g., 1e15, 1e300)
- **When**: Importing the document
- **Then**: Import succeeds without floating-point overflow/crash

**Acceptance Criteria**:
- Large finite values (up to f64::MAX) parse without error
- No panic on extreme coordinate values
- System handles Infinity/NaN gracefully (if present)

### Test 5: Import Older Version Migration
- **Given**: A JSON document with `version: 1` (older schema)
- **When**: Importing the document
- **Then**: Document migrates to current version (version: 2)

**Acceptance Criteria**:
- Version 1 documents parse successfully
- Legacy field names are remapped correctly (via persistence_compat)
- After import, document is valid current version

## Test Location

Tests should be added to the existing test modules in:
- `/home/lewis/src/seshat/diagram_tool/src/export/svg.rs` (bounds, rotated items)
- `/home/lewis/src/seshat/diagram_tool/src/export/png.rs` (bounds, rotated items)
- `/home/lewis/src/seshat/diagram_tool/src/ui/toolbar/persistence.rs` (save/reopen)
- `/home/lewis/src/seshat/diagram_tool/src/ui/toolbar/persistence_compat.rs` (version migration)
- `/home/lewis/src/seshat/diagram_tool/src/geometry/mod.rs` (large coordinates, if applicable)

## Preconditions

- Existing export pipeline (SVG, PNG) is functional
- persistence_compat module handles legacy field remapping
- OrderedFloat type preserves f64 precision

## Postconditions

- All 5 tests pass
- No regression in existing tests
- `moon run :test` passes

## Invariants

- Tests follow Given/When/Then structure
- Each test is independent and isolated
- Tests use tempfile for file operations
- No clippy warnings in new code
