//! Subgraph module
//!
//! Provides operations for subgraph containers, node grouping/reparenting,
//! and transform operations on groups of nodes.
//!
//! ## Module Structure
//!
//! - [`types`](types) - Core types: BoundingBox, Padding, PositiveScale, Error
//! - [`reparenting`](reparenting) - Node parent setting and cycle detection
//! - [`grouping`](grouping) - Create/ungroup subgraphs
//! - [`collapse`](collapse) - Toggle collapsed state
//! - [`transform`](transform) - Scale operations on groups
//! - [`selection`](selection) - Hit-testing and selection evaluation

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod collapse;
pub mod grouping;
pub mod reparenting;
pub mod selection;
pub mod transform;
pub mod types;

pub use types::CanvasState;
pub use types::{
    apply_viewport_transform, calculate_container_bounds, create_empty_subgraph, BoundingBox,
    Error, Padding, PositiveScale,
};

pub use collapse::toggle_collapse;
pub use grouping::{create_subgraph_from_nodes, group_nodes, ungroup_nodes};
pub use reparenting::{set_node_parent, unparent_node};
pub use selection::{evaluate_selection, SelectionModifiers, SelectionResult};
pub use transform::{scale_group, GroupTransformError, Subgraph};

#[cfg(test)]
#[path = "subgraph_tests.rs"]
mod tests;
