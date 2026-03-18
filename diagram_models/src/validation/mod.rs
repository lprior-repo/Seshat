#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod rules;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptests;

pub use rules::{validate_document, validate_document_data};
pub use types::{ValidationCode, ValidationIssue, ValidationSeverity};
