use dioxus::prelude::*;

use diagram_models::document::{DiagramDocument, Revision};
use diagram_models::schema::validate_schema;
use diagram_models::validation::{
    validate_document_data, ValidationIssue, ValidationSeverity,
};

use super::types::VALIDATION_IDLE_MS;

pub(crate) fn collect_validation_issues(doc: &DiagramDocument) -> Vec<ValidationIssue> {
    let schema_issues = validate_schema(doc)
        .err()
        .map(|e| ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "schema",
            message: e.to_string(),
            subject: None,
        })
        .into_iter();

    schema_issues
        .chain(validate_document_data(&doc.document))
        .collect()
}

pub(crate) fn use_validation_state(
    doc_signal: Signal<DiagramDocument>,
    validate_trigger: Signal<u64>,
) -> Signal<Vec<ValidationIssue>> {
    let mut validation_issues = use_signal(move || collect_validation_issues(&doc_signal.read()));
    let mut last_validated_revision = use_signal(move || doc_signal.read().revision);
    let mut last_validate_trigger = use_signal(move || *validate_trigger.read());
    let mut queued_validation_revision = use_signal(|| Option::<Revision>::None);
    let mut validation_job = use_signal(|| 0_u64);

    use_effect(move || {
        let current_trigger = *validate_trigger.read();
        if current_trigger != *last_validate_trigger.read() {
            let current_document = doc_signal.read().clone();
            validation_issues.set(collect_validation_issues(&current_document));
            last_validated_revision.set(doc_signal.read().revision);
            last_validate_trigger.set(current_trigger);
            queued_validation_revision.set(None);
            validation_job.with_mut(|job| {
                *job = job.saturating_add(1);
            });
            return;
        }

        let doc = doc_signal.read();
        let current_revision = doc.revision;
        let already_validated = current_revision == *last_validated_revision.read();
        let already_queued = queued_validation_revision
            .read()
            .as_ref()
            .is_some_and(|queued| *queued == current_revision);

        if already_validated || already_queued {
            return;
        }

        queued_validation_revision.set(Some(current_revision));

        let next_job = (*validation_job.read()).saturating_add(1);
        validation_job.set(next_job);
        let current_document = doc.clone();
        drop(doc);

        let mut eval = document::eval(&format!(
            "setTimeout(() => dioxus.send({{ job: {next_job} }}), {VALIDATION_IDLE_MS});"
        ));

        spawn(async move {
            let Ok(message) = eval.recv::<serde_json::Value>().await else {
                return;
            };
            let fired_job = message["job"].as_u64().map_or(0, |value| value);

            if fired_job != next_job || *validation_job.read() != next_job {
                return;
            }

            let still_queued = queued_validation_revision
                .read()
                .as_ref()
                .is_some_and(|queued| *queued == current_revision);

            if !still_queued {
                return;
            }

            validation_issues.set(collect_validation_issues(&current_document));
            last_validated_revision.set(current_revision);
            queued_validation_revision.set(None);
        });
    });

    validation_issues
}
