# Martin Fowler Test Plan

## Happy Path Tests

- `test_execute_simple_echo_command_returns_output`
  Given: A valid NuRunner instance
  When: Executing command "echo hello"
  Then: Returns Ok(NuOutput) with stdout containing "hello"

- `test_execute_command_with_spaces_in_argument`
  Given: A valid NuRunner instance
  When: Executing command "echo 'hello world'"
  Then: Returns Ok(NuOutput) with stdout containing "hello world"

- `test_environment_variables_passed_to_command`
  Given: A NuRunner with env {"TEST_VAR": "test_value"}
  When: Executing command "echo $TEST_VAR"
  Then: Returns Ok(NuOutput) with stdout containing "test_value"

- `test_working_directory_respected`
  Given: A NuRunner with cwd set to temp directory
  When: Executing command "pwd"
  Then: Returns Ok(NuOutput) with stdout matching the cwd

## Error Path Tests

- `test_returns_error_for_empty_or_whitespace_command`
  Given: A valid NuRunner instance
  When: Executing empty string "" or whitespace "   "
  Then: Returns Err(NuError::InvalidCommand(...)) for both cases

- `test_returns_error_when_working_directory_not_found`
  Given: A NuRunner with invalid cwd
  When: Executing any command
  Then: Returns Err(NuError::WorkingDirectoryNotFound(...))

- `test_returns_error_on_command_failure`
  Given: A valid NuRunner instance
  When: Executing "exit 1"
  Then: Returns Ok(NuOutput) with exit_code = 1

- `test_returns_error_when_command_times_out`
  Given: A NuRunner with short timeout
  When: Executing "sleep 60"
  Then: Returns Err(NuError::Timeout{...})

## Edge Case Tests

- `test_handles_unicode_in_command_output`
  Given: A valid NuRunner instance
  When: Executing "echo 'Hello 世界'"
  Then: Returns Ok(NuOutput) with correct unicode in stdout

- `test_handles_large_output_gracefully`
  Given: A valid NuRunner instance
  When: Executing command that produces 1MB output
  Then: Returns Ok(NuOutput) with complete stdout (no truncation)

- `test_handles_stderr_separately`
  Given: A valid NuRunner instance
  When: Executing "echo error 1>&2"
  Then: Returns Ok(NuOutput) with empty stdout and stderr containing "error"

- `test_timeout_zero_is_rejected`
  Given: A NuRunner with timeout(Duration::ZERO)
  When: Executing any command
  Then: Returns Err - timeout must be > 0

## Invariant Tests

- `test_runner_remains_valid_after_execution`
  Given: A NuRunner instance
  When: Executing "echo first" successfully
  Then: The runner remains valid and can execute subsequent commands

- `test_env_vars_captured_in_first_command`
  Given: A NuRunner with env {"PERSIST_VAR": "persistent_value"}
  When: Executing "echo $PERSIST_VAR"
  Then: First command returns output with "persistent_value"

- `test_env_vars_available_in_subsequent_commands`
  Given: A NuRunner with env {"PERSIST_VAR": "persistent_value"} after first command
  When: Executing another command "echo $PERSIST_VAR"
  Then: Subsequent command also returns output with "persistent_value"

## End-to-End Tests (@e2e)

- `test_nushell_binary_actually_executes`
  Given: Nushell is installed on the system
  When: Executing "nu --version"
  Then: Returns Ok(NuOutput) with version string in stdout

- `test_full_pipeline_from_command_creation_to_output`
  Given: A NuRunner configured with env and cwd
  When: Executing a complex pipeline "echo (1 + 2)"
  Then: Returns Ok(NuOutput) with "3" in stdout

## Integration Tests (@integration)

- `test_no_file_descriptor_leaks`
  Given: A NuRunner executing multiple commands
  When: Executing 100 sequential commands
  Then: All commands complete successfully
  And: No file descriptor errors occur

- `test_timeout_actually_terminates_command`
  Given: A NuRunner with 100ms timeout
  When: Executing "sleep 10"
  Then: Command is terminated within 500ms of timeout

## Property-Based Tests (@property)

- `test_env_vars_always_captured`
  Given: Arbitrary valid env map
  When: Executing "echo $KEY" where KEY is in env
  Then: Output contains the env value

- `test_exit_code_always_reflected`
  Given: Arbitrary exit code 0-255
  When: Executing "exit $CODE"
  Then: Output.exit_code equals the code

- `test_cwd_always_respected`
  Given: Arbitrary valid directory
  When: Executing "pwd"
  Then: Output matches the configured directory

## Fuzz Tests (@fuzz)

- `test_arbitrary_command_strings`
  Given: Randomly generated command strings
  When: Executing via NuRunner
  Then: Either returns Ok or returns a specific error (no panics)

- `test_malformed_input_handled_gracefully`
  Given: Random bytes or malformed UTF-8 in command
  When: Attempting to execute
  Then: Returns Err with specific error variant (no panics)

## Contract Verification Tests

- **test_precondition_p1_command_not_empty**
  Given: An empty command string
  When: Passed to NuCommand::new()
  Then: Returns Err(NuError::InvalidCommand)

- **test_precondition_p2_working_directory_valid**
  Given: A PathBuf to nonexistent directory
  When: Used as cwd in NuRunner
  Then: Execute returns Err(NuError::WorkingDirectoryNotFound)

- **test_postcondition_q1_stdout_captured_on_success**
  Given: A valid command that outputs to stdout
  When: Executed via NuRunner
  Then: NuOutput.stdout contains the command output

- **test_postcondition_q2_exit_code_reflected**
  Given: A command that exits with code 42
  When: Executed via NuRunner
  Then: NuOutput.exit_code equals 42

- **test_postcondition_q3_timeout_enforced**
  Given: A long-running command with short timeout
  When: Executed via NuRunner
  Then: Returns Err(NuError::Timeout{...})

- **test_invariant_i1_runner_reusable**
  Given: A NuRunner instance
  When: Executing multiple commands sequentially
  Then: All commands execute successfully (no state corruption)

- **test_invariant_i3_env_vars_available_for_runner_lifetime**
  Given: A NuRunner with env set
  When: Executing multiple commands
  Then: All commands see the same environment variables

## Contract Violation Tests

- **test_violates_p1_empty_command_returns_invalid_command_error**
  Given: NuRunner instance
  When: execute("") is called
  Then: Returns Err(NuError::InvalidCommand("".to_string()))

- **test_violates_p1_whitespace_command_returns_invalid_command_error**
  Given: NuRunner instance
  When: execute("   ") is called
  Then: Returns Err(NuError::InvalidCommand("   ".to_string()))

- **test_violates_p2_nonexistent_cwd_returns_working_directory_not_found**
  Given: NuRunner with cwd set to "/nonexistent/path"
  When: execute("echo test") is called
  Then: Returns Err(NuError::WorkingDirectoryNotFound(PathBuf))

- **test_violates_q2_exit_code_matches_shell_exit_status**
  Given: NuRunner instance
  When: execute("exit 42") is called
  Then: Returns Ok(NuOutput { exit_code: 42, ... })

- **test_violates_q3_timeout_fires_after_duration**
  Given: NuRunner with timeout(1ms)
  When: execute("sleep 10") is called
  Then: Returns Err(NuError::Timeout { command: "sleep 10", duration_ms: 1 })

## Given-When-Then Scenarios (DSL Executable)

### Scenario 1: Execute Basic Nushell Command
Given: A newly constructed NuRunner with default configuration
When: I execute "echo 'Hello from Nushell'"
Then:
- The result is Ok(NuOutput)
- The stdout contains "Hello from Nushell"
- The exit_code is 0

### Scenario 2: Execute Failing Command
Given: A newly constructed NuRunner
When: I execute "false" (which exits with code 1)
Then:
- The result is Ok(NuOutput)
- The exit_code is 1

### Scenario 3: Custom Environment
Given: A NuRunner configured with env {"MY_VAR": "custom_value"}
When: I execute "echo $MY_VAR"
Then:
- The stdout contains "custom_value"

### Scenario 4: Timeout Handling
Given: A NuRunner with timeout(Duration::from_secs(1))
When: I execute "sleep 60"
Then:
- The result is Err(NuError::Timeout{...})

### Scenario 5: Working Directory Control
Given: A NuRunner with cwd set to a known temp directory
When: I execute "pwd"
Then:
- The stdout matches the configured working directory

### Scenario 6: Environment Variable Persistence
Given: A NuRunner configured with env {"SCOPED_VAR": "value1"}
When: I execute "echo $SCOPED_VAR" followed by "echo $SCOPED_VAR"
Then:
- Both commands return "value1" in stdout

### Scenario 7: End-to-End Real Binary
Given: Nushell is installed on the system
When: I execute "nu -c 'echo (2 + 2)'"
Then:
- The result is Ok(NuOutput)
- The stdout contains "4"
