# Nushell Command Runner Contract Bundle

## Scope Map

| Attribute | Value |
|---|---|
| Bead ID | seshat-it8 |
| Title | test bead for nushell 2 |
| Priority | P4 |
| Type | task |
| Domain | CLI / Shell Integration |
| Component | nu_runner |

## Contract Clauses

### Preconditions

| ID | Clause | Enforcement |
|---|---|---|
| P1 | Command string must be non-empty | Runtime: `NuCommand::new()` returns `Result` |
| P2 | Working directory must exist | Runtime: `std::fs::read_dir` check |
| P3 | Environment variables valid UTF-8 | Compile-time: `HashMap<String, String>` |

### Postconditions

| ID | Clause | Verification |
|---|---|---|
| Q1 | Success returns stdout in NuOutput | Unit test |
| Q2 | Exit code reflected in output | Unit test |
| Q3 | Timeout enforced (30s default) | Integration test |
| Q4 | No file descriptor leaks | Integration test |

### Invariants

| ID | Clause |
|---|---|
| I1 | NuRunner instance is reusable |
| I2 | One command at a time per instance |
| I3 | Environment vars persist for next command (scoped to runner instance) |

## Error Taxonomy

| Error Variant | Condition | Recovery |
|---|---|---|
| `NuError::InvalidCommand(String)` | Empty or whitespace-only command | Retry with valid command |
| `NuError::WorkingDirectoryNotFound(PathBuf)` | cwd does not exist | Provide valid path |
| `NuError::Timeout { command, duration_ms }` | Command exceeds timeout | Increase timeout or optimize |
| `NuError::ExecutableNotFound` | nushell not in PATH | Install nushell |
| `NuError::CommandFailed { code, stderr }` | Non-zero exit code | Inspect stderr |
| `NuError::IoError(String)` | Permission denied, etc. | Fix permissions |

## Traceability Matrix

| Contract Clause | Test ID | Test Name |
|---|---|---|
| P1 | TC-001 | test_returns_error_for_empty_or_whitespace_command |
| P2 | TC-002 | test_returns_error_when_working_directory_not_found |
| Q1 | TC-003 | test_execute_simple_echo_command_returns_output |
| Q2 | TC-004 | test_returns_error_on_command_failure |
| Q3 | TC-005 | test_returns_error_when_command_times_out |
| I1 | TC-006 | test_runner_reuses_for_sequential_commands |
| I3 | TC-007 | test_env_vars_persist_across_commands |

## Evaluation Protocol

1. **Compilation**: Code must compile without errors
2. **Static Analysis**: `cargo clippy` must pass with no warnings
3. **Unit Tests**: All contract verification tests must pass
4. **Integration Tests**: Timeout and resource leak tests must pass
5. **Property Tests**: Generated commands must satisfy preconditions
6. **E2E Tests**: Real Nushell binary execution tests must pass

## Violation Test Parity

| Violation Example | Test Name | Status |
|---|---|---|
| `execute("")` -> Err(InvalidCommand) | test_violates_p1_empty_command | Required |
| `execute("   ")` -> Err(InvalidCommand) | test_violates_p1_whitespace_command | Required |
| cwd("/nonexistent") -> Err(WorkingDirNotFound) | test_violates_p2_nonexistent_cwd | Required |
| exit 42 -> exit_code = 42 | test_violates_q2_exit_code | Required |
| timeout(1ms) sleep(10) -> Err(Timeout) | test_violates_q3_timeout | Required |

## DSL Specification Layer

### Test Builder DSL

```rust
// Domain-specific language for executable specifications
NuRunner::new()                    // Create runner
    .with_env("KEY", "value")      // Add environment variable
    .with_cwd("/path")             // Set working directory
    .with_timeout(Duration)        // Set timeout
    .execute("echo hello")         // Execute command
    .await                         // Await result
```

### Scenario Tags

| Tag | Meaning | Example |
|---|---|---|
| @e2e | End-to-end test with real binary | `@e2e test_nushell_binary_actually_executes` |
| @integration | Integration with OS | `@integration test_file_descriptor_leak` |
| @property | Property-based test | `@property test_env_vars_always_captured` |
| @fuzz | Fuzz test | `@fuzz test_arbitrary_command_strings` |
