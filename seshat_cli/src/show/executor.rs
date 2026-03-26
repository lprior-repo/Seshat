use super::loader::{load_document_from_path, load_document_from_reader};
use crate::domain::{ShowCommand, ShowSource};
use crate::error::ShowError;
use diagram_models::document::DiagramDocument;

/// Top-level executor for the `show` subcommand.
/// Orchestrates: load → serialize → write.
///
/// Accepts an injectable writer and an injectable `serialize_fn` for deterministic testing.
///
/// # Postconditions
/// - On `Ok(())`: the JSON representation of the `DiagramDocument` followed by `\n` has been
///   written to `writer`.
///
/// # Errors
/// - Any `ShowError` variant may be returned.
pub fn execute_show<W: std::io::Write>(
    cmd: &ShowCommand,
    stdin_reader: impl std::io::Read,
    mut writer: W,
    serialize_fn: impl Fn(&DiagramDocument) -> Result<String, ShowError>,
) -> Result<(), ShowError> {
    let doc = load_document(cmd, stdin_reader)?;
    let json = serialize_fn(&doc)?;
    write_output(&mut writer, &json)
}

/// Dispatches loading to the correct source (file vs. stdin).
fn load_document(
    cmd: &ShowCommand,
    stdin_reader: impl std::io::Read,
) -> Result<DiagramDocument, ShowError> {
    match &cmd.source {
        ShowSource::File(path) => load_document_from_path(path),
        ShowSource::Stdin => load_document_from_reader(stdin_reader),
    }
}

/// Writes the JSON string followed by a single newline to `writer`, then flushes.
///
/// # Errors
/// - `ShowError::StdoutWriteFailure` if `write_all` or `flush` fails.
fn write_output<W: std::io::Write>(writer: &mut W, json: &str) -> Result<(), ShowError> {
    let mut output = Vec::with_capacity(json.len() + 1);
    output.extend_from_slice(json.as_bytes());
    output.push(b'\n');
    writer
        .write_all(&output)
        .map_err(|e| ShowError::StdoutWriteFailure(e.to_string()))?;
    writer
        .flush()
        .map_err(|e| ShowError::StdoutWriteFailure(e.to_string()))
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Cursor;

    proptest! {
        #[test]
        fn proptest_execute_show_output_ends_with_exactly_one_newline_when_successful(
            version in any::<u32>()
        ) {
            let doc = DiagramDocument {
                version,
                ..DiagramDocument::default()
            };
            let Ok(json) = serde_json::to_string(&doc) else {
                return Ok(());
            };
            let cmd = ShowCommand { source: ShowSource::Stdin };
            let reader = Cursor::new(json.into_bytes());
            let mut writer = Vec::<u8>::new();
            let result = execute_show(&cmd, reader, &mut writer, crate::show::serialize_document);
            prop_assert!(result.is_ok(), "execute_show must succeed: {result:?}");
            prop_assert!(writer.ends_with(b"\n"), "output must end with newline");
            prop_assert!(writer.len() >= 2, "output must have at least JSON char + newline");
            prop_assert_ne!(writer[writer.len() - 2], b'\n', "no double trailing newline");
        }
    }
}
