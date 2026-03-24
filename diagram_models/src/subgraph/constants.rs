//! Domain constants for layout and positioning.

use crate::geometry::Coordinate;

/// Domain-specific constants for diagram layout.
pub struct LayoutConstants;

impl LayoutConstants {
    /// Default offset for paste operations.
    pub const PASTE_OFFSET: Coordinate = Coordinate::new_unchecked(20.0);

    /// Default padding for subgraph containers.
    pub const SUBGRAPH_PADDING: Coordinate = Coordinate::new_unchecked(24.0);
}
