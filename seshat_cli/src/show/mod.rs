//! Implementation of the `seshat show` subcommand.
//!
//! # Architecture: Data → Calculations → Actions
//!
//! - **Data**: `ShowCommand`, `ShowSource`, `DiagramDocument` — inert, no I/O.
//! - **Calculations**: `map_show_subcommand`, `serialize_document` — pure functions.
//! - **Actions**: `load_document_from_path`, `load_document_from_reader`, `execute_show` — I/O boundary.

pub mod executor;
pub mod loader;
pub mod mapper;
pub mod serializer;

#[cfg(test)]
pub mod error_tests;

pub use executor::execute_show;
pub use loader::{load_document_from_path, load_document_from_reader};
pub(crate) use mapper::map_show_subcommand;
pub use serializer::serialize_document;
