import re

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

# Replace start_resize_interaction
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

# Replace finalize_motion_release
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

# Replace tests usage
content = content.replace(
    "InteractionMode::DraggingSelection {", 
    "InteractionMode::DragPending(DragPendingState {"
).replace(
    "did_move: true,\n        };",
    "}),\n        };\n        mode = InteractionMode::Dragging(DragState {\n            anchor_canvas: (0.0, 0.0),\n            original_positions: HashMap::new(),\n        });"
).replace(
    "did_move: false,\n        };",
    "}),\n        };"
).replace(
    "did_move: false, // No actual movement\n        };",
    "}),\n        };"
)

# Wait, `InteractionMode::DragPending(DragPendingState {` will break syntax if not closed.
# It's better to manually replace tests with regex or direct string replacements.
# Let's write the whole file back.
with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "w") as f:
    f.write(content)

print("Done phase 1")
