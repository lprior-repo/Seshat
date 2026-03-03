# Contract Specification: bd-2wx Cross-Platform Compatibility

## Overview

Bead ID: `bd-2wx`
Feature: Cross-platform compatibility for Windows, macOS, and Linux

This contract specifies the requirements for ensuring the diagram_tool application
operates correctly across Windows, macOS, and Linux platforms.

## Platform Support Matrix

| Platform | Architecture | Support Level |
|----------|-------------|---------------|
| Windows  | x86_64      | Full          |
| macOS    | x86_64      | Full          |
| macOS    | aarch64     | Full          |
| Linux    | x86_64      | Full          |
| Web      | wasm32      | Partial       |

## Contract Requirements

### CR-1: Path Handling
**Requirement**: All file system paths must be handled using platform-agnostic APIs.

**Acceptance Criteria**:
- Use `std::path::Path` and `std::path::PathBuf` for all path operations
- No hardcoded path separators (`/` or `\`)
- No hardcoded platform-specific paths (e.g., `C:\`, `/home/`)
- Path concatenation uses `.join()` method

**Verification**:
- Grep for string concatenation with path separators
- Verify all file operations use `Path`/`PathBuf`

### CR-2: Line Endings
**Requirement**: File import/export must handle platform line endings correctly.

**Acceptance Criteria**:
- Import handles both LF (Unix) and CRLF (Windows) line endings
- Export uses consistent line endings (LF recommended)
- JSON serialization is line-ending agnostic

**Verification**:
- Test import with files containing CRLF
- Test import with files containing LF
- Verify exported files are consistent

### CR-3: File System Operations
**Requirement**: File operations must work correctly on all supported platforms.

**Acceptance Criteria**:
- Atomic writes use temp file + rename pattern
- File locking works on platforms that support it
- Temporary directory resolution is platform-aware
- Home directory resolution is platform-aware

**Verification**:
- Verify use of `tempfile` crate for temp directories
- Verify atomic write pattern in `cli_persistence.rs`
- Test file operations on all platforms

### CR-4: Platform-Specific Dependencies
**Requirement**: Platform-specific dependencies must be properly isolated.

**Acceptance Criteria**:
- WASM-specific dependencies under `cfg(target_arch = "wasm32")`
- Desktop-specific dependencies under `cfg(not(target_arch = "wasm32"))`
- Conditional compilation for platform-specific code
- No runtime panics due to missing platform-specific features

**Verification**:
- Review `Cargo.toml` target-specific dependencies
- Verify `cfg` attributes on platform-specific code
- Test build on all target platforms

### CR-5: SQLite Storage
**Requirement**: SQLite storage must work on all platforms.

**Acceptance Criteria**:
- Use bundled SQLite on desktop platforms
- Handle WASM SQLite limitations gracefully
- Database files are platform-independent

**Verification**:
- Verify rusqlite bundled feature on non-WASM
- Test database operations on all platforms

### CR-6: File Dialogs
**Requirement**: Native file dialogs must work on desktop platforms.

**Acceptance Criteria**:
- Use `rfd` (Rust File Dialog) for native dialogs
- Graceful fallback for platforms without native dialogs
- Dialog behavior is consistent across platforms

**Verification**:
- Verify `rfd` dependency in `Cargo.toml`
- Test file open/save dialogs on desktop platforms

### CR-7: File Watching
**Requirement**: File watching must work on desktop platforms.

**Acceptance Criteria**:
- Use `notify` crate for file system watching
- Handle platforms without file watching support
- No panics when file watching is unavailable

**Verification**:
- Verify `notify` dependency in `Cargo.toml`
- Test file watching functionality

### CR-8: Functional Rust Compliance
**Requirement**: Source code must comply with functional Rust standards.

**Acceptance Criteria**:
- Zero `unwrap()` in production source code (tests excluded)
- Zero `panic!` in production source code (test assertions excluded)
- Zero `todo!` or `unimplemented!` macros
- `#![deny(clippy::unwrap_used)]` present in all source modules
- `#![forbid(unsafe_code)]` present in all source modules

**Verification**:
- Grep for `unwrap()` in source (excluding tests/)
- Grep for `panic!` in source (excluding test assertions)
- Verify lints in `lib.rs` and `main.rs`

### CR-9: Build Verification
**Requirement**: Project must build successfully on all supported platforms.

**Acceptance Criteria**:
- `cargo build` succeeds on Windows
- `cargo build` succeeds on macOS
- `cargo build` succeeds on Linux
- `cargo build --target wasm32-unknown-unknown` succeeds for web target

**Verification**:
- CI builds on all platforms
- No platform-specific compile errors

### CR-10: Test Execution
**Requirement**: All tests must pass on all supported platforms.

**Acceptance Criteria**:
- `cargo test` passes on Windows
- `cargo test` passes on macOS
- `cargo test` passes on Linux
- No platform-specific test failures

**Verification**:
- CI test runs on all platforms
- Test results are consistent across platforms

## Out of Scope

- Mobile platforms (iOS, Android)
- 32-bit architectures
- BSD variants
- Real-time operating systems

## Dependencies

### Platform-Agnostic Dependencies
- `serde`, `serde_json` - Serialization
- `clap` - CLI argument parsing
- `anyhow`, `thiserror` - Error handling
- `uuid` - Unique identifiers
- `dioxus` - UI framework

### Desktop-Only Dependencies
- `rfd` - Native file dialogs
- `fs2` - File locking
- `notify` - File system watching
- `rusqlite` (bundled) - SQLite with bundled library

### WASM-Only Dependencies
- `getrandom` (wasm_js) - Random number generation
- `uuid` (js) - UUID generation with JS interop
- `rusqlite` (no bundled) - SQLite without bundled library

## Verification Evidence

All verification items are documented in:
- `martin-fowler-tests.md` - Test cases and scenarios
- `verification.md` - Verification results
- `receipts.jsonl` - Test execution receipts
