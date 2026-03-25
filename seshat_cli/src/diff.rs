use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityDiff {
    pub human_state: Option<serde_json::Value>,
    pub ai_proposed_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictContext {
    pub expected_revision: u64,
    pub actual_revision: u64,
    pub conflicting_entities: Vec<String>,
    pub diff: HashMap<String, EntityDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RichDiff {
    pub status: String,
    pub reason: String,
    pub conflict_context: ConflictContext,
}

impl RichDiff {
    #[must_use]
    pub fn new(expected_revision: u64, actual_revision: u64) -> Self {
        Self {
            status: "rejected".to_string(),
            reason: "Human Priority Block".to_string(),
            conflict_context: ConflictContext {
                expected_revision,
                actual_revision,
                conflicting_entities: Vec::new(),
                diff: HashMap::new(),
            },
        }
    }

    pub fn add_entity_diff(
        &mut self,
        entity_id: String,
        human_state: Option<serde_json::Value>,
        ai_proposed_state: Option<serde_json::Value>,
    ) {
        if !self
            .conflict_context
            .conflicting_entities
            .contains(&entity_id)
        {
            self.conflict_context
                .conflicting_entities
                .push(entity_id.clone());
        }
        self.conflict_context.diff.insert(
            entity_id,
            EntityDiff {
                human_state,
                ai_proposed_state,
            },
        );
    }
}

pub fn build_rich_diff(
    expected_revision: u64,
    actual_revision: u64,
    human_nodes: Option<&serde_json::Map<String, serde_json::Value>>,
    human_edges: Option<&serde_json::Map<String, serde_json::Value>>,
    ai_nodes: Option<&serde_json::Map<String, serde_json::Value>>,
    ai_edges: Option<&serde_json::Map<String, serde_json::Value>>,
) -> RichDiff {
    let mut rich_diff = RichDiff::new(expected_revision, actual_revision);

    if let Some(hn_obj) = human_nodes {
        for (node_id, hn_val) in hn_obj {
            let ai_val = ai_nodes.and_then(|an| an.get(node_id));
            if Some(hn_val) != ai_val {
                rich_diff.add_entity_diff(node_id.clone(), Some(hn_val.clone()), ai_val.cloned());
            }
        }
    }

    if let Some(an_obj) = ai_nodes {
        for (node_id, an_val) in an_obj {
            if human_nodes.map_or(true, |hn| !hn.contains_key(node_id)) {
                rich_diff.add_entity_diff(node_id.clone(), None, Some(an_val.clone()));
            }
        }
    }

    if let Some(he_obj) = human_edges {
        for (edge_id, he_val) in he_obj {
            let ai_val = ai_edges.and_then(|ae| ae.get(edge_id));
            if Some(he_val) != ai_val {
                rich_diff.add_entity_diff(edge_id.clone(), Some(he_val.clone()), ai_val.cloned());
            }
        }
    }

    if let Some(ae_obj) = ai_edges {
        for (edge_id, ae_val) in ae_obj {
            if human_edges.map_or(true, |he| !he.contains_key(edge_id)) {
                rich_diff.add_entity_diff(edge_id.clone(), None, Some(ae_val.clone()));
            }
        }
    }

    rich_diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rich_diff_initialization() {
        let diff = RichDiff::new(1, 2);
        assert_eq!(diff.status, "rejected");
        assert_eq!(diff.conflict_context.expected_revision, 1);
        assert_eq!(diff.conflict_context.actual_revision, 2);
        assert!(diff.conflict_context.conflicting_entities.is_empty());
    }

    #[test]
    fn rich_diff_add_entity() {
        let mut diff = RichDiff::new(1, 2);
        let human = serde_json::json!({"x": 10});
        let ai = serde_json::json!({"x": 20});
        diff.add_entity_diff("node-1".to_string(), Some(human.clone()), Some(ai.clone()));

        assert_eq!(
            diff.conflict_context.conflicting_entities,
            vec!["node-1".to_string()]
        );
        let entity_diff = diff.conflict_context.diff.get("node-1").unwrap();
        assert_eq!(entity_diff.human_state, Some(human));
        assert_eq!(entity_diff.ai_proposed_state, Some(ai));
    }
}
