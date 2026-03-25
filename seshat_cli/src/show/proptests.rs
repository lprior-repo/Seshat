//! Proptest invariants for the `show` module.

use crate::domain::{ShowCommand, ShowSource};
use crate::error::ShowError;
use diagram_models::document::DiagramDocument;
use proptest::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;

use super::*;

// INV-1: map_show_subcommand never panics for any Option<PathBuf>
proptest! {
    #[test]
    fn proptest_map_show_subcommand_never_panics_for_any_option_pathbuf(
        bytes in prop::option::of(prop::collection::vec(any::<u8>(), 0..256))
    ) {
        let path_opt = bytes.map(|b| {
            use std::ffi::OsString;
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStringExt;
                PathBuf::from(OsString::from_vec(b))
            }
            #[cfg(not(unix))]
            {
                PathBuf::from(String::from_utf8_lossy(&b).to_string())
            }
        });
        let is_some = path_opt.is_some();
        let result = map_show_subcommand(path_opt);
        if is_some {
            prop_assert!(matches!(result.source, ShowSource::File(_)));
        } else {
            prop_assert_eq!(result.source, ShowSource::Stdin);
        }
    }
}

// INV-2: serialize_document is total for any DiagramDocument (version sweep)
proptest! {
    #[test]
    fn proptest_serialize_document_returns_ok_for_any_well_formed_document(
        version in any::<u32>()
    ) {
        let doc = DiagramDocument {
            version,
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        prop_assert!(result.is_ok(), "serialize_document must return Ok for any version: {result:?}");
        if let Ok(ref json) = result {
            prop_assert!(!json.is_empty(), "serialized JSON must be non-empty");
        }
    }
}

// INV-3: JSON round-trip identity
proptest! {
    #[test]
    fn proptest_serialize_then_deserialize_produces_identical_document(
        version in any::<u32>()
    ) {
        let doc = DiagramDocument {
            version,
            ..DiagramDocument::default()
        };
        let json_result = serialize_document(&doc);
        prop_assert!(json_result.is_ok(), "serialize must succeed: {json_result:?}");
        if let Ok(json) = json_result {
            let doc2 = load_document_from_reader(Cursor::new(json.into_bytes()));
            prop_assert_eq!(doc2, Ok(doc));
        }
    }
}

// INV-4: load_document_from_reader never panics on arbitrary bytes
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

// INV-5: execute_show output always ends with exactly one newline on success
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
        let result = execute_show(&cmd, reader, &mut writer, serialize_document);
        prop_assert!(result.is_ok(), "execute_show must succeed: {result:?}");
        prop_assert!(writer.ends_with(b"\n"), "output must end with newline");
        prop_assert!(writer.len() >= 2, "output must have at least JSON char + newline");
        prop_assert_ne!(writer[writer.len() - 2], b'\n', "no double trailing newline");
    }
}

// INV-6: ShowError display always starts with "error: show:"
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
        prop_assert!(
            variants[0].to_string().starts_with("error: show: "),
            "FileNotFound display must start with 'error: show: ', got: {:?}", variants[0].to_string()
        );
        prop_assert!(
            variants[1].to_string().starts_with("error: show: "),
            "IoError display must start with 'error: show: ', got: {:?}", variants[1].to_string()
        );
        prop_assert!(
            variants[2].to_string().starts_with("error: show: "),
            "InvalidUtf8 display must start with 'error: show: ', got: {:?}", variants[2].to_string()
        );
        prop_assert!(
            variants[3].to_string().starts_with("error: show: "),
            "EmptyInput display must start with 'error: show: ', got: {:?}", variants[3].to_string()
        );
        prop_assert!(
            variants[4].to_string().starts_with("error: show: "),
            "JsonDeserialize display must start with 'error: show: ', got: {:?}", variants[4].to_string()
        );
        prop_assert!(
            variants[5].to_string().starts_with("error: show: "),
            "InvalidDocument display must start with 'error: show: ', got: {:?}", variants[5].to_string()
        );
        prop_assert!(
            variants[6].to_string().starts_with("error: show: "),
            "SerializationFailure display must start with 'error: show: ', got: {:?}", variants[6].to_string()
        );
        prop_assert!(
            variants[7].to_string().starts_with("error: show: "),
            "StdoutWriteFailure display must start with 'error: show: ', got: {:?}", variants[7].to_string()
        );
    }
}
