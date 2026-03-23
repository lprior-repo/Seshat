#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod label;
pub mod rules;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptests;

pub use label::{is_valid_label, MAX_LABEL_LENGTH};
pub use rules::{validate_document, validate_document_data};
pub use types::{ValidationCode, ValidationIssue, ValidationSeverity};
