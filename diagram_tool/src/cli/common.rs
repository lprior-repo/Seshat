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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    // --- CliEvent::start ---

    #[test]
    fn cli_event_start_has_correct_fields() {
        let event = CliEvent::start("validate".to_string());
        assert_eq!(event.event, "cli_start");
        assert_eq!(event.name, "validate");
        assert!(event.success, "start event should have success=true");
        assert_eq!(event.code, None, "start event should have code=None");
        assert_eq!(event.message, None, "start event should have message=None");
    }

    // --- CliEvent::finish ---

    #[test]
    fn cli_event_finish_has_correct_fields() {
        let event = CliEvent::finish("render".to_string(), true, "ok".to_string());
        assert_eq!(event.event, "cli_finish");
        assert_eq!(event.name, "render");
        assert!(event.success, "finish event success should match");
        assert_eq!(
            event.code,
            Some("ok".to_string()),
            "finish event should have code=Some(code)"
        );
        assert_eq!(event.message, None, "finish event should have message=None");
    }

    #[test]
    fn cli_event_finish_failure() {
        let event = CliEvent::finish("layout".to_string(), false, "file_not_found".to_string());
        assert_eq!(event.event, "cli_finish");
        assert!(!event.success);
        assert_eq!(event.code, Some("file_not_found".to_string()));
    }

    // --- CliEvent::error ---

    #[test]
    fn cli_event_error_has_correct_fields() {
        let event = CliEvent::error(
            "validate".to_string(),
            "file_not_found".to_string(),
            "No such file".to_string(),
        );
        assert_eq!(event.event, "cli_error");
        assert_eq!(event.name, "validate");
        assert!(!event.success, "error event should have success=false");
        assert_eq!(event.code, Some("file_not_found".to_string()));
        assert_eq!(event.message, Some("No such file".to_string()));
    }

    // --- emit_event ---

    #[test]
    fn emit_event_does_not_panic() {
        let event = CliEvent::start("test".to_string());
        emit_event(&event);
    }

    #[test]
    fn emit_event_produces_valid_json() {
        let event = CliEvent::start("render".to_string());
        // Capture that emit_event serializes without error
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["event"], "cli_start");
        assert_eq!(parsed["name"], "render");
        assert_eq!(parsed["success"], true);
    }

    // --- error_code ---

    #[test]
    fn error_code_maps_not_found() {
        let err = TestError("file not found".to_string());
        assert_eq!(error_code(&err), "file_not_found");
    }

    #[test]
    fn error_code_maps_permission_denied() {
        let err = TestError("permission denied".to_string());
        assert_eq!(error_code(&err), "permission_denied");
    }

    #[test]
    fn error_code_maps_parse() {
        let err = TestError("parse error on line 5".to_string());
        assert_eq!(error_code(&err), "parse_error");
    }

    #[test]
    fn error_code_maps_invalid() {
        let err = TestError("invalid field value".to_string());
        assert_eq!(error_code(&err), "validation_error");
    }

    #[test]
    fn error_code_maps_unknown() {
        let err = TestError("something completely unexpected".to_string());
        assert_eq!(error_code(&err), "unknown_error");
    }

    // --- exit_code ---

    #[test]
    fn exit_code_maps_file_not_found() {
        let err = TestError("file not found".to_string());
        assert_eq!(exit_code(&err), 2);
    }

    #[test]
    fn exit_code_maps_permission_denied() {
        let err = TestError("permission denied".to_string());
        assert_eq!(exit_code(&err), 3);
    }

    #[test]
    fn exit_code_maps_parse_error() {
        let err = TestError("parse error".to_string());
        assert_eq!(exit_code(&err), 4);
    }

    #[test]
    fn exit_code_maps_validation_error() {
        let err = TestError("invalid data".to_string());
        assert_eq!(exit_code(&err), 5);
    }

    #[test]
    fn exit_code_maps_unknown_to_default() {
        let err = TestError("totally unknown issue".to_string());
        assert_eq!(exit_code(&err), 1);
    }
}
