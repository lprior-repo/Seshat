#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::stubs::DispatchError;
use im::HashMap;

use diagram_models::document::NodeId;

/// Error type for `commit_inline_edit` operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    /// Dispatch failed (e.g., channel closed)
    DispatchFailed(DispatchError),
    /// Target node or edge not found in document
    TargetNotFound,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InteractionMode {
    Select,
    RubberBand {
        start: (f64, f64),
        current: (f64, f64),
    },
    DraggingSelection {
        anchor_canvas: (f64, f64),
        anchor_client: (f64, f64),
        original_positions: HashMap<NodeId, (f64, f64)>,
        did_move: bool,
    },
    DrawingEdge {
        from_node: NodeId,
        current_pos: (f64, f64),
    },
    DrawingSubgraph {
        start: (f64, f64),
        current: (f64, f64),
    },
    ResizingSelection {
        handle: ResizeHandle,
        original_bounds: (f64, f64, f64, f64),
        originals: HashMap<NodeId, (f64, f64, f64, f64)>,
        anchor: (f64, f64),
        did_resize: bool,
        aspect_ratio: Option<f64>,
    },
    Panning {
        last_pos: (f64, f64),
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}
