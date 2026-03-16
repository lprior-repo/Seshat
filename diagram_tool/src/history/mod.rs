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

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
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

/// Persistent history using persistent data structures (rpds)
#[derive(Clone, Default)]
pub struct History {
    undo_stack: List<DiagramDocument>,
    redo_stack: List<DiagramDocument>,
}

/// Maximum number of history entries to retain
const MAX_HISTORY: HistoryLimit = HistoryLimit(100);

/// Truncate stack to the given limit (uses default limit for backward compatibility)
fn truncate_stack_default(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    truncate_stack_with_limit(stack, MAX_HISTORY)
}

/// Truncate stack to at most `limit` entries
fn truncate_stack_with_limit(
    stack: &List<DiagramDocument>,
    limit: HistoryLimit,
) -> List<DiagramDocument> {
    let len = stack.len();
    if len <= limit.value() {
        return stack.clone();
    }
    stack.iter().take(limit.value()).cloned().collect()
}

/// Drops the first element from the list
fn drop_first(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let len = stack.len();
    if len <= 1 {
        return List::new();
    }
    stack.iter().skip(1).cloned().collect()
}

/// Drops the first two elements from the list (used when current matches first)
fn drop_first_two(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let len = stack.len();
    if len <= 2 {
        return List::new();
    }
    stack.iter().skip(2).cloned().collect()
}

/// Gets the second element from the list
fn second_element(stack: &List<DiagramDocument>) -> Option<DiagramDocument> {
    stack.iter().nth(1).cloned()
}

impl History {
    /// Creates a new empty history
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pure transition to push a new state
    ///
    /// This is the core state transition that adds a new document to history.
    /// Postcondition Q1: After push, `redo_stack` is empty (new timeline branch)
    #[must_use]
    pub fn push(&self, doc: DiagramDocument) -> Self {
        Self {
            undo_stack: self.undo_stack.push_front(doc),
            redo_stack: List::new(),
        }
        .tap_history_limit()
    }

    /// Pure transition to undo
    ///
    /// Returns the previous state from the undo stack.
    /// If `current` matches the most recent entry (first in stack),
    /// skips it and returns the next entry (the state before current).
    #[must_use]
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        let first = self.undo_stack.first()?;

        // Check if current matches the first element (most recent state)
        // If so, we need to return the second element (the state before current)
        if first.revision == current.revision {
            // Current is on the stack at position 0, return the second element
            let second = second_element(&self.undo_stack)?;
            Some((
                second,
                Self {
                    undo_stack: drop_first_two(&self.undo_stack),
                    redo_stack: self.redo_stack.push_front(current),
                }
                .tap_history_limit(),
            ))
        } else {
            // Current is not on the stack, return the first element
            Some((
                first.clone(),
                Self {
                    undo_stack: drop_first(&self.undo_stack),
                    redo_stack: self.redo_stack.push_front(current),
                }
                .tap_history_limit(),
            ))
        }
    }

    /// Pure transition to redo
    ///
    /// Returns the next state from the redo stack.
    /// If `current` matches the first entry in redo stack,
    /// skips it and returns the next entry.
    #[must_use]
    pub fn redo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        let first = self.redo_stack.first()?;

        // Check if current matches the first element
        if first.revision == current.revision {
            // Current is on the redo stack at position 0, return the second element
            let second = second_element(&self.redo_stack)?;
            Some((
                second,
                Self {
                    undo_stack: self.undo_stack.push_front(current),
                    redo_stack: drop_first_two(&self.redo_stack),
                }
                .tap_history_limit(),
            ))
        } else {
            // Current is not on the redo stack, return the first element
            Some((
                first.clone(),
                Self {
                    undo_stack: self.undo_stack.push_front(current),
                    redo_stack: drop_first(&self.redo_stack),
                }
                .tap_history_limit(),
            ))
        }
    }

    /// Apply history limit to both stacks
    #[must_use]
    pub fn tap_history_limit(self) -> Self {
        Self {
            undo_stack: truncate_stack_with_limit(&self.undo_stack, MAX_HISTORY),
            redo_stack: truncate_stack_with_limit(&self.redo_stack, MAX_HISTORY),
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
}

/// Re-export `truncate_stack` for tests (backward compatibility)
#[must_use]
pub fn truncate_stack(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    truncate_stack_default(stack)
}

pub mod tests;
