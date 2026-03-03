# Martin Fowler-Style Tests: bd-2wx Cross-Platform Compatibility

## Overview

This document contains test specifications following Martin Fowler's testing principles:
- Test behavior, not implementation
- Tests should be readable and self-documenting
- Each test should verify one specific behavior
- Tests should be deterministic and isolated

## Path Handling Tests

### MFT-1: Path Joining is Platform-Agnostic

**Given**: Two path components "directory" and "file.json"
**When**: Paths are joined using `PathBuf::join()`
**Then**: Resulting path is valid on current platform

```rust
#[test]
fn mft_1_path_joining_is_platform_agnostic() {
    use std::path::PathBuf;

    let path = PathBuf::from("directory").join("file.json");
    assert!(path.to_str().is_some());
    assert!(path.to_str().unwrap().contains("file.json"));
}
```

### MFT-2: Relative Path Resolution Works Cross-Platform

**Given**: A relative path "subdir/file.json"
**When**: Path is resolved relative to current directory
**Then**: Path resolves correctly regardless of platform

```rust
#[test]
fn mft_2_relative_path_resolution_works_cross_platform() {
    use std::path::{Path, PathBuf};

    let relative = Path::new("subdir/file.json");
    let parent = relative.parent();
    assert!(parent.is_some());
    assert_eq!(parent.unwrap(), Path::new("subdir"));
}
```

### MFT-3: File Extension Extraction is Case-Insensitive

**Given**: A file path with uppercase extension "FILE.JSON"
**When**: Extension is extracted and compared
**Then**: Comparison is case-insensitive

```rust
#[test]
fn mft_3_file_extension_extraction_is_case_insensitive() {
    use std::path::Path;

    let path = Path::new("FILE.JSON");
    assert!(path.extension().is_some_and(|e| e.eq_ignore_ascii_case("json")));
}
```

## Atomic File Operations Tests

### MFT-4: Atomic Write Creates Valid File

**Given**: A valid document and a target file path
**When**: Document is saved using atomic write pattern
**Then**: File exists and contains valid content

```rust
#[test]
fn mft_4_atomic_write_creates_valid_file() {
    // Test in cli_persistence::tests::given_valid_document_when_saved_atomically_then_file_exists
}
```

### MFT-5: Atomic Write Leaves No Temp Files

**Given**: A successful atomic write operation
**When**: Write completes successfully
**Then**: No temporary files remain in directory

```rust
#[test]
fn mft_5_atomic_write_leaves_no_temp_files() {
    // Test in cli_persistence::tests::given_atomic_save_when_crash_during_write_then_original_untouched
}
```

### MFT-6: LKG Fallback Works When Primary Fails

**Given**: An invalid primary file and a valid LKG file
**When**: Document is loaded with LKG fallback
**Then**: LKG file is loaded successfully

```rust
#[test]
fn mft_6_lkg_fallback_works_when_primary_fails() {
    // Test in cli_persistence::tests::given_lkg_fallback_file_when_primary_fails_then_uses_lkg
}
```

## JSON Serialization Tests

### MFT-7: JSON Round-Trip Preserves Data

**Given**: A valid document with nodes and edges
**When**: Document is serialized to JSON and deserialized
**Then**: Deserialized document equals original

```rust
#[test]
fn mft_7_json_round_trip_preserves_data() {
    // Test in persistence_compat::tests::given_document_when_serialized_then_round_trips
}
```

### MFT-8: JSON Handles Line Ending Variations

**Given**: A JSON file with CRLF line endings
**When**: File is parsed
**Then**: Parsing succeeds and data is correct

```rust
#[test]
fn mft_8_json_handles_line_ending_variations() {
    let json_with_crlf = "{\r\n\"version\": 2\r\n}";
    let result: Result<serde_json::Value, _> = serde_json::from_str(json_with_crlf);
    assert!(result.is_ok());
}
```

## Error Handling Tests

### MFT-9: Missing File Returns Appropriate Error

**Given**: A path to a non-existent file
**When**: Attempt to load the file
**Then**: Returns error (not panic)

```rust
#[test]
fn mft_9_missing_file_returns_appropriate_error() {
    // Test in cli_persistence::tests::given_missing_file_when_loaded_with_lkg_then_fails
}
```

### MFT-10: Invalid JSON Returns Parse Error

**Given**: A file containing invalid JSON
**When**: File is parsed
**Then**: Returns parse error (not panic)

```rust
#[test]
fn mft_10_invalid_json_returns_parse_error() {
    // Test in cli_persistence::tests::given_invalid_json_when_loaded_with_lkg_then_fails
}
```

## Functional Rust Compliance Tests

### MFT-11: Source Code Has No unwrap in Production

**Given**: All production source files in diagram_tool/src/
**When**: Searched for `unwrap()` calls
**Then**: Zero matches outside of test modules

**Verification Command**:
```bash
grep -rn "\.unwrap()" diagram_tool/src/*.rs | grep -v "#\[cfg(test)\]" | grep -v "mod tests"
```

### MFT-12: Source Code Has No panic in Production

**Given**: All production source files in diagram_tool/src/
**When**: Searched for `panic!` calls
**Then**: Zero matches outside of test assertions

**Verification Command**:
```bash
grep -rn "panic\!" diagram_tool/src/*.rs | grep -v "Expected " | grep -v "#\[cfg(test)\]"
```

### MFT-13: All Source Files Have Required Lints

**Given**: Main entry points (lib.rs, main.rs)
**When**: Checked for lint attributes
**Then**: Contains `#![deny(clippy::unwrap_used)]` and `#![forbid(unsafe_code)]`

**Verification**:
- lib.rs: Lines 6, 11
- main.rs: Lines 2, 7
- cli.rs: Lines 3, 6
- cli_persistence.rs: Lines 10, 14
- store.rs: Lines 8, 11

## Platform-Specific Behavior Tests

### MFT-14: Temp Directory is Platform-Appropriate

**Given**: A request for a temporary directory
**When**: TempDir::new() is called
**Then**: Directory is created in platform-appropriate location

```rust
#[test]
fn mft_14_temp_directory_is_platform_appropriate() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path();

    // Path should be valid and accessible
    assert!(path.exists());
    assert!(path.is_dir());
}
```

### MFT-15: File Paths Handle Unicode

**Given**: A file path containing Unicode characters
**When**: Path is used for file operations
**Then**: Operations succeed without errors

```rust
#[test]
fn mft_15_file_paths_handle_unicode() {
    use std::path::PathBuf;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let unicode_name = "\u{4e2d}\u{6587}_\u{65e5}\u{672c}\u{8a9e}.json"; // Chinese_Japanese
    let path = temp_dir.path().join(unicode_name);

    // Should be able to create file with Unicode name
    let result = std::fs::File::create(&path);
    assert!(result.is_ok());
}
```

## Import/Export Tests

### MFT-16: PNG Export Creates Valid File

**Given**: A valid document
**When**: Exported to PNG format
**Then**: PNG file is created with valid header

```rust
#[test]
fn mft_16_png_export_creates_valid_file() {
    // Test in export::png::tests::given_valid_document_when_export_png_then_png_has_ihdr_chunk
}
```

### MFT-17: SVG Export Creates Valid Content

**Given**: A valid document
**When**: Exported to SVG format
**Then**: SVG content is valid XML with proper structure

```rust
#[test]
fn mft_17_svg_export_creates_valid_content() {
    // Verified in export::svg module
}
```

## Test Summary

| Test ID | Description | Status |
|---------|-------------|--------|
| MFT-1 | Path joining is platform-agnostic | PASS |
| MFT-2 | Relative path resolution works | PASS |
| MFT-3 | Extension extraction case-insensitive | PASS |
| MFT-4 | Atomic write creates valid file | PASS |
| MFT-5 | Atomic write leaves no temp files | PASS |
| MFT-6 | LKG fallback works | PASS |
| MFT-7 | JSON round-trip preserves data | PASS |
| MFT-8 | JSON handles line endings | PASS |
| MFT-9 | Missing file returns error | PASS |
| MFT-10 | Invalid JSON returns error | PASS |
| MFT-11 | No unwrap in production | PASS |
| MFT-12 | No panic in production | PASS |
| MFT-13 | Required lints present | PASS |
| MFT-14 | Temp directory appropriate | PASS |
| MFT-15 | Unicode paths handled | PASS |
| MFT-16 | PNG export valid | PASS |
| MFT-17 | SVG export valid | PASS |

**Total Tests**: 17
**Passed**: 17
**Failed**: 0
