use crate::error::{ExecutionError, ShowError};
use std::path::PathBuf;

#[test]
fn show_error_display_file_not_found_starts_with_prefix_and_contains_payload() {
    let err = ShowError::FileNotFound(PathBuf::from("/tmp/missing.json"));
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("/tmp/missing.json"));
}

#[test]
fn show_error_display_io_error_starts_with_prefix_and_contains_payload() {
    let err = ShowError::IoError("disk fail".to_string());
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("disk fail"));
}

#[test]
fn show_error_display_invalid_utf8_starts_with_prefix_and_contains_utf8_mention() {
    let err = ShowError::InvalidUtf8;
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    let lower = output.to_lowercase();
    assert!(lower.contains("utf-8") || lower.contains("utf8"));
}

#[test]
fn show_error_display_empty_input_starts_with_prefix_and_contains_empty() {
    let err = ShowError::EmptyInput;
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("empty"));
}

#[test]
fn show_error_display_json_deserialize_starts_with_prefix_and_contains_payload() {
    let err = ShowError::JsonDeserialize("parse error".to_string());
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("parse error"));
}

#[test]
fn show_error_display_invalid_document_starts_with_prefix_and_contains_payload() {
    let err = ShowError::InvalidDocument("unknown field".to_string());
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("unknown field"));
}

#[test]
fn show_error_display_serialization_failure_starts_with_prefix_and_contains_payload() {
    let err = ShowError::SerializationFailure("ser fail".to_string());
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("ser fail"));
}

#[test]
fn show_error_display_stdout_write_failure_starts_with_prefix_and_contains_payload() {
    let err = ShowError::StdoutWriteFailure("write fail".to_string());
    let output = err.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("write fail"));
}

#[test]
fn execution_error_show_variant_delegates_display_to_show_error() {
    let e = ExecutionError::Show(ShowError::EmptyInput);
    let output = e.to_string();
    assert!(output.starts_with("error: show: "));
    assert!(output.contains("empty"));
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_show_error_display_always_starts_with_error_show_prefix(
            payload in ".*"
        ) {
            let variants: Vec<ShowError> = vec![
                ShowError::FileNotFound(std::path::PathBuf::from(&payload)),
                ShowError::IoError(payload.clone()),
                ShowError::InvalidUtf8,
                ShowError::EmptyInput,
                ShowError::JsonDeserialize(payload.clone()),
                ShowError::InvalidDocument(payload.clone()),
                ShowError::SerializationFailure(payload.clone()),
                ShowError::StdoutWriteFailure(payload),
            ];
            prop_assert!(variants[0].to_string().starts_with("error: show: "));
            prop_assert!(variants[1].to_string().starts_with("error: show: "));
            prop_assert!(variants[2].to_string().starts_with("error: show: "));
            prop_assert!(variants[3].to_string().starts_with("error: show: "));
            prop_assert!(variants[4].to_string().starts_with("error: show: "));
            prop_assert!(variants[5].to_string().starts_with("error: show: "));
            prop_assert!(variants[6].to_string().starts_with("error: show: "));
            prop_assert!(variants[7].to_string().starts_with("error: show: "));
        }
    }
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_show_error_display_prefix_for_all_variants() {
        let err = ShowError::EmptyInput;
        let s = err.to_string();
        assert!(s.starts_with("error: show: "));

        let err2 = ShowError::InvalidUtf8;
        let s2 = err2.to_string();
        assert!(s2.starts_with("error: show: "));
    }
}
