use crate::document::DiagramDocument;
use crate::geometry::Point;
use crate::selection::types::{ElementId, SelectionError};
use crate::spatial_index::{build_spatial_index, point_query};

/// Performs a hit test at the given point.
///
/// Uses spatial indexing for O(log n) average case instead of scanning all nodes.
///
/// # Errors
///
/// Currently infallible but returns Result for signature compatibility.
pub fn hit_test(
    point: &Point,
    document: &DiagramDocument,
) -> Result<Option<ElementId>, SelectionError> {
    // Build spatial index once for the document
    let index = build_spatial_index(&document.document.nodes);

    // Use point query to find topmost node at this position
    let hit = point_query(&index, &document.document.nodes, point)
        .filter(|id| {
            // Check visibility - node is guaranteed to exist since point_query returned it
            document
                .document
                .nodes
                .get(id)
                .is_some_and(|node| node.is_visible())
        })
        .map(ElementId::Node);

    Ok(hit)
}
