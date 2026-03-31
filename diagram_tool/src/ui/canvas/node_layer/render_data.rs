use diagram_models::document::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeInteractionState {
    #[default]
    Normal,
    Hovered,
    Selected {
        hovered: bool,
    },
    Editing,
}

impl NodeInteractionState {
    pub fn is_selected(self) -> bool {
        matches!(self, Self::Selected { .. } | Self::Editing)
    }

    pub fn is_hovered(self) -> bool {
        matches!(self, Self::Hovered | Self::Selected { hovered: true })
    }

    pub fn is_editing(self) -> bool {
        matches!(self, Self::Editing)
    }
}

/// Pre-computed render data extracted from a `Node`. Much cheaper to compare
/// via `PartialEq` than the full `Node` struct (which contains `HashMap<String, Value>`).
#[derive(Debug, Clone, PartialEq)]
pub struct NodeRenderData {
    pub kind: diagram_models::document::NodeKind,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub z_index: i64,
    pub font_size: Option<f64>,
    pub icon: String,
    pub tags: im::Vector<String>,
    pub style: Option<diagram_models::document::NodeStyle>,
}

impl NodeRenderData {
    pub fn from_node(node: &Node) -> Self {
        Self {
            kind: node.kind,
            label: node.label.clone(),
            x: node.x.0,
            y: node.y.0,
            width: node.width.0,
            height: node.height.0,
            z_index: node.z_index,
            font_size: node.font_size.map(|f| f.0),
            icon: node.icon.clone(),
            tags: node.tags.clone(),
            style: node.style.clone(),
        }
    }
}

/// Custom `PartialEq` for `NodeElementProps` that skips the heavy `node` field.
/// Dioxus uses `PartialEq` to decide if a child component needs re-rendering.
/// The `node` field is only used in event handlers (not rendering), so we only
/// need to compare `render_data` and `interaction_state` for diff correctness.
impl PartialEq for super::node_element::NodeElementProps {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.render_data == other.render_data
            && self.interaction_state == other.interaction_state
            && self.camera_x == other.camera_x
            && self.camera_y == other.camera_y
            && self.zoom == other.zoom
            // Signal handles are compared by identity (Copy pointer)
            && self.doc_signal == other.doc_signal
            && self.history_signal == other.history_signal
            && self.tool_signal == other.tool_signal
            && self.interaction_mode == other.interaction_mode
            && self.editor_state == other.editor_state
            && self.edit_value == other.edit_value
            && self.canvas_origin == other.canvas_origin
            && self.shift_pressed == other.shift_pressed
            && self.ctrl_pressed == other.ctrl_pressed
            && self.meta_pressed == other.meta_pressed
            && self.space_pressed == other.space_pressed
            && self.multi_touch_active == other.multi_touch_active
            && self.space_pan_active == other.space_pan_active
            && self.pending_pointer_sample == other.pending_pointer_sample
            && self.db_tx == other.db_tx
    }
}
