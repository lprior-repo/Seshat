use crate::document::{Node, NodeId};
use im::{HashMap, HashSet};
use thiserror::Error;

/// Maximum nesting depth for subgraphs
pub const MAX_SUBGRAPH_NESTING_DEPTH: usize = 5;

#[derive(Debug, Error, PartialEq)]
pub enum GroupingError {
    #[error("Selection is empty")]
    EmptySelection,
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("Locked nodes: {0:?}")]
    LockedNode(Vec<NodeId>),
    #[error("Subgraph too small: width={width}, height={height}")]
    SubgraphTooSmall { width: f64, height: f64 },
    #[error("Nested subgraph limit exceeded (max {0})")]
    NestedSubgraphLimitExceeded(usize),
    #[error("Invalid coordinates (NaN/Inf)")]
    InvalidCoordinates,
}

/// A validated set of nodes ready for grouping
pub struct ValidatedSelection(HashSet<NodeId>);

impl ValidatedSelection {
    /// Try to create a new validated selection.
    ///
    /// # Errors
    ///
    /// Returns `GroupingError` if selection is invalid.
    pub fn try_new(
        nodes: &HashMap<NodeId, Node>,
        selected_ids: &HashSet<NodeId>,
    ) -> Result<Self, GroupingError> {
        if selected_ids.is_empty() {
            return Err(GroupingError::EmptySelection);
        }

        for id in selected_ids {
            if !nodes.contains_key(id) {
                return Err(GroupingError::NodeNotFound(id.clone()));
            }
        }

        let locked = find_locked_nodes(nodes, selected_ids);
        if !locked.is_empty() {
            return Err(GroupingError::LockedNode(locked));
        }

        if !check_nesting_depth(nodes, selected_ids) {
            return Err(GroupingError::NestedSubgraphLimitExceeded(
                MAX_SUBGRAPH_NESTING_DEPTH,
            ));
        }

        Ok(Self(selected_ids.clone()))
    }

    #[must_use]
    pub const fn inner(&self) -> &HashSet<NodeId> {
        &self.0
    }
}

/// Validates the selection.
///
/// # Errors
///
/// Returns `GroupingError` if selection is invalid.
pub fn validate_selection(
    nodes: &HashMap<NodeId, Node>,
    selected: &HashSet<String>,
) -> Result<(), GroupingError> {
    let ids: HashSet<NodeId> = selected.iter().map(|s| NodeId::new(s.clone())).collect();
    ValidatedSelection::try_new(nodes, &ids)?;
    Ok(())
}

/// Validate nodes are not locked - returns all locked node IDs
fn find_locked_nodes(nodes: &HashMap<NodeId, Node>, selected: &HashSet<NodeId>) -> Vec<NodeId> {
    selected
        .iter()
        .filter_map(|id| {
            nodes
                .get(id)
                .and_then(|node| node.lock_state.is_locked().then_some(id.clone()))
        })
        .collect()
}

/// Count the nesting depth of a node's parent chain
#[must_use]
pub fn count_nesting_depth(nodes: &HashMap<NodeId, Node>, parent: Option<&NodeId>) -> usize {
    parent.and_then(|pid| nodes.get(pid)).map_or(0, |node| {
        1 + count_nesting_depth(nodes, node.parent.as_ref())
    })
}

/// Check nesting depth doesn't exceed limit
fn check_nesting_depth(nodes: &HashMap<NodeId, Node>, selected: &HashSet<NodeId>) -> bool {
    selected.iter().all(|id| {
        nodes.get(id).is_none_or(|node| {
            count_nesting_depth(nodes, node.parent.as_ref()) < MAX_SUBGRAPH_NESTING_DEPTH
        })
    })
}

/// Validate coordinates are valid (not NaN or Infinity)
#[must_use]
pub fn validate_coordinates(min_x: f64, min_y: f64, width: f64, height: f64) -> bool {
    min_x.is_finite()
        && min_y.is_finite()
        && width.is_finite()
        && height.is_finite()
        && width > 0.0
        && height > 0.0
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
