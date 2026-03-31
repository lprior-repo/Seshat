//! History module for undo/redo operations
//!
//! Provides persistent undo/redo history using immutable data structures.
//!
//! ## Design by Contract
//!
//! ### Preconditions
//! - P1: Documents pushed to history must be valid
//! - P2: Undo requires non-empty undo stack
//! - P3: Redo requires non-empty redo stack
//!
//! ### Postconditions
//! - Q1: Push clears redo stack (new timeline branch)
//! - Q2: Undo returns previous state and moves current to redo
//! - Q3: Redo returns next state and moves current to undo
//! - Q4: All operations return new History (immutable)
//! - Q5: History capped at `HistoryLimit` entries
//!
//! ### Invariants
//! - I1: Undo stack contains documents in reverse chronological order
//! - I2: Redo stack contains documents in chronological order
//! - I3: After push: redo stack is empty
//!
//! ## Performance
//!
//! Uses `rpds::List` for O(1) structural-sharing prepend and drop-first.
//! Truncation uses O(n) fold to rebuild when exceeding limit — acceptable
//! since history limit is small (100) and truncation is rare.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::document::DiagramDocument;
use rpds::List;

/// Newtype for history size limit - eliminates primitive obsession
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryLimit(usize);

impl HistoryLimit {
    /// Create a new `HistoryLimit` with the default value of 100
    #[must_use]
    pub const fn new() -> Self {
        Self(100)
    }

    /// Get the limit value
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }
}

impl Default for HistoryLimit {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent history using `rpds::List` for O(1) prepend and drop-first
/// with structural sharing (no element cloning).
#[derive(Clone, Default)]
pub struct History {
    undo_stack: List<DiagramDocument>,
    redo_stack: List<DiagramDocument>,
}

/// Maximum number of history entries to retain
const MAX_HISTORY: HistoryLimit = HistoryLimit(100);

/// Truncate a `rpds::List` to at most `limit` entries.
///
/// Since `rpds::List` is prepend-based, we collect into a vec,
/// reverse to get chronological order, take the first `limit`, then
/// fold back into a list in reverse (prepend order).
fn truncate_stack(stack: &List<DiagramDocument>, limit: HistoryLimit) -> List<DiagramDocument> {
    let limit_val = limit.value();
    if stack.len() <= limit_val {
        stack.clone()
    } else {
        stack
            .iter()
            .take(limit_val)
            .fold(List::new(), |acc, doc| acc.push_front(doc.clone()))
    }
}

/// Drop the first two elements via sequential `drop_first`.
fn drop_first_two(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    stack
        .drop_first()
        .and_then(|s| s.drop_first())
        .unwrap_or_default()
}

impl History {
    /// Creates a new empty history
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pure transition to push a new state
    #[must_use]
    pub fn push(&self, doc: DiagramDocument) -> Self {
        Self {
            undo_stack: self.undo_stack.push_front(doc),
            redo_stack: List::new(),
        }
        .tap_history_limit()
    }

    /// Pure transition to undo
    #[must_use]
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        let first = self.undo_stack.first()?;

        if first.revision == current.revision {
            let second = self
                .undo_stack
                .drop_first()
                .as_ref()
                .and_then(|s| s.first())
                .cloned()?;
            Some((
                second,
                Self {
                    undo_stack: drop_first_two(&self.undo_stack),
                    redo_stack: self.redo_stack.push_front(current),
                }
                .tap_history_limit(),
            ))
        } else {
            Some((
                first.clone(),
                Self {
                    undo_stack: self.undo_stack.drop_first().unwrap_or_default(),
                    redo_stack: self.redo_stack.push_front(current),
                }
                .tap_history_limit(),
            ))
        }
    }

    /// Pure transition to redo
    #[must_use]
    pub fn redo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        let first = self.redo_stack.first()?;

        if first.revision == current.revision {
            let second = self
                .redo_stack
                .drop_first()
                .as_ref()
                .and_then(|s| s.first())
                .cloned()?;
            Some((
                second,
                Self {
                    undo_stack: self.undo_stack.push_front(current),
                    redo_stack: drop_first_two(&self.redo_stack),
                }
                .tap_history_limit(),
            ))
        } else {
            Some((
                first.clone(),
                Self {
                    undo_stack: self.undo_stack.push_front(current),
                    redo_stack: self.redo_stack.drop_first().unwrap_or_default(),
                }
                .tap_history_limit(),
            ))
        }
    }

    /// Apply history limit to both stacks
    #[must_use]
    pub fn tap_history_limit(self) -> Self {
        Self {
            undo_stack: truncate_stack(&self.undo_stack, MAX_HISTORY),
            redo_stack: truncate_stack(&self.redo_stack, MAX_HISTORY),
        }
    }

    /// Check if undo is available
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of elements in the undo stack
    #[must_use]
    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of elements in the redo stack
    #[must_use]
    pub fn redo_stack_len(&self) -> usize {
        self.redo_stack.len()
    }
}

/// Re-export `truncate_stack` for tests (backward compatibility).
#[must_use]
pub fn truncate_stack_reexport(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    truncate_stack(stack, MAX_HISTORY)
}
