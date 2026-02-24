#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::validation::{ValidationIssue, ValidationSeverity};
use dioxus::prelude::*;

/// Read-only panel that displays validation issues for the current document.
///
/// Never mutates `doc_signal`. Never panics on an empty issue list.
#[component]
pub fn ValidationPanel(issues: ReadOnlySignal<Vec<ValidationIssue>>) -> Element {
    let issue_list = issues.read();
    let error_count = issue_list
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Error)
        .count();
    let has_issues = !issue_list.is_empty();

    rsx! {
        div {
            style: "padding: 8px; border-top: 1px solid #ccc; background: #fafafa; max-height: 200px; overflow-y: auto;",

            div {
                style: "display: flex; align-items: center; gap: 8px; margin-bottom: 4px;",
                span { style: "font-weight: bold; font-size: 12px;", "Validation" }
                if has_issues {
                    span {
                        style: "background: #ef4444; color: white; border-radius: 9999px; padding: 1px 7px; font-size: 11px;",
                        "{error_count}"
                    }
                } else {
                    span {
                        style: "background: #22c55e; color: white; border-radius: 9999px; padding: 1px 7px; font-size: 11px;",
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
                        ValidationSeverity::Error => "#ef4444",
                        ValidationSeverity::Warning => "#f59e0b",
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
                            style: "font-size: 11px; padding: 2px 4px; display: flex; gap: 6px; align-items: flex-start;",
                            span { style: "{span_style}", "{severity_icon}" }
                            span { "{message}" }
                        }
                    }
                }
            }
        }
    }
}
