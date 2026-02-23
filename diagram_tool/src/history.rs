#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use rpds::List;

/// Persistent history using persistent data structures (rpds)
#[derive(Clone, Default)]
pub struct History {
    undo_stack: List<DiagramDocument>,
    redo_stack: List<DiagramDocument>,
}

impl History {
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
        }.tap_history_limit()
    }

    /// Pure transition to undo
    #[must_use]
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        self.undo_stack.first().map(|prev| {
            (
                prev.clone(),
                Self {
                    undo_stack: self.undo_stack.drop_first().iter().fold(List::new(), |_, l| l.clone()),
                    redo_stack: self.redo_stack.push_front(current),
                }
            )
        })
    }

    /// Pure transition to redo
    #[must_use]
    pub fn redo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        self.redo_stack.first().map(|next| {
            (
                next.clone(),
                Self {
                    undo_stack: self.undo_stack.push_front(current),
                    redo_stack: self.redo_stack.drop_first().iter().fold(List::new(), |_, l| l.clone()),
                }
            )
        })
    }

    #[must_use]
    pub const fn tap_history_limit(self) -> Self {
        self
    }
}
