#![allow(clippy::uninlined_format_args)]

use std::process::Command;

fn get_bin() -> String {
    env!("CARGO_BIN_EXE_seshat").to_string()
}

#[test]
fn main_returns_success_when_minimum_boundary() -> Result<(), String> {
    let output = Command::new(get_bin())
        .output()
        .map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}

#[test]
fn main_returns_error_when_one_below_minimum_boundary() -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    let output = Command::new(get_bin())
        .arg0("")
        .output()
        .map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn main_returns_error_when_underflow_potential_boundary() -> Result<(), String> {
    let output = Command::new(get_bin())
        .args(["complex-state", "--depth", "-2147483649"])
        .output()
        .map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn main_returns_success_when_valid_subcommand_is_executed() -> Result<(), String> {
    let output = Command::new(get_bin())
        .args(["valid-command"])
        .output()
        .map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}

#[test]
fn main_returns_success_when_version_flag_is_passed() -> Result<(), String> {
    let output = Command::new(get_bin())
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}

#[test]
fn main_returns_error_when_execute_fails() -> Result<(), String> {
    let output = Command::new(get_bin())
        .args(["simulate-failure"])
        .output()
        .map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

#[test]
fn main_returns_success_when_environment_limit_isolated_execution() -> Result<(), String> {
    let mut cmd = Command::new(get_bin());
    cmd.envs((0..10000).map(|i| (format!("VAR_{i}"), "value")));
    let output = cmd.output().map_err(|e| e.to_string())?;
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}
