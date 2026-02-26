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

const MAX_HISTORY: usize = 100;

#[allow(clippy::needless_collect)]
fn truncate_stack(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let capped = stack.iter().take(MAX_HISTORY).cloned().collect::<Vec<_>>();
    capped
        .into_iter()
        .rev()
        .fold(List::new(), |acc, entry| acc.push_front(entry))
}

#[allow(clippy::needless_collect)]
fn drop_first(stack: &List<DiagramDocument>) -> List<DiagramDocument> {
    let remainder = stack.iter().skip(1).cloned().collect::<Vec<_>>();
    remainder
        .into_iter()
        .rev()
        .fold(List::new(), |acc, entry| acc.push_front(entry))
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
        }
        .tap_history_limit()
    }

    /// Pure transition to undo
    #[must_use]
    pub fn undo(&self, current: DiagramDocument) -> Option<(DiagramDocument, Self)> {
        self.undo_stack.first().map(|prev| {
            (
                prev.clone(),
                Self {
                    undo_stack: drop_first(&self.undo_stack),
                    redo_stack: self.redo_stack.push_front(current),
                }
                .tap_history_limit(),
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
                    redo_stack: drop_first(&self.redo_stack),
                }
                .tap_history_limit(),
            )
        })
    }

    #[must_use]
    pub fn tap_history_limit(self) -> Self {
        Self {
            undo_stack: truncate_stack(&self.undo_stack),
            redo_stack: truncate_stack(&self.redo_stack),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    use crate::models::document::{DiagramDocument, Revision};

    fn doc_with_revision(steps: u64) -> DiagramDocument {
        let mut revision = Revision::INITIAL;
        for _ in 0..steps {
            revision = revision.increment();
        }
        DiagramDocument {
            revision,
            ..DiagramDocument::default()
        }
    }

    #[test]
    fn given_more_than_cap_when_pushing_then_undo_stack_is_capped_at_100() {
        let history = (0..105_u64).fold(History::new(), |acc, step| {
            acc.push(doc_with_revision(step))
        });
        let current = doc_with_revision(10_000);

        let mut next = history;
        let mut undo_count = 0_usize;
        while let Some((doc, h)) = next.undo(current.clone()) {
            let _ = doc;
            undo_count += 1;
            next = h;
        }

        assert_eq!(undo_count, 100);
    }

    #[test]
    fn given_multiple_entries_when_undo_then_it_walks_back_in_order() {
        let history = History::new()
            .push(doc_with_revision(1))
            .push(doc_with_revision(2))
            .push(doc_with_revision(3));

        let current = doc_with_revision(4);
        let first_undo = history.undo(current);
        assert!(first_undo.is_some());
        let Some((first, history)) = first_undo else {
            return;
        };

        let second_undo = history.undo(first.clone());
        assert!(second_undo.is_some());
        let Some((second, history)) = second_undo else {
            return;
        };

        let third_undo = history.undo(second.clone());
        assert!(third_undo.is_some());
        let Some((third, _history)) = third_undo else {
            return;
        };

        assert_eq!(first.revision, doc_with_revision(3).revision);
        assert_eq!(second.revision, doc_with_revision(2).revision);
        assert_eq!(third.revision, doc_with_revision(1).revision);
    }

    #[test]
    fn given_cap_boundary_when_undo_and_redo_then_round_trip_is_sane() {
        let history = (0..100_u64).fold(History::new(), |acc, step| {
            acc.push(doc_with_revision(step))
        });
        let current = doc_with_revision(500);

        let undo_result = history.undo(current.clone());
        assert!(undo_result.is_some());
        let Some((latest, after_undo)) = undo_result else {
            return;
        };

        let redo_result = after_undo.redo(latest.clone());
        assert!(redo_result.is_some());
        let Some((restored, _after_redo)) = redo_result else {
            return;
        };

        assert_eq!(latest.revision, doc_with_revision(99).revision);
        assert_eq!(restored.revision, current.revision);
    }
}
