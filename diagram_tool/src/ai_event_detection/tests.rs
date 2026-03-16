//! Tests for AI Event Detection Module
//!
//! These tests verify the pure calculation functions for detecting dropped AI events.

use super::*;
use std::collections::HashSet;

fn create_event(op_id: &str) -> EventRecord {
    EventRecord {
        op_id: op_id.to_string(),
        revision: 1,
        timestamp: 1700000000,
        payload: "{}".to_string(),
    }
}

#[test]
fn test_find_dropped_op_ids_all_dropped() {
    let pending: HashSet<String> = ["ai-op-1".to_string(), "ai-op-2".to_string()]
        .into_iter()
        .collect();
    let fetched = vec![];

    let dropped = find_dropped_op_ids(&pending, &fetched);

    assert_eq!(dropped.len(), 2);
    assert!(dropped.contains(&"ai-op-1".to_string()));
    assert!(dropped.contains(&"ai-op-2".to_string()));
}

#[test]
fn test_find_dropped_op_ids_none_dropped() {
    let pending: HashSet<String> = ["ai-op-1".to_string()].into_iter().collect();
    let fetched = vec![create_event("ai-op-1")];

    let dropped = find_dropped_op_ids(&pending, &fetched);

    assert!(dropped.is_empty());
}

#[test]
fn test_find_dropped_op_ids_partial_dropped() {
    let pending: HashSet<String> = ["ai-op-1".to_string(), "ai-op-2".to_string()]
        .into_iter()
        .collect();
    let fetched = vec![create_event("ai-op-1")];

    let dropped = find_dropped_op_ids(&pending, &fetched);

    assert_eq!(dropped.len(), 1);
    assert!(dropped.contains(&"ai-op-2".to_string()));
}

#[test]
fn test_find_dropped_op_ids_empty_pending() {
    let pending: HashSet<String> = HashSet::new();
    let fetched = vec![create_event("ai-op-1")];

    let dropped = find_dropped_op_ids(&pending, &fetched);

    assert!(dropped.is_empty());
}

#[test]
fn test_generate_conflict_message_single() {
    let msg = generate_conflict_message(&["ai-op-1".to_string()]);

    assert!(msg.contains("AI operation rejected"));
    assert!(msg.contains("ai-op-1"));
}

#[test]
fn test_generate_conflict_message_multiple() {
    let msg = generate_conflict_message(&["ai-op-1".to_string(), "ai-op-2".to_string()]);

    assert!(msg.contains("AI operation rejected"));
    assert!(msg.contains('2'));
}

#[test]
fn test_generate_conflict_message_empty() {
    let msg = generate_conflict_message(&[]);

    assert!(msg.is_empty());
}

#[test]
fn test_detect_dropped_ai_events_no_pending() {
    let pending: HashSet<String> = HashSet::new();
    let fetched = vec![create_event("ai-op-1")];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert!(result.dropped_op_ids.is_empty());
    assert!(!result.has_conflict);
}

#[test]
fn test_detect_dropped_ai_events_all_confirmed() {
    let pending: HashSet<String> = ["ai-op-1".to_string()].into_iter().collect();
    let fetched = vec![create_event("ai-op-1")];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert!(result.dropped_op_ids.is_empty());
    assert!(!result.has_conflict);
}

#[test]
fn test_detect_dropped_ai_events_with_drop() {
    let pending: HashSet<String> = ["ai-op-1".to_string()].into_iter().collect();
    let fetched = vec![];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert_eq!(result.dropped_op_ids.len(), 1);
    assert!(result.has_conflict);
    assert!(result.conflict_state.is_some());
}

#[test]
fn test_detect_dropped_ai_events_existing_conflict() {
    let pending: HashSet<String> = ["ai-op-1".to_string()].into_iter().collect();
    let fetched = vec![];
    let conflict_state: Option<AiConflictState> = Some(AiConflictState::new(
        Some("existing conflict".to_string()),
        Vec::new(),
    ));

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    // Should detect but not report conflict (to avoid overwriting)
    assert_eq!(result.dropped_op_ids.len(), 1);
    assert!(!result.has_conflict);
    assert!(result.conflict_state.is_none());
}

#[test]
fn test_remove_dropped_ops() {
    let pending: HashSet<String> = ["ai-op-1".to_string(), "ai-op-2".to_string()]
        .into_iter()
        .collect();
    let dropped = vec!["ai-op-1".to_string()];

    let remaining = remove_dropped_ops(&pending, &dropped);

    assert!(!remaining.contains(&"ai-op-1".to_string()));
    assert!(remaining.contains(&"ai-op-2".to_string()));
}

#[test]
fn test_drop_detection_result_no_drops() {
    let result = DropDetectionResult::no_drops();

    assert!(result.dropped_op_ids.is_empty());
    assert!(!result.has_conflict);
    assert!(result.conflict_state.is_none());
}

#[test]
fn test_drop_detection_result_with_drops() {
    let result = DropDetectionResult::with_drops(vec!["ai-op-1".to_string()]);

    assert_eq!(result.dropped_op_ids.len(), 1);
    assert!(result.has_conflict);
    assert!(result.conflict_state.is_some());
}

// Happy Path Tests from martin-fowler-tests.md

/// `test_detects_dropped_ai_event_when_not_in_wal`
#[test]
fn test_detects_dropped_ai_event_when_not_in_wal() {
    let pending: HashSet<String> = ["ai-op-1".to_string()].into_iter().collect();
    let fetched = vec![]; // No events in WAL
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert_eq!(result.dropped_op_ids, vec!["ai-op-1"]);
    assert!(result.has_conflict);
    assert!(result
        .conflict_state
        .as_ref()
        .unwrap()
        .reason
        .as_ref()
        .unwrap()
        .contains("AI operation rejected"));
}

/// `test_no_conflict_when_ai_event_appears_in_wal`
#[test]
fn test_no_conflict_when_ai_event_appears_in_wal() {
    let pending: HashSet<String> = ["ai-op-1".to_string()].into_iter().collect();
    let fetched = vec![create_event("ai-op-1")];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert!(result.dropped_op_ids.is_empty());
    assert!(!result.has_conflict);
}

/// `test_handles_empty_pending_set_gracefully`
#[test]
fn test_handles_empty_pending_set_gracefully() {
    let pending: HashSet<String> = HashSet::new();
    let fetched = vec![];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert!(result.dropped_op_ids.is_empty());
    assert!(!result.has_conflict);
}

// Edge Case Tests from martin-fowler-tests.md

/// `test_multiple_dropped_ai_events_detected_together`
#[test]
fn test_multiple_dropped_ai_events_detected_together() {
    let pending: HashSet<String> = [
        "ai-op-1".to_string(),
        "ai-op-2".to_string(),
        "ai-op-3".to_string(),
    ]
    .into_iter()
    .collect();
    let fetched = vec![];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert_eq!(result.dropped_op_ids.len(), 3);
    assert!(result.has_conflict);
}

/// `test_mixed_dropped_and_confirmed_events`
#[test]
fn test_mixed_dropped_and_confirmed_events() {
    let pending: HashSet<String> = ["ai-op-1".to_string(), "ai-op-2".to_string()]
        .into_iter()
        .collect();
    let fetched = vec![create_event("ai-op-1")];
    let conflict_state: Option<AiConflictState> = None;

    let result = detect_dropped_ai_events(&pending, &fetched, &conflict_state);

    assert_eq!(result.dropped_op_ids, vec!["ai-op-2"]);
}
