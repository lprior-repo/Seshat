//! Centralized grouping module - re-exports from `diagram_models`
//!
//! This module re-exports all grouping operations from `diagram_models::grouping`
//! to maintain API compatibility with existing code.

pub use diagram_models::grouping::{
    calculate_bounding_box, calculate_edge_cleanup, calculate_ungroup, compute_padded_bounds,
    create_subgraph_node, find_lca, group_selection, ungroup_selection, validate_coordinates,
    validate_selection, GroupingError, ValidatedSelection,
};

#[cfg(test)]
#[path = "../grouping_tests.rs"]
mod tests;
