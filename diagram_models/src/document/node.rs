//! Node-related domain types for diagram documents.
//!
//! Contains Node, `NodeKind`, `LockState`, `NodeStyle`, `FontWeight`, and related types.

use im::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::OrderedFloat;

/// Visual style for nodes
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeStyle {
    #[default]
    Box,
    Cloud,
    Cylinder,
    Dashed,
}

/// Font weight for node labels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    Normal,
    Bold,
}

/// Kind of node in the diagram
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Node,
    Subgraph,
    Text,
}

/// Lock state for nodes - makes illegal states unrepresentable
///
/// This enum models the locked/unlocked state of a node. Using an enum rather than
/// a boolean ensures that invalid states cannot be constructed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum LockState {
    #[default]
    Unlocked,
    Locked,
}

impl LockState {
    /// Returns true if the node is in a locked state
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        matches!(self, Self::Locked)
    }

    /// Returns true if the node can be moved/edited
    /// Subgraphs are always movable regardless of lock state
    #[must_use]
    pub const fn is_movable(&self, node_kind: &NodeKind) -> bool {
        match node_kind {
            NodeKind::Subgraph => true,
            _ => !self.is_locked(),
        }
    }
}

/// Custom serializer to serialize `LockState` as "locked": bool for backwards compatibility
mod lock_state_serde {
    use super::LockState;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes `LockState` to JSON as "locked": bool
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(lock_state: &LockState, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let locked = lock_state.is_locked();
        locked.serialize(serializer)
    }

    /// Deserializes from JSON - accepts "locked": bool (legacy format)
    #[allow(clippy::unnecessary_wraps)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<LockState, D::Error>
    where
        D: Deserializer<'de>,
    {
        let result: Result<bool, _> = Deserialize::deserialize(deserializer);
        match result {
            Ok(true) => Ok(LockState::Locked),
            Ok(false) | Err(_) => Ok(LockState::Unlocked),
        }
    }
}

/// A node in the diagram
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub kind: NodeKind,
    #[serde(default)]
    pub icon: String,
    pub label: String,
    pub x: OrderedFloat,
    pub y: OrderedFloat,
    pub width: OrderedFloat,
    pub height: OrderedFloat,
    #[serde(default, rename = "fontSize", alias = "font_size")]
    pub font_size: Option<OrderedFloat>,
    #[serde(default)]
    pub font_weight: Option<FontWeight>,
    #[serde(
        default,
        rename = "locked",
        serialize_with = "lock_state_serde::serialize",
        deserialize_with = "lock_state_serde::deserialize"
    )]
    pub lock_state: LockState,
    #[serde(default)]
    pub parent: Option<super::types::NodeId>,
    #[serde(default)]
    pub dag_rank: Option<i64>,
    #[serde(default)]
    pub tags: im::Vector<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default)]
    pub z_index: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<NodeStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
}

impl Node {
    /// Gets the world coordinates of the node.
    ///
    /// # Errors
    /// Returns `Error::NodeNotFound` if a parent node is missing from the provided map.
    pub fn get_world_coords(
        &self,
        nodes: &std::collections::HashMap<super::types::NodeId, Self>,
    ) -> Result<(f64, f64), String> {
        let mut world_x = self.x.0;
        let mut world_y = self.y.0;
        let mut current_parent_id = self.parent.as_ref();

        let mut depth = 0;
        while let Some(parent_id) = current_parent_id {
            if depth > 1000 {
                return Err("Cycle".into());
            }
            depth += 1;
            let parent_node = nodes
                .get(parent_id)
                .ok_or_else(|| format!("Parent node not found: {parent_id}"))?;
            world_x += parent_node.x.0;
            world_y += parent_node.y.0;
            current_parent_id = parent_node.parent.as_ref();
        }

        Ok((world_x, world_y))
    }

    /// Gets the world coordinates of the node using an `im::HashMap`.
    ///
    /// # Errors
    /// Returns `Error` if a parent node is missing from the provided map.
    pub fn get_world_coords_im(
        &self,
        nodes: &im::HashMap<super::types::NodeId, Self>,
    ) -> Result<(f64, f64), String> {
        let mut world_x = self.x.0;
        let mut world_y = self.y.0;
        let mut current_parent_id = self.parent.as_ref();

        let mut depth = 0;
        while let Some(parent_id) = current_parent_id {
            if depth > 1000 {
                return Err("Cycle".into());
            }
            depth += 1;
            let parent_node = nodes
                .get(parent_id)
                .ok_or_else(|| format!("Parent node not found: {parent_id}"))?;
            world_x += parent_node.x.0;
            world_y += parent_node.y.0;
            current_parent_id = parent_node.parent.as_ref();
        }

        Ok((world_x, world_y))
    }

    /// Returns the rotation in radians from metadata, or 0.0 if not set.
    #[inline]
    #[must_use]
    pub fn rotation(&self) -> f64 {
        self.metadata
            .get("rotation")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0)
    }

    /// Returns true if the node is visible (not hidden).
    #[inline]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.metadata
            .get("visibility")
            .and_then(serde_json::Value::as_str)
            != Some("hidden")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::super::types::OrderedFloat;
    use super::{LockState, Node, NodeKind};

    fn create_test_node(id: &str) -> Node {
        Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: id.to_string(),
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
            width: OrderedFloat(100.0),
            height: OrderedFloat(100.0),
            font_size: None,
            font_weight: None,
            lock_state: LockState::Unlocked,
            parent: None,
            dag_rank: None,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
            z_index: 0,
            style: None,
            collapsed: None,
        }
    }

    #[test]
    fn lock_state_is_locked_returns_true_for_locked() {
        let locked = LockState::Locked;
        assert!(locked.is_locked());
    }

    #[test]
    fn lock_state_is_locked_returns_false_for_unlocked() {
        let unlocked = LockState::Unlocked;
        assert!(!unlocked.is_locked());
    }

    #[test]
    fn lock_state_is_movable_returns_false_for_locked_node() {
        let locked = LockState::Locked;
        assert!(!locked.is_movable(&NodeKind::Node));
    }

    #[test]
    fn lock_state_is_movable_returns_true_for_unlocked_node() {
        let unlocked = LockState::Unlocked;
        assert!(unlocked.is_movable(&NodeKind::Node));
    }

    #[test]
    fn lock_state_is_movable_returns_true_for_subgraph_regardless_of_lock() {
        let locked = LockState::Locked;
        let unlocked = LockState::Unlocked;
        assert!(locked.is_movable(&NodeKind::Subgraph));
        assert!(unlocked.is_movable(&NodeKind::Subgraph));
    }

    #[test]
    fn node_serialization_roundtrip() {
        let node = create_test_node("test-node");
        let json = serde_json::to_string(&node).unwrap();
        let parsed: Node = serde_json::from_str(&json).unwrap();
        assert_eq!(node, parsed);
    }
}
