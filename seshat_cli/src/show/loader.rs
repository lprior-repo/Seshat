use crate::error::ShowError;
use diagram_models::document::DiagramDocument;
use std::io::Read as _;

/// Reads bytes from the filesystem and deserializes a `DiagramDocument`.
///
/// # Errors
/// - `ShowError::FileNotFound`    — path does not exist or permissions denied.
/// - `ShowError::IoError`         — other I/O failure during read.
/// - `ShowError::InvalidUtf8`     — bytes are not valid UTF-8.
/// - `ShowError::EmptyInput`      — file is zero bytes (or whitespace only).
/// - `ShowError::JsonDeserialize` — content is not valid JSON.
/// - `ShowError::InvalidDocument` — JSON is valid but does not match `DiagramDocument` schema.
pub fn load_document_from_path(path: &std::path::Path) -> Result<DiagramDocument, ShowError> {
    std::fs::File::open(path)
        .map_err(|e| map_open_error(&e, path))
        .and_then(load_document_from_reader)
}

/// Maps a `std::io::Error` from `File::open` into the correct `ShowError` variant.
///
/// `NotFound` and `PermissionDenied` map to `FileNotFound`.
/// All other I/O errors map to `IoError`.
fn map_open_error(e: &std::io::Error, path: &std::path::Path) -> ShowError {
    match e.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            ShowError::FileNotFound(path.to_path_buf())
        }
        _ => ShowError::IoError(e.to_string()),
    }
}

/// Reads bytes from any `Read` implementor and deserializes a `DiagramDocument`.
///
/// # Errors
/// - `ShowError::IoError`         — I/O failure while reading.
/// - `ShowError::InvalidUtf8`     — bytes are not valid UTF-8.
/// - `ShowError::EmptyInput`      — reader yields zero bytes (or whitespace only).
/// - `ShowError::JsonDeserialize` — content is not valid JSON.
/// - `ShowError::InvalidDocument` — JSON valid but document schema mismatch.
pub fn load_document_from_reader<R: std::io::Read>(
    mut reader: R,
) -> Result<DiagramDocument, ShowError> {
    let bytes = read_bytes_from_reader(&mut reader)?;
    let text = decode_utf8(bytes)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ShowError::EmptyInput);
    }
    parse_document(trimmed)
}

/// Maximum bytes read from any reader to prevent infinite-stream hangs.
/// 64 MiB is sufficient for any realistic diagram document.
const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;

/// Reads all bytes from a reader into a `Vec<u8>`.
///
/// Reads at most `MAX_INPUT_BYTES` bytes. If the stream exceeds this limit
/// the read is truncated and subsequent parsing will fail gracefully.
///
/// # Errors
/// - `ShowError::IoError` if reading fails.
fn read_bytes_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Vec<u8>, ShowError> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(MAX_INPUT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| ShowError::IoError(e.to_string()))?;
    Ok(bytes)
}

/// Converts raw bytes to a UTF-8 `String`.
///
/// # Errors
/// - `ShowError::InvalidUtf8` if the bytes are not valid UTF-8.
fn decode_utf8(bytes: Vec<u8>) -> Result<String, ShowError> {
    String::from_utf8(bytes).map_err(|_| ShowError::InvalidUtf8)
}

/// Parses a trimmed JSON string into a `DiagramDocument`.
///
/// Uses a two-pass strategy:
/// 1. Parse as raw `serde_json::Value` to detect syntax/EOF errors → `JsonDeserialize`.
/// 2. Parse as `DiagramDocument`; data errors (unknown fields, schema mismatches)
///    containing "unknown field" → `InvalidDocument`; all others → `JsonDeserialize`.
///
/// # Errors
/// - `ShowError::InvalidDocument` — JSON valid but unknown fields or structural mismatch.
/// - `ShowError::JsonDeserialize` — malformed JSON syntax or type mismatch.
fn parse_document(json: &str) -> Result<DiagramDocument, ShowError> {
    // Pass 1: validate raw JSON syntax first.
    serde_json::from_str::<serde_json::Value>(json)
        .map_err(|e| ShowError::JsonDeserialize(e.to_string()))?;

    // Pass 2: deserialize into the typed struct.
    serde_json::from_str::<DiagramDocument>(json).map_err(|e| {
        if e.is_data() && e.to_string().contains("unknown field") {
            ShowError::InvalidDocument(e.to_string())
        } else {
            ShowError::JsonDeserialize(e.to_string())
        }
    })
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Cursor;

    proptest! {
        #[test]
        fn proptest_load_document_from_reader_never_panics_for_arbitrary_byte_input(
            bytes in prop::collection::vec(any::<u8>(), 0..1000)
        ) {
            let reader = Cursor::new(bytes);
            let result = load_document_from_reader(reader);
            match result {
                Ok(_) => prop_assert!(true),
                Err(
                    ShowError::EmptyInput
                    | ShowError::InvalidUtf8
                    | ShowError::JsonDeserialize(_)
                    | ShowError::InvalidDocument(_)
                    | ShowError::IoError(_),
                ) => prop_assert!(true),
                Err(
                    ShowError::FileNotFound(_)
                    | ShowError::SerializationFailure(_)
                    | ShowError::StdoutWriteFailure(_),
                ) => {
                    prop_assert!(false, "reader-based load must not return file/write errors");
                }
            }
        }
    }
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_load_document_from_reader_cannot_return_file_not_found() {
        use std::io::Cursor;
        let reader = Cursor::new(vec![]);
        let result = load_document_from_reader(reader);
        match result {
            Err(crate::error::ShowError::FileNotFound(_)) => {
                assert!(false, "FileNotFound is impossible for reader-based load");
            }
            _ => {}
        }
    }
}
