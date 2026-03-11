use crate::models::document::{DiagramDocument, Node, NodeId, OrderedFloat};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Empty selection")]
    EmptySelection,
    #[error("Item is locked")]
    ItemLocked,
    #[error("Invalid hierarchy")]
    InvalidHierarchy,
    #[error("Postcondition violated")]
    PostconditionViolated,
    #[error("Node not found")]
    NodeNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> NonEmptyVec<T> {
    pub fn try_from(vec: Vec<T>) -> Result<Self, Error> {
        if vec.is_empty() {
            Err(Error::EmptySelection)
        } else {
            Ok(Self(vec))
        }
    }

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClipboardData {
    pub nodes: Vec<Node>,
}

fn check_locked_items(doc: &DiagramDocument, selection: &[NodeId]) -> Result<(), Error> {
    for id in selection {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        if node.locked {
            return Err(Error::ItemLocked);
        }
    }
    Ok(())
}

fn check_invalid_hierarchy(doc: &DiagramDocument, selection: &[NodeId]) -> Result<(), Error> {
    let selected_set: HashSet<_> = selection.iter().collect();

    for id in selection {
        let mut current_id = id;
        loop {
            let node = doc
                .document
                .nodes
                .get(current_id)
                .ok_or(Error::NodeNotFound)?;
            if let Some(parent_id) = &node.parent {
                if selected_set.contains(parent_id) {
                    return Err(Error::InvalidHierarchy);
                }
                current_id = parent_id;
            } else {
                break;
            }
        }
    }
    Ok(())
}

pub fn move_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    delta: Vector2D,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;
    check_invalid_hierarchy(doc, selection_slice)?;

    for id in selection_slice {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            node.x = node.x + OrderedFloat::new_unchecked(delta.x);
            node.y = node.y + OrderedFloat::new_unchecked(delta.y);
        } else {
            return Err(Error::NodeNotFound);
        }
    }

    Ok(())
}

pub fn resize_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
    new_bounds: Rect,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;
    check_invalid_hierarchy(doc, selection_slice)?;

    // Q: Does resizing a mixed selection of lines and rectangles proportionally scale the lines' endpoints?
    // A: In a complete implementation, this would compute the old bounding box and map each node's position and size.
    // For this minimal implementation per contract:

    // Calculate old bounds to determine scale factors
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for id in selection_slice {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        let nx = node.x.0;
        let ny = node.y.0;
        let nw = node.width.0;
        let nh = node.height.0;

        min_x = min_x.min(nx);
        min_y = min_y.min(ny);
        max_x = max_x.max(nx + nw);
        max_y = max_y.max(ny + nh);
    }

    let old_width = max_x - min_x;
    let old_height = max_y - min_y;

    if old_width == 0.0 || old_height == 0.0 {
        return Ok(()); // Or return an error depending on semantics, we will ignore scaling zero size
    }

    let scale_x = new_bounds.width / old_width;
    let scale_y = new_bounds.height / old_height;

    for id in selection_slice {
        if let Some(node) = doc.document.nodes.get_mut(id) {
            let relative_x = node.x.0 - min_x;
            let relative_y = node.y.0 - min_y;

            node.x = OrderedFloat::new_unchecked(new_bounds.x + relative_x * scale_x);
            node.y = OrderedFloat::new_unchecked(new_bounds.y + relative_y * scale_y);
            node.width = OrderedFloat::new_unchecked(node.width.0 * scale_x);
            node.height = OrderedFloat::new_unchecked(node.height.0 * scale_y);
        }
    }

    Ok(())
}

pub fn delete_selection(
    doc: &mut DiagramDocument,
    selection: NonEmptyVec<NodeId>,
) -> Result<(), Error> {
    let selection_slice = selection.as_slice();
    check_locked_items(doc, selection_slice)?;

    for id in selection_slice {
        if !doc.document.nodes.contains_key(id) {
            return Err(Error::NodeNotFound);
        }
    }

    for id in selection_slice {
        doc.document.nodes.remove(id);
        doc.editor_state.selected_items.remove(&id.to_string());
    }

    for id in selection_slice {
        if doc.document.nodes.contains_key(id) {
            return Err(Error::PostconditionViolated);
        }
    }

    Ok(())
}

pub fn copy_selection(
    doc: &DiagramDocument,
    selection: NonEmptyVec<NodeId>,
) -> Result<ClipboardData, Error> {
    let selection_slice = selection.as_slice();
    let mut copied_nodes = Vec::new();

    for id in selection_slice {
        let node = doc.document.nodes.get(id).ok_or(Error::NodeNotFound)?;
        copied_nodes.push(node.clone());
    }

    Ok(ClipboardData {
        nodes: copied_nodes,
    })
}

pub fn paste_selection(
    doc: &mut DiagramDocument,
    clipboard: &ClipboardData,
    offset: Vector2D,
) -> Result<Vec<NodeId>, Error> {
    let mut new_ids = Vec::new();

    // Clear current selection
    doc.editor_state.selected_items.clear();

    for node in &clipboard.nodes {
        let mut new_node = node.clone();
        new_node.x = new_node.x + OrderedFloat::new_unchecked(offset.x);
        new_node.y = new_node.y + OrderedFloat::new_unchecked(offset.y);

        // Generate a new ID (in a real app, use UUID. Here we just append a counter or timestamp, but since we are pure...)
        // To be functional and deterministic, let's append "_copy" or use something from the document
        // We will just do a simple iteration
        let mut idx = 1;
        let mut new_id;
        loop {
            new_id = NodeId::new(format!("{}_{idx}", node.label)); // simplistic
            if !doc.document.nodes.contains_key(&new_id) {
                break;
            }
            idx += 1;
        }

        doc.document.nodes.insert(new_id.clone(), new_node);
        doc.editor_state.selected_items.insert(new_id.to_string());
        new_ids.push(new_id);
    }

    Ok(new_ids)
}
