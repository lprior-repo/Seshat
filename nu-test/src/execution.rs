use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::types::{Error, NuResult, DEFAULT_TIMEOUT_SECS};

pub fn execute_command_internal(
    nu_path: &str,
    command: &str,
    cwd: Option<&Path>,
    timeout_secs: Option<u64>,
) -> Result<NuResult, Error> {
    let timeout = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
    let start = Instant::now();

    let mut cmd = Command::new(nu_path);
    cmd.arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| Error::ExecutionFailed(e.to_string()))?;

    let timed_out = loop {
        match child.try_wait()? {
            Some(status) => {
                let exit_code = status.code().unwrap_or(-1);
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut s| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                let (stdout_valid, stderr_valid) = (
                    std::str::from_utf8(stdout.as_bytes()).is_ok(),
                    std::str::from_utf8(stderr.as_bytes()).is_ok(),
                );

                return Ok(NuResult {
                    stdout,
                    stdout_valid,
                    stderr,
                    stderr_valid,
                    exit_code,
                });
            }
            None => {
                if start.elapsed() > Duration::from_secs(timeout) {
                    let _ = child.kill();
                    let _ = child.wait();
                    break true;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    };

    if timed_out {
        Err(Error::ExecutionFailed(format!(
            "Command timed out after {} seconds",
            timeout
        )))
    } else {
        unreachable!()
    }
}
