#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use dioxus::prelude::*;
use im::HashMap;

use crate::stubs::{dispatch_node_resize, ResizeBounds};
use diagram_models::document::{DiagramDocument, NodeId};
use diagram_models::envelope::EventEnvelope;

use super::types::InteractionMode;

pub fn finalize_motion_release(
    mode: &mut InteractionMode,
    doc: &mut DiagramDocument,
    db_tx: &Option<Coroutine<EventEnvelope>>,
) -> bool {
    let did_resize = matches!(
        mode,
        InteractionMode::ResizingSelection {
            did_resize: true,
            ..
        }
    );

    let originals: HashMap<NodeId, (f64, f64, f64, f64)> = match mode {
        InteractionMode::ResizingSelection { originals, .. } => originals.clone(),
        _ => HashMap::new(),
    };

    let should_increment = match mode {
        InteractionMode::DraggingSelection { did_move, .. } => Some(*did_move),
        InteractionMode::ResizingSelection { did_resize, .. } => Some(*did_resize),
        _ => None,
    };

    if let Some(increment) = should_increment {
        if increment {
            doc.revision = doc.revision.increment();

            if did_resize {
                for (node_id, (ox, oy, ow, oh)) in originals {
                    if let Some(node) = doc.document.nodes.get(&node_id) {
                        let bounds = ResizeBounds::new(
                            node_id.clone(),
                            ox,
                            oy,
                            ow,
                            oh,
                            node.x.0,
                            node.y.0,
                            node.width.0,
                            node.height.0,
                        );
                        let _ = dispatch_node_resize(db_tx.as_ref(), bounds);
                    }
                }
            }
        }
        *mode = InteractionMode::Select;
        true
    } else {
        false
    }
}
