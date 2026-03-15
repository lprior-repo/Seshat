use std::path::Path;

use crate::execution::execute_command_internal;
use crate::types::{
    find_nushell, validate_working_directory, CheckedCommand, Error, NuResult, DEFAULT_TIMEOUT_SECS,
};

pub fn execute_nu_command(command: &str) -> Result<NuResult, Error> {
    let _ = CheckedCommand::new(command)?;
    let nu_path = find_nushell()?;
    execute_command_internal(&nu_path, command, None, Some(DEFAULT_TIMEOUT_SECS))
}

pub fn execute_with_timeout(command: &str, timeout_secs: u64) -> Result<NuResult, Error> {
    let _ = CheckedCommand::new(command)?;
    let nu_path = find_nushell()?;
    execute_command_internal(&nu_path, command, None, Some(timeout_secs))
}

pub fn execute_in_directory(command: &str, cwd: &Path) -> Result<NuResult, Error> {
    let _ = CheckedCommand::new(command)?;
    validate_working_directory(cwd)?;
    let nu_path = find_nushell()?;
    execute_command_internal(&nu_path, command, Some(cwd), Some(DEFAULT_TIMEOUT_SECS))
}
