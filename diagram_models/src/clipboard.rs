//! Canonical clipboard data type and operations for diagramming.
//!
//! This module provides the single source of truth for clipboard functionality,
//! ensuring that both nodes and edges are correctly preserved during operations.

use crate::document::{DiagramDocument, Edge, EdgeId, Node, NodeId, OrderedFloat};
use crate::geometry::Coordinate;
use crate::subgraph::LayoutConstants;
use im::HashMap;
use uuid::Uuid;

/// Results of a paste operation calculation.
pub struct PasteResult {
    /// The updated node map
    pub nodes: HashMap<NodeId, Node>,
    /// The updated edge map
    pub edges: HashMap<EdgeId, Edge>,
    /// Mapping from old node IDs to new ones
    pub id_map: HashMap<NodeId, NodeId>,
    /// Set of newly created (pasted) item IDs
    pub selected: im::HashSet<String>,
}

/// Pure clipboard data type - immutable state for clipboard operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardData {
    /// The nodes that were copied to the clipboard, preserving their original IDs
    pub nodes: Vec<(NodeId, Node)>,
    /// The edges that were copied to the clipboard
    pub edges: Vec<Edge>,
    /// Serial number for tracking paste operations (for offset calculation)
    pub paste_serial: u32,
}

impl ClipboardData {
    /// Creates a new empty clipboard
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            paste_serial: 0,
        }
    }

    /// Returns true if the clipboard has content that can be pasted
    #[must_use]
    pub const fn has_content(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Prepares the clipboard for a paste operation by incrementing the serial
    #[must_use]
    pub const fn prepare_paste(mut self) -> Self {
        self.paste_serial = self.paste_serial.saturating_add(1);
        self
    }
}

impl Default for ClipboardData {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure function: Creates a clipboard from the given selection in the document.
#[must_use]
pub fn copy_selection(doc: &DiagramDocument, selection: &[NodeId]) -> Option<ClipboardData> {
    if selection.is_empty() {
        return None;
    }

    let nodes = collect_clipboard_nodes(doc, selection);
    let edges = collect_clipboard_edges(doc, selection);

    Some(ClipboardData {
        nodes,
        edges,
        paste_serial: 0,
    })
}

fn collect_clipboard_nodes(doc: &DiagramDocument, selection: &[NodeId]) -> Vec<(NodeId, Node)> {
    selection
        .iter()
        .filter_map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.clone()))
        })
        .collect()
}

fn collect_clipboard_edges(doc: &DiagramDocument, selection: &[NodeId]) -> Vec<Edge> {
    doc.document
        .edges
        .values()
        .filter(|edge| selection.contains(&edge.source) && selection.contains(&edge.target))
        .cloned()
        .collect()
}

/// Pure function: Calculates the results of pasting clipboard contents.
///
/// Returns a `PasteResult` containing (`updated_nodes`, `updated_edges`, `id_map`, `selected`).
#[must_use]
pub fn calculate_paste(clipboard: &ClipboardData, doc: &DiagramDocument) -> PasteResult {
    let offset = calculate_paste_offset(clipboard.paste_serial);
    let id_map = generate_id_map(&clipboard.nodes);

    let nodes = paste_nodes(&clipboard.nodes, &doc.document.nodes, offset, &id_map);
    let edges = paste_edges(&clipboard.edges, &doc.document.edges, &id_map);
    let selected = collect_pasted_ids(&id_map);

    PasteResult {
        nodes,
        edges,
        id_map,
        selected,
    }
}

fn calculate_paste_offset(serial: u32) -> Coordinate {
    let multiplier = f64::from(serial.saturating_add(1).max(1));
    LayoutConstants::PASTE_OFFSET * multiplier
}

fn generate_id_map(nodes: &[(NodeId, Node)]) -> HashMap<NodeId, NodeId> {
    nodes
        .iter()
        .map(|(old_id, _)| (old_id.clone(), NodeId::new(Uuid::new_v4().to_string())))
        .collect()
}

fn paste_nodes(
    clipboard_nodes: &[(NodeId, Node)],
    existing_nodes: &HashMap<NodeId, Node>,
    offset: Coordinate,
    id_map: &HashMap<NodeId, NodeId>,
) -> HashMap<NodeId, Node> {
    clipboard_nodes
        .iter()
        .fold(existing_nodes.clone(), |acc, (old_id, node)| {
            if let Some(new_id) = id_map.get(old_id).cloned() {
                acc.update(new_id, create_pasted_node(node, offset, id_map))
            } else {
                acc
            }
        })
}

fn paste_edges(
    clipboard_edges: &[Edge],
    existing_edges: &HashMap<EdgeId, Edge>,
    id_map: &HashMap<NodeId, NodeId>,
) -> HashMap<EdgeId, Edge> {
    clipboard_edges
        .iter()
        .fold(existing_edges.clone(), |acc, edge| {
            if let (Some(new_source), Some(new_target)) =
                (id_map.get(&edge.source), id_map.get(&edge.target))
            {
                let next = create_pasted_edge(edge, new_source, new_target);
                acc.update(EdgeId::new(Uuid::new_v4().to_string()), next)
            } else {
                acc
            }
        })
}

fn collect_pasted_ids(id_map: &HashMap<NodeId, NodeId>) -> im::HashSet<String> {
    id_map
        .values()
        .map(std::string::ToString::to_string)
        .collect()
}

fn create_pasted_node(node: &Node, offset: Coordinate, id_map: &HashMap<NodeId, NodeId>) -> Node {
    let mut next = node.clone();
    next.x = OrderedFloat(next.x.0 + offset.0);
    next.y = OrderedFloat(next.y.0 + offset.0);
    next.parent = next
        .parent
        .and_then(|pid| id_map.get(&pid).cloned().or(Some(pid)));
    next
}

fn create_pasted_edge(edge: &Edge, source: &NodeId, target: &NodeId) -> Edge {
    let mut next = edge.clone();
    next.source = source.clone();
    next.target = target.clone();
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clipboard_is_empty() {
        let clipboard = ClipboardData::new();
        assert!(!clipboard.has_content());
        assert_eq!(clipboard.paste_serial, 0);
    }
}
