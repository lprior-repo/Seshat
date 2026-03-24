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
mod contract_tests;
#[cfg(test)]
mod edgecase_tests;
#[cfg(test)]
mod regression_tests;
#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod proptests;

pub use label::{is_valid_label, MAX_LABEL_LENGTH};
pub use rules::{is_valid_hex_color, validate_document, validate_document_data};
pub use types::{ValidationCode, ValidationIssue, ValidationSeverity};
