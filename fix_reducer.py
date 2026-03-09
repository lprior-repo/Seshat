import re

with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "r") as f:
    content = f.read()

# I will just restore the file from git to be safe and do a clean edit using python since the regex replacement messed it up.
import subprocess

subprocess.run(["git", "checkout", "diagram_tool/src/ui/canvas/interaction_reducer.rs"])

with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "r") as f:
    content = f.read()

# Replace InteractionMode definition
old_enum = """pub(super) enum InteractionMode {
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
        originals: HashMap<NodeId, (f64, f64, f64, f64),
        anchor: (f64, f64),
        did_resize: bool,
    },
    Panning {
        last_pos: (f64, f64),
    },
}"""
# Note: the original_bounds originals value is actually `originals: HashMap<NodeId, (f64, f64, f64, f64)>,`
old_enum = """pub(super) enum InteractionMode {
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
    },
    Panning {
        last_pos: (f64, f64),
    },
}"""

new_enum = """#[derive(Clone, Debug, PartialEq)]
pub(super) struct DragState {
    pub anchor_canvas: (f64, f64),
    pub original_positions: HashMap<NodeId, (f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct DragPendingState {
    pub anchor_canvas: (f64, f64),
    pub anchor_client: (f64, f64),
    pub original_positions: HashMap<NodeId, (f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ResizeState {
    pub handle: ResizeHandle,
    pub original_bounds: (f64, f64, f64, f64),
    pub originals: HashMap<NodeId, (f64, f64, f64, f64)>,
    pub anchor: (f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum InteractionMode {
    Select,
    RubberBand {
        start: (f64, f64),
        current: (f64, f64),
    },
    DragPending(DragPendingState),
    Dragging(DragState),
    DrawingEdge {
        from_node: NodeId,
        current_pos: (f64, f64),
    },
    DrawingSubgraph {
        start: (f64, f64),
        current: (f64, f64),
    },
    ResizePending(ResizeState),
    Resizing(ResizeState),
    Panning {
        last_pos: (f64, f64),
    },
}"""

content = content.replace(old_enum, new_enum)

old_start_resize = """        interaction_mode.set(InteractionMode::ResizingSelection {
            handle,
            original_bounds: bounds,
            originals,
            anchor: (cx, cy),
            did_resize: false,
        });"""

new_start_resize = """        interaction_mode.set(InteractionMode::ResizePending(ResizeState {
            handle,
            original_bounds: bounds,
            originals,
            anchor: (cx, cy),
        }));"""

content = content.replace(old_start_resize, new_start_resize)

old_finalize = """pub(super) fn finalize_motion_release(
    mode: &mut InteractionMode,
    doc: &mut DiagramDocument,
) -> bool {
    let should_increment = match mode {
        InteractionMode::DraggingSelection { did_move, .. } => Some(*did_move),
        InteractionMode::ResizingSelection { did_resize, .. } => Some(*did_resize),
        _ => None,
    };

    if let Some(increment) = should_increment {
        if increment {
            doc.revision = doc.revision.increment();
        }
        *mode = InteractionMode::Select;
        true
    } else {
        false
    }
}"""

new_finalize = """pub(super) fn finalize_motion_release(
    mode: &mut InteractionMode,
    doc: &mut DiagramDocument,
) -> bool {
    let should_increment = match mode {
        InteractionMode::Dragging(_) | InteractionMode::Resizing(_) => true,
        InteractionMode::DragPending(_) | InteractionMode::ResizePending(_) => {
            *mode = InteractionMode::Select;
            return true;
        }
        _ => return false,
    };

    if should_increment {
        doc.revision = doc.revision.increment();
        *mode = InteractionMode::Select;
        true
    } else {
        false
    }
}"""

content = content.replace(old_finalize, new_finalize)

# We need to replace all instances of InteractionMode::DraggingSelection in tests
content = re.sub(
    r"InteractionMode::DraggingSelection \{\s*anchor_canvas: (.*),\s*anchor_client: (.*),\s*original_positions: (.*),\s*did_move: true,\s*\}",
    r"InteractionMode::Dragging(DragState { anchor_canvas: \1, original_positions: \3 })",
    content,
)

content = re.sub(
    r"InteractionMode::DraggingSelection \{\s*anchor_canvas: (.*),\s*anchor_client: (.*),\s*original_positions: (.*),\s*did_move: false(?:, // No actual movement)?,\s*\}",
    r"InteractionMode::DragPending(DragPendingState { anchor_canvas: \1, anchor_client: \2, original_positions: \3 })",
    content,
)

content = re.sub(
    r"InteractionMode::ResizingSelection \{\s*handle: (.*),\s*original_bounds: (.*),\s*originals: (.*),\s*anchor: (.*),\s*did_resize: true,\s*\}",
    r"InteractionMode::Resizing(ResizeState { handle: \1, original_bounds: \2, originals: \3, anchor: \4 })",
    content,
)

content = re.sub(
    r"InteractionMode::ResizingSelection \{\s*handle: (.*),\s*original_bounds: (.*),\s*originals: (.*),\s*anchor: (.*),\s*did_resize: false,\s*\}",
    r"InteractionMode::ResizePending(ResizeState { handle: \1, original_bounds: \2, originals: \3, anchor: \4 })",
    content,
)

# And one special case in proptest
content = content.replace(
    "use super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle};",
    "use super::{finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle, DragPendingState, DragState, ResizeState};",
)

with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "w") as f:
    f.write(content)
