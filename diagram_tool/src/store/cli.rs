use super::append::append_event;
use super::errors::CliError;
use super::types::AppendOutcome;
use crate::models::envelope::EventEnvelope;
use rusqlite::Connection;

pub fn submit_cli_op(
    conn: &mut Connection,
    envelope: EventEnvelope,
    expected_revision: Option<i64>,
) -> Result<AppendOutcome, CliError> {
    if envelope.op_id.is_empty() {
        return Err(CliError::InvalidInput("op_id is required".to_string()));
    }
    if envelope.author.id.is_empty() {
        return Err(CliError::InvalidInput("author.id is required".to_string()));
    }

    let result = append_event(conn, envelope, expected_revision)?;
    Ok(AppendOutcome::from(result))
}

#[must_use]
pub fn cli_submit_response(outcome: &AppendOutcome) -> String {
    serde_json::json!({
        "ok": true,
        "revision": outcome.revision.get(),
        "op_id": outcome.op_id.as_str(),
        "timestamp": outcome.timestamp.get()
    })
    .to_string()
}
