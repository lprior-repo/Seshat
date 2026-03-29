#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::stubs::DispatchError;
use im::HashMap;

use diagram_models::document::NodeId;

/// Error type for pure domain label edit calculations.
///
/// This error type is used by `calculate_*` functions that perform
/// pure domain transformations without any I/O or persistence concerns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelEditError {
    /// Target node or edge not found in document
    TargetNotFound,
    /// Invalid input data provided (e.g., label too long, invalid characters)
    ValidationError,
}

/// Error type for `commit_inline_edit` operations (Action layer).
///
/// This error type wraps `LabelEditError` for pure domain failures
/// and adds action-specific errors like persistence failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    /// Pure domain error during label edit calculation
    LabelEdit(LabelEditError),
    /// The system failed to persist the new label
    UpdateFailed(DispatchError),
}

impl From<LabelEditError> for CommitError {
    fn from(err: LabelEditError) -> Self {
        Self::LabelEdit(err)
    }
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
        start_port: Option<diagram_models::port::PortAnchor>,
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
    DraggingBendPoint {
        edge_id: diagram_models::document::EdgeId,
        bend_index: usize,
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
