#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::validation::{ValidationIssue, ValidationSeverity};
use crate::ui::theme::{BG_SURFACE, BORDER_SUBTLE, ERROR, SUCCESS, TEXT_MAIN, TEXT_MUTED, WARNING};
use dioxus::prelude::*;

/// Read-only panel that displays validation issues for the current document.
///
/// Never mutates `doc_signal`. Never panics on an empty issue list.
#[component]
pub fn ValidationPanel(issues: ReadSignal<Vec<ValidationIssue>>) -> Element {
    let issue_list = issues.read();
    let error_count = issue_list
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Error)
        .count();
    let has_issues = !issue_list.is_empty();

    rsx! {
        div {
            style: "padding: 8px; border-top: 1px solid {BORDER_SUBTLE}; background: {BG_SURFACE}; max-height: 200px; overflow-y: auto;",

            div {
                style: "display: flex; align-items: center; gap: 8px; margin-bottom: 4px;",
                span { style: "font-weight: bold; font-size: 12px; color: {TEXT_MAIN};", "Validation" }
                if has_issues {
                    span {
                        style: "background: {ERROR}; color: {TEXT_MAIN}; border-radius: 9999px; padding: 1px 7px; font-size: 11px;",
                        "{error_count}"
                    }
                } else {
                    span {
                        style: "background: {SUCCESS}; color: {TEXT_MAIN}; border-radius: 9999px; padding: 1px 7px; font-size: 11px;",
                        "Valid"
                    }
                }
            }

            for issue in issue_list.iter() {
                {
                    let subject_str = issue
                        .subject
                        .as_deref()
                        .map_or_else(|| String::from("global"), str::to_string);
                    let key = format!("{}-{}", issue.code, subject_str);
                    let severity_color = match issue.severity {
                        ValidationSeverity::Error => ERROR,
                        ValidationSeverity::Warning => WARNING,
                    };
                    let severity_icon = match issue.severity {
                        ValidationSeverity::Error => "✕",
                        ValidationSeverity::Warning => "⚠",
                    };
                    let span_style =
                        format!("color: {severity_color}; font-weight: bold; flex-shrink: 0;");
                    let message = issue.message.clone();
                    rsx! {
                        div {
                            key: "{key}",
                            style: "font-size: 11px; padding: 2px 4px; display: flex; gap: 6px; align-items: flex-start; color: {TEXT_MUTED};",
                            span { style: "{span_style}", "{severity_icon}" }
                            span { "{message}" }
                        }
                    }
                }
            }
        }
    }
}
