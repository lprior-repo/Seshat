//! Server module for Dioxus server functions.
//!
//! This module contains server-side only code that is compiled when targeting
//! non-WASM architectures (server/desktop).

pub mod ai_documents;

#[cfg(test)]
pub mod ai_documents_tests;
