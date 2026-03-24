//! Z-order core logic for diagram operations
//!
//! This module provides the pure logic for reordering node IDs based on z-index
//! operations (bring forward, send backward, bring to front, send to back).

use crate::document::NodeId;
use std::collections::BTreeSet;

/// Z-order operation variants
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZOrderOp {
    BringForward,
    SendBackward,
    BringToFront,
    SendToBack,
}

/// Pure function: Reorders a list of IDs based on a z-order operation and a selection set.
///
/// This function performs the relative reordering of IDs in-place.
/// It assumes the input `ids` are already sorted by their current z-index.
pub fn apply_z_order_reorder(ids: &mut Vec<NodeId>, selected: &BTreeSet<NodeId>, op: ZOrderOp) {
    if ids.len() < 2 {
        return;
    }

    match op {
        ZOrderOp::BringForward => {
            for idx in (0..(ids.len() - 1)).rev() {
                let current_selected = selected.contains(&ids[idx]);
                let next_selected = selected.contains(&ids[idx + 1]);
                if current_selected && !next_selected {
                    ids.swap(idx, idx + 1);
                }
            }
        }
        ZOrderOp::SendBackward => {
            for idx in 1..ids.len() {
                let current_selected = selected.contains(&ids[idx]);
                let previous_selected = selected.contains(&ids[idx - 1]);
                if current_selected && !previous_selected {
                    ids.swap(idx - 1, idx);
                }
            }
        }
        ZOrderOp::BringToFront => {
            let mut reordered = ids
                .iter()
                .filter(|id| !selected.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            reordered.extend(ids.iter().filter(|id| selected.contains(*id)).cloned());
            *ids = reordered;
        }
        ZOrderOp::SendToBack => {
            let mut reordered = ids
                .iter()
                .filter(|id| selected.contains(*id))
                .cloned()
                .collect::<Vec<_>>();
            reordered.extend(ids.iter().filter(|id| !selected.contains(*id)).cloned());
            *ids = reordered;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bring_forward() {
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let mut ids = vec![n1.clone(), n2.clone(), n3.clone()];
        let mut selected = BTreeSet::new();
        selected.insert(n1.clone());

        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::BringForward);
        assert_eq!(ids, vec![n2.clone(), n1.clone(), n3.clone()]);

        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::BringForward);
        assert_eq!(ids, vec![n2.clone(), n3.clone(), n1.clone()]);

        // Already at front
        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::BringForward);
        assert_eq!(ids, vec![n2.clone(), n3.clone(), n1.clone()]);
    }

    #[test]
    fn test_send_backward() {
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let mut ids = vec![n1.clone(), n2.clone(), n3.clone()];
        let mut selected = BTreeSet::new();
        selected.insert(n3.clone());

        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::SendBackward);
        assert_eq!(ids, vec![n1.clone(), n3.clone(), n2.clone()]);

        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::SendBackward);
        assert_eq!(ids, vec![n3.clone(), n1.clone(), n2.clone()]);
    }

    #[test]
    fn test_bring_to_front() {
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let mut ids = vec![n1.clone(), n2.clone(), n3.clone()];
        let mut selected = BTreeSet::new();
        selected.insert(n1.clone());

        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::BringToFront);
        assert_eq!(ids, vec![n2.clone(), n3.clone(), n1.clone()]);
    }

    #[test]
    fn test_send_to_back() {
        let n1 = NodeId::new("n1".to_string());
        let n2 = NodeId::new("n2".to_string());
        let n3 = NodeId::new("n3".to_string());
        let mut ids = vec![n1.clone(), n2.clone(), n3.clone()];
        let mut selected = BTreeSet::new();
        selected.insert(n3.clone());

        apply_z_order_reorder(&mut ids, &selected, ZOrderOp::SendToBack);
        assert_eq!(ids, vec![n3.clone(), n1.clone(), n2.clone()]);
    }
}
