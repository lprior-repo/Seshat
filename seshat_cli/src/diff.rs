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

#[must_use]
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
            if human_nodes.is_none_or(|hn| !hn.contains_key(node_id)) {
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
            if human_edges.is_none_or(|he| !he.contains_key(edge_id)) {
                rich_diff.add_entity_diff(edge_id.clone(), None, Some(ae_val.clone()));
            }
        }
    }

    rich_diff
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
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

    #[test]
    fn build_rich_diff_no_nodes_no_edges() {
        let diff = build_rich_diff(1, 1, None, None, None, None);
        assert_eq!(diff.status, "rejected");
        assert_eq!(diff.conflict_context.expected_revision, 1);
        assert_eq!(diff.conflict_context.actual_revision, 1);
        assert!(diff.conflict_context.conflicting_entities.is_empty());
        assert!(diff.conflict_context.diff.is_empty());
    }

    #[test]
    fn build_rich_diff_only_human_nodes_no_ai_nodes() {
        let human_nodes =
            serde_json::Map::from_iter(vec![("n1".to_string(), serde_json::json!({"x": 10}))]);
        let diff = build_rich_diff(1, 1, Some(&human_nodes), None, None, None);
        assert_eq!(diff.conflict_context.conflicting_entities, vec!["n1"]);
        let ed = diff.conflict_context.diff.get("n1").unwrap();
        assert_eq!(ed.human_state, Some(serde_json::json!({"x": 10})));
        assert_eq!(ed.ai_proposed_state, None);
    }

    #[test]
    fn build_rich_diff_only_ai_nodes_no_human_nodes() {
        let ai_nodes =
            serde_json::Map::from_iter(vec![("n2".to_string(), serde_json::json!({"x": 20}))]);
        let diff = build_rich_diff(1, 1, None, None, Some(&ai_nodes), None);
        assert_eq!(diff.conflict_context.conflicting_entities, vec!["n2"]);
        let ed = diff.conflict_context.diff.get("n2").unwrap();
        assert_eq!(ed.human_state, None);
        assert_eq!(ed.ai_proposed_state, Some(serde_json::json!({"x": 20})));
    }

    #[test]
    fn build_rich_diff_matching_nodes_no_conflicts() {
        let val = serde_json::json!({"x": 10, "y": 20});
        let human_nodes = serde_json::Map::from_iter(vec![("n1".to_string(), val.clone())]);
        let ai_nodes = serde_json::Map::from_iter(vec![("n1".to_string(), val)]);
        let diff = build_rich_diff(1, 1, Some(&human_nodes), None, Some(&ai_nodes), None);
        assert!(diff.conflict_context.conflicting_entities.is_empty());
        assert!(diff.conflict_context.diff.is_empty());
    }

    #[test]
    fn build_rich_diff_mixed_matching_and_differing_nodes() {
        let shared_val = serde_json::json!({"x": 10});
        let human_nodes = serde_json::Map::from_iter(vec![
            ("n1".to_string(), shared_val.clone()),
            ("n2".to_string(), serde_json::json!({"x": 20})),
        ]);
        let ai_nodes = serde_json::Map::from_iter(vec![
            ("n1".to_string(), shared_val),
            ("n3".to_string(), serde_json::json!({"x": 30})),
        ]);
        let diff = build_rich_diff(1, 1, Some(&human_nodes), None, Some(&ai_nodes), None);

        // n1 matches → no conflict; n2 only in human → conflict; n3 only in ai → conflict
        let mut entities = diff.conflict_context.conflicting_entities.clone();
        entities.sort();
        assert_eq!(entities, vec!["n2", "n3"]);

        // n2: human present, ai absent
        let ed_n2 = diff.conflict_context.diff.get("n2").unwrap();
        assert_eq!(ed_n2.human_state, Some(serde_json::json!({"x": 20})));
        assert_eq!(ed_n2.ai_proposed_state, None);

        // n3: human absent, ai present
        let ed_n3 = diff.conflict_context.diff.get("n3").unwrap();
        assert_eq!(ed_n3.human_state, None);
        assert_eq!(ed_n3.ai_proposed_state, Some(serde_json::json!({"x": 30})));
    }

    #[test]
    fn build_rich_diff_differing_edges() {
        let human_edges = serde_json::Map::from_iter(vec![(
            "e1".to_string(),
            serde_json::json!({"source": "a", "target": "b"}),
        )]);
        let ai_edges = serde_json::Map::from_iter(vec![(
            "e1".to_string(),
            serde_json::json!({"source": "a", "target": "c"}),
        )]);
        let diff = build_rich_diff(1, 1, None, Some(&human_edges), None, Some(&ai_edges));
        assert_eq!(diff.conflict_context.conflicting_entities, vec!["e1"]);
        let ed = diff.conflict_context.diff.get("e1").unwrap();
        assert_eq!(
            ed.human_state,
            Some(serde_json::json!({"source": "a", "target": "b"}))
        );
        assert_eq!(
            ed.ai_proposed_state,
            Some(serde_json::json!({"source": "a", "target": "c"}))
        );
    }

    #[test]
    fn add_entity_diff_same_id_appears_once_in_conflicting_entities() {
        let mut diff = RichDiff::new(1, 2);
        diff.add_entity_diff(
            "n1".to_string(),
            Some(serde_json::json!({"x": 10})),
            Some(serde_json::json!({"x": 20})),
        );
        diff.add_entity_diff(
            "n1".to_string(),
            Some(serde_json::json!({"x": 10})),
            Some(serde_json::json!({"x": 30})),
        );
        // Should only appear once in conflicting_entities
        assert_eq!(
            diff.conflict_context.conflicting_entities,
            vec!["n1".to_string()]
        );
        // But the diff should reflect the second call (overwritten)
        let ed = diff.conflict_context.diff.get("n1").unwrap();
        assert_eq!(ed.ai_proposed_state, Some(serde_json::json!({"x": 30})));
    }

    #[test]
    fn rich_diff_serialization_roundtrip() {
        let mut diff = RichDiff::new(3, 5);
        diff.add_entity_diff(
            "n1".to_string(),
            Some(serde_json::json!({"x": 10})),
            Some(serde_json::json!({"x": 20})),
        );
        let json_str = serde_json::to_string(&diff).expect("serialize");
        let deserialized: RichDiff = serde_json::from_str(&json_str).expect("deserialize");
        assert_eq!(diff, deserialized);
    }
}
