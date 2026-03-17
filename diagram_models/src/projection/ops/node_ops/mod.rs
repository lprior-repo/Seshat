//! Node operations for diagram projection.

mod add_move;
mod delete;
mod update;

use im::HashMap;

use crate::document::{Edge, EdgeId, Node, NodeId};
use crate::projection::types::DiagramProjection;

type NodeMap = HashMap<NodeId, Node>;
type EdgeMap = HashMap<EdgeId, Edge>;

fn build_projection(state: DiagramProjection, nodes: NodeMap, edges: EdgeMap) -> DiagramProjection {
    DiagramProjection {
        version: state.version,
        revision: state.revision,
        nodes,
        edges,
        author_priority: state.author_priority,
        cycle_policy: state.cycle_policy,
    }
}

pub use add_move::{apply_node_add, apply_node_move, create_default_node};
pub use delete::{apply_node_delete, apply_node_op, apply_node_restore};
pub use update::{apply_update_label, apply_update_node_style};
