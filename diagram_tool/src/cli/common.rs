#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Serialize, Clone)]
pub struct CliEvent {
    pub event: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub success: bool,
}

impl CliEvent {
    pub fn start(name: String) -> Self {
        Self {
            event: String::from("cli_start"),
            name,
            code: None,
            message: None,
            success: true,
        }
    }

    pub fn finish(name: String, success: bool, code: String) -> Self {
        Self {
            event: String::from("cli_finish"),
            name,
            code: Some(code),
            message: None,
            success,
        }
    }

    pub fn error(name: String, code: String, message: String) -> Self {
        Self {
            event: String::from("cli_error"),
            name,
            code: Some(code),
            message: Some(message),
            success: false,
        }
    }
}

pub fn emit_event(event: &CliEvent) {
    let json = serde_json::to_string(event).unwrap_or_else(|_| {
        format!(
            r#"{{"event":"error","name":"{}","message":"json_encode_failed"}}"#,
            event.name
        )
    });
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{json}");
    let _ = handle.flush();
}

pub fn error_code(error: &(dyn std::error::Error + Send + Sync)) -> String {
    let err_str = error.to_string().to_lowercase();
    if err_str.contains("not found") || err_str.contains("no such file") {
        String::from("file_not_found")
    } else if err_str.contains("permission denied") {
        String::from("permission_denied")
    } else if err_str.contains("parse") || err_str.contains("json") {
        String::from("parse_error")
    } else if err_str.contains("validation") || err_str.contains("invalid") {
        String::from("validation_error")
    } else {
        String::from("unknown_error")
    }
}

pub fn exit_code(error: &(dyn std::error::Error + Send + Sync)) -> i32 {
    let code = error_code(error);
    match code.as_str() {
        "file_not_found" => 2,
        "permission_denied" => 3,
        "parse_error" => 4,
        "validation_error" => 5,
        _ => 1,
    }
}
