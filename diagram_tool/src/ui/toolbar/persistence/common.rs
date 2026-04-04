#![allow(dead_code)]
use crate::history::History;
use crate::mutation::pipeline::{run_mutation_with_policy, RevisionPolicy, ValidationPolicy};
use crate::ui::toast::{ToastHandle, ToastIntent, ToastUpdate};
use diagram_models::document::{DiagramDocument, Revision};
use diagram_models::schema::validate_schema;

#[derive(Debug)]
pub enum ImportTransitionError {
    Parse(String),
    Validation(String),
}

pub fn prepare_import_transition(
    current: &DiagramDocument,
    contents: &str,
) -> Result<(DiagramDocument, History), ImportTransitionError> {
    let mut loaded_doc =
        super::super::persistence_compat::parse_diagram_document_with_compat(contents)
            .map_err(ImportTransitionError::Parse)?;

    validate_schema(&loaded_doc)
        .map_err(|e| ImportTransitionError::Validation(format!("Schema validation failed: {e}")))?;

    loaded_doc.revision = Revision::INITIAL;

    run_mutation_with_policy(
        current,
        RevisionPolicy::Preserve,
        ValidationPolicy::default(),
        |_| Ok(loaded_doc),
    )
    .map(|next_doc| (next_doc, History::new().push(current.clone())))
    .map_err(|err| {
        ImportTransitionError::Validation(super::super::mutation_error_code(&err).to_string())
    })
}

pub fn apply_import_contents(
    doc: &mut DiagramDocument,
    history: &mut History,
    contents: &str,
) -> Result<(), ImportTransitionError> {
    let current = doc.clone();
    match prepare_import_transition(&current, contents) {
        Ok((next_doc, next_history)) => {
            *doc = next_doc;
            *history = next_history;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

pub fn update_load_save_success(toast_handle: ToastHandle, title: &str, detail: String) {
    if !toast_handle.update(ToastUpdate {
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Success),
        action: None,
    }) {
        eprintln!("Failed to update success toast");
    }
}

pub fn update_load_save_error(toast_handle: ToastHandle, title: &str, detail: String) {
    if !toast_handle.update(ToastUpdate {
        title: Some(title.to_string()),
        detail: Some(Some(detail)),
        intent: Some(ToastIntent::Error),
        action: None,
    }) {
        eprintln!("Failed to update error toast");
    }
}
