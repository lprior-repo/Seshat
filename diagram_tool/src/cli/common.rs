use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::cli_persistence::load_workspace_with_lkg;
use diagram_models::document::DiagramDocument;

#[derive(Serialize, Deserialize)]
pub struct CliEvent {
    pub event: String,
    pub command: String,
    pub ok: bool,
    pub code: String,
    pub message: Option<String>,
}

impl CliEvent {
    pub fn start(command: String) -> Self {
        Self {
            event: String::from("start"),
            command,
            ok: true,
            code: String::from("start"),
            message: None,
        }
    }

    pub fn error(command: String, code: String, message: String) -> Self {
        Self {
            event: String::from("error"),
            command,
            ok: false,
            code,
            message: Some(message),
        }
    }

    pub fn finish(command: String, ok: bool, code: String) -> Self {
        Self {
            event: String::from("finish"),
            command,
            ok,
            code,
            message: None,
        }
    }
}

pub fn error_code(err: &anyhow::Error) -> String {
    let msg = err.to_string().to_lowercase();
    if msg.contains("dag") || msg.contains("cycle") {
        String::from("dag_violation")
    } else if msg.contains("dangling") || msg.contains("edge-dangling") {
        String::from("dangling_reference")
    } else if msg.contains("stale_revision") {
        String::from("stale_revision")
    } else if msg.contains("schema") {
        String::from("schema_violation")
    } else if msg.contains("semantic") || msg.contains("semantic validation error") {
        String::from("semantic_error")
    } else if msg.contains("parse")
        || msg.contains("deserialize")
        || msg.contains("unknown variant")
        || msg.contains("failed to parse")
    {
        String::from("parse_error")
    } else {
        String::from("command_error")
    }
}

pub fn exit_code(err: &anyhow::Error) -> i32 {
    let code = error_code(err);
    match code.as_str() {
        "parse_error" | "command_error" => 2,
        _ => 1,
    }
}

pub fn emit_event(event: &CliEvent) {
    match serde_json::to_string(&event) {
        Ok(line) => println!("{line}"),
        Err(_) => {
            println!("{{\"event\":\"error\",\"ok\":false,\"code\":\"jsonl_encode_error\"}}");
        }
    }
}

pub fn load_doc(path: &str) -> Result<DiagramDocument> {
    load_workspace_with_lkg(Path::new(path))
        .map_err(|e| anyhow!("Failed to load document from {path}: {e}"))
}
