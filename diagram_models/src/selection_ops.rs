use crate::document::NodeId;
use im::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Item not found: {0}")]
    ItemNotFound(NodeId),
    #[error("Invalid interaction state: Cannot start marquee on a node")]
    InvalidInteractionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTestResult {
    Item(NodeId),
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeMode {
    Contain,
    Intersect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn contains_point(&self, p: &Point) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }

    #[must_use]
    pub fn contains_rect(&self, other: &Self) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.width >= other.x + other.width
            && self.y + self.height >= other.y + other.height
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiagramState {
    pub selected_items: HashSet<NodeId>,
    pub nodes: HashMap<NodeId, Rect>,
}

impl DiagramState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_nodes(nodes: HashMap<NodeId, Rect>) -> Self {
        Self {
            selected_items: HashSet::new(),
            nodes,
        }
    }
}

#[must_use]
pub fn hit_test(state: &DiagramState, point: &Point) -> HitTestResult {
    state
        .nodes
        .iter()
        .find(|(_, rect)| rect.contains_point(point))
        .map_or(HitTestResult::Empty, |(id, _)| {
            HitTestResult::Item(id.clone())
        })
}

/// Select an item by ID
///
/// # Errors
///
/// Returns `Error::ItemNotFound` if the `node_id` does not exist in the document.
pub fn select_item(state: &mut DiagramState, id: NodeId, mode: SelectionMode) -> Result<(), Error> {
    if !state.nodes.contains_key(&id) {
        return Err(Error::ItemNotFound(id));
    }

    let new_selection = match mode {
        SelectionMode::Replace => HashSet::unit(id),
        SelectionMode::Toggle => {
            if state.selected_items.contains(&id) {
                state.selected_items.without(&id)
            } else {
                state.selected_items.update(id)
            }
        }
    };

    state.selected_items = new_selection;

    // Invariants assertion
    debug_assert!(
        state
            .selected_items
            .iter()
            .all(|nid| state.nodes.contains_key(nid)),
        "Invariant I2: Selected items must exist in document"
    );

    Ok(())
}

/// Clear the current selection
///
/// # Errors
///
/// This function does not currently return an error, but returns `Result` to match contract.
pub fn clear_selection(state: &mut DiagramState) -> Result<(), Error> {
    state.selected_items = HashSet::new();
    Ok(())
}

/// Marquee select items
///
/// # Errors
///
/// Returns `Error::InvalidInteractionState` if the marquee starts on a node.
pub fn marquee_select(state: &mut DiagramState, start: Point, end: Point) -> Result<(), Error> {
    if let HitTestResult::Item(_) = hit_test(state, &start) {
        return Err(Error::InvalidInteractionState);
    }

    let marquee_rect = Rect::new(
        start.x.min(end.x),
        start.y.min(end.y),
        (start.x - end.x).abs(),
        (start.y - end.y).abs(),
    );

    let mode = if start.x <= end.x {
        MarqueeMode::Contain
    } else {
        MarqueeMode::Intersect
    };

    let selected = state
        .nodes
        .iter()
        .filter(|(_, rect)| match mode {
            MarqueeMode::Contain => marquee_rect.contains_rect(rect),
            MarqueeMode::Intersect => marquee_rect.intersects(rect),
        })
        .map(|(id, _)| id.clone())
        .collect::<HashSet<NodeId>>();

    state.selected_items = selected;

    // Invariants assertion
    debug_assert!(
        state
            .selected_items
            .iter()
            .all(|nid| state.nodes.contains_key(nid)),
        "Invariant I2: Selected items must exist in document"
    );

    Ok(())
}
