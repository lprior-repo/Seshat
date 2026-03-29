//! UI mutation helpers - validated mutation operations
//!
//! This module provides validated mutation helpers that maintain functional-rust principles:
//! - No unwrap/expect/panic in source code
//! - No mut - use persistent data structures
//! - Result<T, E> for error handling

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use dioxus::prelude::{ReadableExt, Signal, WritableExt};

use crate::history::History;
use crate::mutation::error::MutationError;
use crate::mutation::pipeline::run_mutation;
use diagram_models::document::DiagramDocument;

pub type MutationResult<T> = Result<T, MutationError>;

#[derive(Debug)]
pub enum UiMutationError {
    Mutation(MutationError),
}

impl From<MutationError> for UiMutationError {
    fn from(e: MutationError) -> Self {
        Self::Mutation(e)
    }
}

impl std::fmt::Display for UiMutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mutation(e) => write!(f, "mutation error: {}", e),
        }
    }
}

impl std::error::Error for UiMutationError {}

pub type UiMutationResult<T> = Result<T, UiMutationError>;

pub fn mutate_doc_signal<F>(
    doc_signal: &mut Signal<DiagramDocument>,
    transform: F,
) -> UiMutationResult<()>
where
    F: FnOnce(DiagramDocument) -> MutationResult<DiagramDocument>,
{
    let current = doc_signal.read().clone();
    match run_mutation(&current, |_| transform(current.clone())) {
        Ok(new_doc) => {
            *doc_signal.write() = new_doc;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn mutate_doc_with_history(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    transform: impl FnOnce(DiagramDocument) -> MutationResult<DiagramDocument>,
) -> UiMutationResult<()> {
    let current = doc_signal.read().clone();
    match run_mutation(&current, |_| transform(current.clone())) {
        Ok(new_doc) => {
            let new_history = history_signal.read().push(current.clone());
            *history_signal.write() = new_history;
            *doc_signal.write() = new_doc;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn mutate_doc_with_history_and_result<T, F>(
    doc_signal: &mut Signal<DiagramDocument>,
    history_signal: &mut Signal<History>,
    transform: F,
) -> UiMutationResult<T>
where
    F: FnOnce(DiagramDocument) -> MutationResult<(DiagramDocument, T)>,
{
    let current = doc_signal.read().clone();
    let next = transform(current.clone())?;

    let issues = diagram_models::validation::validate_document(&next.0);
    if let Some(issue) = issues.first() {
        return Err(MutationError::from_issue(issue).into());
    }

    let new_history = history_signal.read().push(current);
    *history_signal.write() = new_history;
    *doc_signal.write() = next.0;
    Ok(next.1)
}

pub fn mutate_editor_signal<T, F>(signal: &mut Signal<T>, transform: F) -> UiMutationResult<()>
where
    F: FnOnce(T) -> T,
    T: Clone + 'static,
{
    let current = signal.read().clone();
    let new_value = transform(current);
    *signal.write() = new_value;
    Ok(())
}
