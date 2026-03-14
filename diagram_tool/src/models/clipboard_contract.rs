use crate::models::document::{DiagramDocument, Edge, EdgeId, Node, NodeId, OrderedFloat};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Empty clipboard")]
    EmptyClipboard,
    #[error("Invalid clipboard data")]
    InvalidClipboardData,
    #[error("Duplicate ID created")]
    DuplicateIdCreated,
    #[error("Invalid edge reference")]
    InvalidEdgeReference,
    #[error("Invalid parent reference")]
    InvalidParentReference,
    #[error("Postcondition violated: {0}")]
    PostconditionViolated(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub nodes: Vec<NodeId>,
}

impl Selection {
    #[must_use]
    pub fn empty() -> Self {
        Self { nodes: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    pub nodes: Vec<(NodeId, Node)>,
    pub edges: Vec<(EdgeId, Edge)>,
}

impl ClipboardData {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteResult {
    pub new_nodes: Vec<NodeId>,
    pub new_edges: Vec<EdgeId>,
}

/// Copies the selected nodes and their connecting edges to the clipboard.
///
/// # Errors
/// Returns `Error::EmptySelection` if the selection is empty.
/// Returns `Error::PostconditionViolated` if a selected node does not exist in the document.
pub fn copy(selection: &Selection, doc: &DiagramDocument) -> Result<ClipboardData, Error> {
    if selection.nodes.is_empty() {
        return Err(Error::EmptySelection);
    }

    let selected_set: HashSet<_> = selection.nodes.iter().collect();

    let nodes = selection
        .nodes
        .iter()
        .map(|id| {
            doc.document
                .nodes
                .get(id)
                .map(|node| (id.clone(), node.clone()))
                .ok_or_else(|| {
                    Error::PostconditionViolated(format!(
                        "Selected node {id} not found in document"
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let edges = doc
        .document
        .edges
        .iter()
        .filter(|(_, edge)| {
            selected_set.contains(&edge.source) && selected_set.contains(&edge.target)
        })
        .map(|(id, edge)| (id.clone(), edge.clone()))
        .collect();

    Ok(ClipboardData { nodes, edges })
}

/// Cuts the selected nodes from the document and places them in the clipboard.
///
/// # Errors
/// Returns `Error::EmptySelection` if the selection is empty.
pub fn cut(selection: &Selection, doc: &mut DiagramDocument) -> Result<ClipboardData, Error> {
    let clipboard = copy(selection, doc)?;

    selection.nodes.iter().for_each(|id| {
        let _ = doc.remove_node(id);
        doc.editor_state.selected_items.remove(id.as_str());
    });

    Ok(clipboard)
}

/// Pastes the clipboard contents into the document, applying an offset and regenerating IDs.
///
/// # Errors
/// Returns `Error::EmptyClipboard` if the clipboard is empty.
/// Returns `Error::DuplicateIdCreated` if a newly generated ID collides with an existing one.
/// Returns `Error::InvalidParentReference` if a pasted node points to a non-existent parent.
/// Returns `Error::InvalidEdgeReference` if a pasted edge points to a non-existent node.
pub fn paste(
    clipboard: &ClipboardData,
    doc: &mut DiagramDocument,
    paste_serial: u32,
) -> Result<PasteResult, Error> {
    if clipboard.nodes.is_empty() {
        return Err(Error::EmptyClipboard);
    }

    let offset_val = 20.0 * f64::from(paste_serial);

    // Generate new nodes mapped with their old IDs
    let new_nodes_mapped = clipboard
        .nodes
        .iter()
        .map(|(old_id, node)| {
            let new_id = NodeId::new(Uuid::new_v4().to_string());
            if doc.document.nodes.contains_key(&new_id) {
                return Err(Error::DuplicateIdCreated);
            }
            Ok((
                old_id.clone(),
                new_id,
                Node {
                    x: OrderedFloat::new_unchecked(node.x.0 + offset_val),
                    y: OrderedFloat::new_unchecked(node.y.0 + offset_val),
                    ..node.clone()
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let id_map: HashMap<NodeId, NodeId> = new_nodes_mapped
        .iter()
        .map(|(old_id, new_id, _)| (old_id.clone(), new_id.clone()))
        .collect();

    // Remap parents
    let mapped_nodes = new_nodes_mapped
        .into_iter()
        .map(|(_, new_id, node)| {
            let remapped_parent = match node.parent.as_ref() {
                Some(parent) => {
                    if let Some(new_parent) = id_map.get(parent) {
                        Some(new_parent.clone())
                    } else if doc.document.nodes.contains_key(parent) {
                        Some(parent.clone())
                    } else {
                        return Err(Error::InvalidParentReference);
                    }
                }
                None => None,
            };
            Ok((
                new_id,
                Node {
                    parent: remapped_parent,
                    ..node
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Prepare edges
    let new_edges_mapped = clipboard
        .edges
        .iter()
        .map(|(_, edge)| {
            let is_remapped_source = id_map.contains_key(&edge.source);
            let is_remapped_target = id_map.contains_key(&edge.target);

            let new_source = if is_remapped_source {
                id_map.get(&edge.source).unwrap_or(&edge.source).clone()
            } else {
                edge.source.clone()
            };

            let new_target = if is_remapped_target {
                id_map.get(&edge.target).unwrap_or(&edge.target).clone()
            } else {
                edge.target.clone()
            };

            if !is_remapped_source && !doc.document.nodes.contains_key(&new_source) {
                return Err(Error::InvalidEdgeReference);
            }
            if !is_remapped_target && !doc.document.nodes.contains_key(&new_target) {
                return Err(Error::InvalidEdgeReference);
            }

            let new_edge_id = EdgeId::new(Uuid::new_v4().to_string());
            if doc.document.edges.contains_key(&new_edge_id) {
                return Err(Error::DuplicateIdCreated);
            }

            Ok((
                new_edge_id,
                Edge {
                    source: new_source,
                    target: new_target,
                    ..edge.clone()
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Perform the mutation at the boundary
    let pasted_node_ids = mapped_nodes
        .into_iter()
        .map(|(new_id, new_node)| {
            doc.document.nodes.insert(new_id.clone(), new_node);
            new_id
        })
        .collect::<Vec<_>>();

    let pasted_edge_ids = new_edges_mapped
        .into_iter()
        .map(|(new_id, new_edge)| {
            doc.document.edges.insert(new_id.clone(), new_edge);
            new_id
        })
        .collect::<Vec<_>>();

    doc.editor_state.selected_items.clear();
    pasted_node_ids.iter().for_each(|id| {
        doc.editor_state
            .selected_items
            .insert(id.as_str().to_string());
    });

    Ok(PasteResult {
        new_nodes: pasted_node_ids,
        new_edges: pasted_edge_ids,
    })
}
