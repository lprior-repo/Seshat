bead_id: bd-1u1
bead_title: tests: Implement IO import/export tests 2/2
phase: p2
updated_at: 2026-03-02T02:38:00Z

# Verification: IO Import/Export Tests (Part 2/2)

## Test Results

### Moon Test
```
moon run :test
```
Result: **PASS** - All 40 tests pass (13 e2e + 27 golden scenes)

### Cargo Test
```
cargo test -p diagram_tool
```
Result: **PASS** - 1088 unit tests pass

## New Tests Added

### IO-TEST-1: Export Image Bounds Match
| Test | Status |
|------|--------|
| `export::svg::io_tests::given_document_with_nodes_when_export_svg_then_bounds_match_with_margin` | PASS |
| `export::svg::io_tests::given_empty_document_when_export_svg_then_uses_default_bounds` | PASS |

### IO-TEST-2: Export with Rotated Items
| Test | Status |
|------|--------|
| `export::svg::io_tests::given_node_with_rotation_metadata_when_export_svg_then_succeeds` | PASS |
| `export::svg::io_tests::given_multiple_rotated_nodes_when_export_svg_then_succeeds` | PASS |
| `export::svg::io_tests::given_node_with_negative_rotation_when_export_svg_then_succeeds` | PASS |

### IO-TEST-3: Save/Reopen Exact Geometry
| Test | Status |
|------|--------|
| `ui::toolbar::persistence::tests::given_document_with_fractional_coords_when_round_trip_then_geometry_preserved` | PASS |
| `ui::toolbar::persistence::tests::given_document_with_various_precision_coords_when_round_trip_then_all_preserved` | PASS |

### IO-TEST-4: Import Large Coordinates No Float Crash
| Test | Status |
|------|--------|
| `ui::toolbar::persistence::tests::given_document_with_large_coordinates_when_import_then_succeeds` | PASS |
| `ui::toolbar::persistence::tests::given_document_with_extreme_finite_coords_when_import_then_succeeds` | PASS |

### IO-TEST-5: Import Older Version Migration
| Test | Status |
|------|--------|
| `ui::toolbar::persistence_compat::tests::given_version_1_document_when_import_then_migrates_to_current_version` | PASS |
| `ui::toolbar::persistence_compat::tests::given_older_document_with_legacy_fields_when_import_then_fields_remapped` | PASS |
| `ui::toolbar::persistence_compat::tests::given_document_without_version_when_import_then_fails_gracefully` | PASS |

## Summary

- Total new tests: 12
- All tests pass
- No regressions
- Contract requirements satisfied
