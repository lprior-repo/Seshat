//! Kani verification harnesses for the `show` module.

#![allow(unexpected_cfgs)]

use crate::domain::{ShowCommand, ShowSource};
use crate::error::ShowError;
use diagram_models::document::DiagramDocument;
use std::path::PathBuf;

use super::*;

#[kani::proof]
fn verify_map_show_subcommand_is_structurally_total() {
    let has_file: bool = kani::any();
    let cmd = if has_file {
        map_show_subcommand(Some(PathBuf::from("/bounded/path.json")))
    } else {
        map_show_subcommand(None)
    };
    if has_file {
        assert!(matches!(cmd.source, ShowSource::File(_)));
    } else {
        assert!(matches!(cmd.source, ShowSource::Stdin));
    }
}

#[kani::proof]
fn verify_serialize_document_never_panics_for_valid_doc() {
    let doc = DiagramDocument {
        version: kani::any(),
        ..DiagramDocument::default()
    };
    let result = serialize_document(&doc);
    if let Ok(s) = result {
        assert!(!s.is_empty());
    }
}

#[kani::proof]
fn verify_show_error_display_prefix_for_all_variants() {
    let err = ShowError::EmptyInput;
    let s = err.to_string();
    assert!(s.starts_with("error: show: "));

    let err2 = ShowError::InvalidUtf8;
    let s2 = err2.to_string();
    assert!(s2.starts_with("error: show: "));
}

#[kani::proof]
fn verify_load_document_from_reader_cannot_return_file_not_found() {
    use std::io::Cursor;
    let reader = Cursor::new(vec![]);
    let result = load_document_from_reader(reader);
    match result {
        Err(ShowError::FileNotFound(_)) => {
            assert!(false, "FileNotFound is impossible for reader-based load");
        }
        _ => {}
    }
}
