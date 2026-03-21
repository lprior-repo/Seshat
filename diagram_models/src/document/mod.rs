//! Document model for diagram data structures.
//!
//! This module re-exports all document-related types and provides the main
//! `DiagramDocument` struct that holds the complete diagram state.

use im::HashMap;
use serde::{Deserialize, Serialize};

pub mod edge;
pub mod edge_direction;
pub mod editor;
pub mod error;
pub mod node;
pub mod types;

// Re-export for convenience
pub use edge::{ArrowType, Edge, EdgeStyle, SerializedPoint};
pub use editor::{EditorState, EditorTheme, GridError, GridSize, NonFiniteKind};
pub use error::DocumentError;
pub use node::{FontWeight, LockState, Node, NodeKind, NodeStyle};
pub use types::{AuthorId, EdgeId, NodeId, OrderedFloat, OrderedFloatError, Revision, Timestamp};

/// A validated rectangle for marquee selection.
/// Ensures width and height are non-negative.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct ValidRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ValidRect {
    /// Creates a new `ValidRect`, returning an error if dimensions are negative.
    ///
    /// # Errors
    /// Returns `DocumentError::InvalidMarqueeBounds` if width or height is negative.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, DocumentError> {
        if width < 0.0 || height < 0.0 {
            Err(DocumentError::InvalidMarqueeBounds)
        } else {
            Ok(Self {
                x,
                y,
                width,
                height,
            })
        }
    }
}

/// The main document structure containing all diagram data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DiagramDocument {
    pub version: u32,
    pub revision: Revision,
    pub document: DocumentData,
    #[serde(default)]
    pub editor_state: EditorState,
}

/// Container for nodes and edges
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DocumentData {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
}

impl Default for DiagramDocument {
    fn default() -> Self {
        Self {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }
}

impl DiagramDocument {
    /// Adds an edge to the document
    ///
    /// # Errors
    /// Returns `DocumentError::NodeNotFound` if source or target node doesn't exist.
    /// Returns `DocumentError::EdgeAlreadyExists` if edge ID already exists.
    pub fn add_edge(&mut self, edge_id: EdgeId, edge: Edge) -> Result<(), DocumentError> {
        if !self.document.nodes.contains_key(&edge.source) {
            return Err(DocumentError::NodeNotFound(edge.source));
        }
        if !self.document.nodes.contains_key(&edge.target) {
            return Err(DocumentError::NodeNotFound(edge.target));
        }
        if self.document.edges.contains_key(&edge_id) {
            return Err(DocumentError::EdgeAlreadyExists(edge_id));
        }
        self.document.edges.insert(edge_id, edge);
        Ok(())
    }

    /// Removes an edge from the document
    ///
    /// # Errors
    /// Returns `DocumentError::EdgeNotFound` if the edge does not exist.
    pub fn remove_edge(&mut self, edge_id: &EdgeId) -> Result<(), DocumentError> {
        if self.document.edges.remove(edge_id).is_none() {
            return Err(DocumentError::EdgeNotFound(edge_id.clone()));
        }
        Ok(())
    }

    /// Removes a node and cascades deletion to all connected edges
    ///
    /// # Errors
    /// Returns `DocumentError::NodeNotFound` if the node does not exist.
    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<(), DocumentError> {
        if self.document.nodes.remove(node_id).is_none() {
            return Err(DocumentError::NodeNotFound(node_id.clone()));
        }

        let edges_to_remove: std::vec::Vec<EdgeId> = self
            .document
            .edges
            .iter()
            .filter(|(_, edge)| edge.source == *node_id || edge.target == *node_id)
            .map(|(id, _)| id.clone())
            .collect();

        for edge_id in edges_to_remove {
            self.document.edges.remove(&edge_id);
        }

        Ok(())
    }

    /// Sets the source port anchor for an edge.
    ///
    /// # Errors
    /// Returns `PortError::EdgeNotFound` if the edge does not exist.
    /// Returns `PortError::NodeNotFound` if the source node does not exist.
    pub fn set_edge_source_port(
        &mut self,
        edge_id: &EdgeId,
        port: Option<crate::port::PortAnchor>,
    ) -> Result<(), crate::port::PortError> {
        let edge = self
            .document
            .edges
            .get_mut(edge_id)
            .ok_or(crate::port::PortError::EdgeNotFound)?;

        if !self.document.nodes.contains_key(&edge.source) {
            return Err(crate::port::PortError::NodeNotFound);
        }

        edge.source_port = port;
        Ok(())
    }

    /// Sets the target port anchor for an edge.
    ///
    /// # Errors
    /// Returns `PortError::EdgeNotFound` if the edge does not exist.
    /// Returns `PortError::NodeNotFound` if the target node does not exist.
    pub fn set_edge_target_port(
        &mut self,
        edge_id: &EdgeId,
        port: Option<crate::port::PortAnchor>,
    ) -> Result<(), crate::port::PortError> {
        let edge = self
            .document
            .edges
            .get_mut(edge_id)
            .ok_or(crate::port::PortError::EdgeNotFound)?;

        if !self.document.nodes.contains_key(&edge.target) {
            return Err(crate::port::PortError::NodeNotFound);
        }

        edge.target_port = port;
        Ok(())
    }

    /// Select nodes within a marquee rectangle.
    ///
    /// # Errors
    /// Returns `DocumentError` if the operation fails.
    pub fn select_marquee(
        &mut self,
        bounds: ValidRect,
        mode: crate::spatial_index::MarqueeMode,
    ) -> Result<(), DocumentError> {
        use crate::geometry::AABB;
        use crate::selection::bounds::{get_node_rotation, rotated_node_bounds};
        use crate::spatial_index::build_spatial_index;

        let index = build_spatial_index(&self.document.nodes);
        let marquee_aabb = AABB::new(
            bounds.x,
            bounds.y,
            bounds.x + bounds.width,
            bounds.y + bounds.height,
        );

        let m_right = bounds.x + bounds.width;
        let m_bottom = bounds.y + bounds.height;

        // Gather candidate nodes from spatial index
        let candidates = crate::spatial_index::gather_candidates(&index, &marquee_aabb);

        let mut selected = im::HashSet::new();
        for id in candidates {
            let node = self
                .document
                .nodes
                .get(&id)
                .ok_or_else(|| DocumentError::NodeNotFound(id.clone()))?;

            let rotation = get_node_rotation(node);
            let (min_x, min_y, max_x, max_y) =
                rotated_node_bounds(node.x.0, node.y.0, node.width.0, node.height.0, rotation);

            let is_selected = match mode {
                crate::spatial_index::MarqueeMode::Contain => {
                    min_x >= bounds.x && max_x <= m_right && min_y >= bounds.y && max_y <= m_bottom
                }
                crate::spatial_index::MarqueeMode::Intersect => {
                    !(min_x > m_right || max_x < bounds.x || min_y > m_bottom || max_y < bounds.y)
                }
            };

            if is_selected {
                selected.insert(id.to_string());
            }
        }

        self.editor_state.selected_items = selected;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
