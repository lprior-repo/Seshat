use crate::document::types::EdgeId;
use crate::document::{DiagramDocument, Edge};
use crate::multi_select::NonEmptyVec;
use itertools::Itertools;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),
    #[error("Postcondition violated: {0}")]
    PostconditionViolated(String),
    #[error("Invariant violated: {0}")]
    InvariantViolated(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    OneWay,
    ZeroWay,
    TwoWay,
}

#[must_use]
pub const fn determine_next_direction(current: Direction) -> Direction {
    match current {
        Direction::OneWay => Direction::ZeroWay,
        Direction::ZeroWay => Direction::TwoWay,
        Direction::TwoWay => Direction::OneWay,
    }
}

/// Determines the current direction of an edge.
/// # Errors
/// Returns `Error::InvariantViolated` if an undirected edge has bidirectional metadata.
pub fn determine_current_direction(edge: &Edge) -> Result<Direction, Error> {
    let bidirectional = edge.metadata.get("bidirectional");
    let is_bidirectional = match bidirectional {
        Some(Value::Bool(b)) => *b,
        _ => false, // Malformed metadata treated as false
    };

    match (edge.directed, is_bidirectional) {
        (true, true) => Ok(Direction::TwoWay),
        (true, false) => Ok(Direction::OneWay),
        (false, false) => Ok(Direction::ZeroWay),
        (false, true) => Err(Error::InvariantViolated(
            "undirected edges cannot be bidirectional".into(),
        )),
    }
}

/// Verifies edge invariants are satisfied.
/// # Errors
/// Returns `Error::InvariantViolated` if an undirected edge has bidirectional metadata.
pub fn verify_invariants(edge: &Edge) -> Result<(), Error> {
    let bidirectional = edge.metadata.get("bidirectional");
    let is_bidirectional = match bidirectional {
        Some(Value::Bool(b)) => *b,
        _ => false,
    };

    if !edge.directed && is_bidirectional {
        return Err(Error::InvariantViolated(
            "undirected edges cannot be bidirectional".into(),
        ));
    }

    Ok(())
}

fn apply_direction(edge: &Edge, new_direction: Direction) -> Edge {
    let mut new_edge = edge.clone();
    match new_direction {
        Direction::OneWay => {
            new_edge.directed = true;
            new_edge.metadata.remove("bidirectional");
        }
        Direction::ZeroWay => {
            new_edge.directed = false;
            new_edge.metadata.remove("bidirectional");
        }
        Direction::TwoWay => {
            new_edge.directed = true;
            new_edge
                .metadata
                .insert("bidirectional".to_string(), Value::Bool(true));
        }
    }
    new_edge
}

/// Toggles the direction of edges in the selection.
/// # Errors
/// Returns `Error::InvariantViolated` if selection is empty.
/// Returns `Error::EdgeNotFound` if any edge in selection does not exist.
/// Returns `Error::InvariantViolated` if an undirected edge has bidirectional metadata.
pub fn toggle_edge_directions(
    doc: &mut DiagramDocument,
    selection: &NonEmptyVec<String>,
) -> Result<(), Error> {
    let first_id = selection
        .as_slice()
        .first()
        .ok_or_else(|| Error::InvariantViolated("Selection cannot be empty".into()))?;

    let unique_ids: Vec<EdgeId> = selection
        .as_slice()
        .iter()
        .map(|id| EdgeId::new(id.clone()))
        .unique()
        .collect();

    // P1: All edges must exist
    unique_ids.iter().try_for_each(|id| {
        if doc.document.edges.contains_key(id) {
            Ok(())
        } else {
            Err(Error::EdgeNotFound(id.as_str().to_string()))
        }
    })?;

    // Invariant check: Determine direction enforces I2 and I3 logic implicitly
    unique_ids.iter().try_for_each(|id| {
        let edge = doc
            .document
            .edges
            .get(id)
            .ok_or_else(|| Error::EdgeNotFound(id.to_string()))?;
        determine_current_direction(edge).map(|_| ())
    })?;

    let primary_edge_id = EdgeId::new(first_id.clone());
    let primary_edge = doc
        .document
        .edges
        .get(&primary_edge_id)
        .ok_or_else(|| Error::EdgeNotFound(primary_edge_id.to_string()))?;
    let current_dir = determine_current_direction(primary_edge)?;
    let target_dir = determine_next_direction(current_dir);

    let new_edges = unique_ids
        .iter()
        .try_fold(doc.document.edges.clone(), |acc_edges, id| {
            let edge = acc_edges
                .get(id)
                .ok_or_else(|| Error::EdgeNotFound(id.to_string()))?;
            let updated_edge = apply_direction(edge, target_dir);
            Ok::<_, Error>(acc_edges.update(id.clone(), updated_edge))
        })?;

    doc.document.edges = new_edges;

    Ok(())
}

/// Verifies that an edge is in the undirected (0-way) state.
/// # Errors
/// Returns `Error::PostconditionViolated` if edge is directed or has bidirectional metadata.
pub fn verify_0_way_postcondition(edge: &Edge) -> Result<(), Error> {
    if edge.directed {
        return Err(Error::PostconditionViolated("Expected undirected".into()));
    }
    if edge.metadata.contains_key("bidirectional") {
        return Err(Error::PostconditionViolated("Expected undirected".into()));
    }
    Ok(())
}

/// Verifies that an edge is in the directed (1-way) state.
/// # Errors
/// Returns `Error::PostconditionViolated` if edge is undirected or has bidirectional metadata.
pub fn verify_1_way_postcondition(edge: &Edge) -> Result<(), Error> {
    if !edge.directed {
        return Err(Error::PostconditionViolated(
            "Expected directed, no bidirectional flag".into(),
        ));
    }
    if edge.metadata.contains_key("bidirectional") {
        return Err(Error::PostconditionViolated(
            "Expected directed, no bidirectional flag".into(),
        ));
    }
    Ok(())
}

/// Verifies that an edge is in the bidirectional (2-way) state.
/// # Errors
/// Returns `Error::PostconditionViolated` if edge is not directed and bidirectional.
pub fn verify_2_way_postcondition(edge: &Edge) -> Result<(), Error> {
    if !edge.directed {
        return Err(Error::PostconditionViolated(
            "Expected bidirectional".into(),
        ));
    }
    let bidirectional = edge.metadata.get("bidirectional");
    let is_bidirectional = match bidirectional {
        Some(Value::Bool(b)) => *b,
        _ => false,
    };
    if !is_bidirectional {
        return Err(Error::PostconditionViolated(
            "Expected bidirectional".into(),
        ));
    }
    Ok(())
}

/// Verifies that malformed metadata was removed from an edge.
/// # Errors
/// Returns `Error::PostconditionViolated` if malformed metadata is still present.
pub fn verify_sanitized_metadata(edge: &Edge) -> Result<(), Error> {
    if edge.metadata.contains_key("bidirectional") {
        return Err(Error::PostconditionViolated(
            "Malformed metadata not removed".into(),
        ));
    }
    Ok(())
}

/// Verifies that all edges in selection are uniformly aligned.
/// # Errors
/// Returns `Error::PostconditionViolated` if edges are not uniformly aligned.
pub fn verify_selection_postcondition(edges: &[Edge]) -> Result<(), Error> {
    if edges.is_empty() {
        return Ok(());
    }
    let first_dir = determine_current_direction(&edges[0])?;
    edges.iter().try_for_each(|edge| {
        let dir = determine_current_direction(edge)?;
        if dir == first_dir {
            Ok(())
        } else {
            Err(Error::PostconditionViolated(
                "Edges not uniformly aligned".into(),
            ))
        }
    })?;
    Ok(())
}

/// Verifies that unselected edges remain unaltered between two document versions.
/// # Errors
/// Returns `Error::PostconditionViolated` if an unselected edge was removed or modified.
pub fn verify_unselected_edges_unaltered(
    doc1: &DiagramDocument,
    doc2: &DiagramDocument,
    selection: &NonEmptyVec<String>,
) -> Result<(), Error> {
    use std::collections::HashSet;
    let selected_ids: HashSet<&str> = selection.as_slice().iter().map(String::as_str).collect();

    for (id, edge1) in &doc1.document.edges {
        if !selected_ids.contains(id.as_str()) {
            let edge2 = doc2.document.edges.get(id).ok_or_else(|| {
                Error::PostconditionViolated(format!("Unselected edge {id} removed"))
            })?;
            if edge1 != edge2 {
                return Err(Error::PostconditionViolated(format!(
                    "Unselected edge {id} was modified"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::document::types::NodeId;
    use crate::test_utils::{DocBuilder, EdgeBuilder};
    use serde_json::json;

    fn build_1_way_edge() -> Edge {
        EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .build()
    }

    fn build_0_way_edge() -> Edge {
        EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(false)
            .build()
    }

    fn build_2_way_edge() -> Edge {
        EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .with_metadata("bidirectional", json!(true))
            .build()
    }

    #[test]
    fn test_cycles_1_way_to_0_way() {
        let mut doc = DocBuilder::new().add_edge("e1", build_1_way_edge()).build();
        let selection = NonEmptyVec::try_from(vec!["e1".to_string()]).unwrap();

        toggle_edge_directions(&mut doc, &selection).unwrap();

        let edge = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        verify_0_way_postcondition(edge).unwrap();
    }

    #[test]
    fn test_cycles_0_way_to_2_way() {
        let mut doc = DocBuilder::new().add_edge("e1", build_0_way_edge()).build();
        let selection = NonEmptyVec::try_from(vec!["e1".to_string()]).unwrap();

        toggle_edge_directions(&mut doc, &selection).unwrap();

        let edge = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        verify_2_way_postcondition(edge).unwrap();
    }

    #[test]
    fn test_cycles_2_way_to_1_way() {
        let mut doc = DocBuilder::new().add_edge("e1", build_2_way_edge()).build();
        let selection = NonEmptyVec::try_from(vec!["e1".to_string()]).unwrap();

        toggle_edge_directions(&mut doc, &selection).unwrap();

        let edge = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        verify_1_way_postcondition(edge).unwrap();
    }

    #[test]
    fn test_deduplicates_combinatorial_variance_of_edge_ids() {
        let mut doc = DocBuilder::new().add_edge("e1", build_1_way_edge()).build();
        let selection = NonEmptyVec::try_from(vec!["e1".to_string(), "e1".to_string()]).unwrap();

        toggle_edge_directions(&mut doc, &selection).unwrap();

        let edge = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        verify_0_way_postcondition(edge).unwrap();
    }

    #[test]
    fn test_returns_error_when_edge_not_found() {
        let mut doc = DocBuilder::new().build();
        let selection = NonEmptyVec::try_from(vec!["missing".to_string()]).unwrap();

        let err = toggle_edge_directions(&mut doc, &selection).unwrap_err();
        assert!(matches!(err, Error::EdgeNotFound(_)));
    }

    #[test]
    fn test_atomic_failure_when_missing_edge_is_in_middle_of_selection() {
        let mut doc = DocBuilder::new()
            .add_edge("e1", build_1_way_edge())
            .add_edge("e2", build_1_way_edge())
            .build();
        let original_doc = doc.clone();
        let selection = NonEmptyVec::try_from(vec![
            "e1".to_string(),
            "missing".to_string(),
            "e2".to_string(),
        ])
        .unwrap();

        let err = toggle_edge_directions(&mut doc, &selection).unwrap_err();
        assert!(matches!(err, Error::EdgeNotFound(_)));
        assert_eq!(doc, original_doc, "Document should remain unmodified");
    }

    #[test]
    fn test_handles_malformed_metadata_null() {
        let edge = EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .with_metadata("bidirectional", Value::Null)
            .build();
        let mut doc = DocBuilder::new().add_edge("e1", edge).build();
        let selection = NonEmptyVec::try_from(vec!["e1".to_string()]).unwrap();

        toggle_edge_directions(&mut doc, &selection).unwrap();

        let toggled = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        verify_0_way_postcondition(toggled).unwrap();
        verify_sanitized_metadata(toggled).unwrap();
    }

    #[test]
    fn test_handles_malformed_metadata_string() {
        let edge = EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .with_metadata("bidirectional", json!("true_string"))
            .build();
        let mut doc = DocBuilder::new().add_edge("e1", edge).build();
        let selection = NonEmptyVec::try_from(vec!["e1".to_string()]).unwrap();

        toggle_edge_directions(&mut doc, &selection).unwrap();
        let toggled = doc.document.edges.get(&EdgeId::new("e1".into())).unwrap();
        verify_0_way_postcondition(toggled).unwrap();
        verify_sanitized_metadata(toggled).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Q1_violation_returns_postcondition_violated() {
        let edge = EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .build();
        let err = verify_0_way_postcondition(&edge).unwrap_err();
        assert!(matches!(err, Error::PostconditionViolated(_)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Q2_violation_returns_postcondition_violated() {
        let edge = EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .build();
        let err = verify_2_way_postcondition(&edge).unwrap_err();
        assert!(matches!(err, Error::PostconditionViolated(_)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Q3_violation_returns_postcondition_violated() {
        let edge = EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(true)
            .with_metadata("bidirectional", json!(true))
            .build();
        let err = verify_1_way_postcondition(&edge).unwrap_err();
        assert!(matches!(err, Error::PostconditionViolated(_)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I2_violation_rejects_undirected_but_bidirectional_edge() {
        let edge = EdgeBuilder::new(NodeId::new("A".into()), NodeId::new("B".into()))
            .directed(false)
            .with_metadata("bidirectional", json!(true))
            .build();
        let err = verify_invariants(&edge).unwrap_err();
        assert!(matches!(err, Error::InvariantViolated(_)));

        let err2 = determine_current_direction(&edge).unwrap_err();
        assert!(matches!(err2, Error::InvariantViolated(_)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_Q5_violation_returns_postcondition_violated() {
        let edge1 = build_1_way_edge();
        let edge2 = build_2_way_edge();
        let err = verify_selection_postcondition(&[edge1, edge2]).unwrap_err();
        assert!(matches!(err, Error::PostconditionViolated(_)));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I4_violation_state_conservation() {
        let doc1 = DocBuilder::new()
            .add_edge("e1", build_1_way_edge())
            .add_edge("e2", build_1_way_edge())
            .build();
        let mut doc2 = doc1.clone();

        // Mutate e2 which is not in selection
        let mut mutated_e2 = doc2
            .document
            .edges
            .get(&EdgeId::new("e2".into()))
            .unwrap()
            .clone();
        mutated_e2.directed = false;
        doc2.document
            .edges
            .insert(EdgeId::new("e2".into()), mutated_e2);

        let selection = NonEmptyVec::try_from(vec!["e1".to_string()]).unwrap();
        let err = verify_unselected_edges_unaltered(&doc1, &doc2, &selection).unwrap_err();
        assert!(matches!(err, Error::PostconditionViolated(_)));
    }

    #[test]
    fn test_atomic_failure_permutation_missing_at_end() {
        let mut doc = DocBuilder::new()
            .add_edge("e1", build_1_way_edge())
            .add_edge("e2", build_1_way_edge())
            .build();
        let original_doc = doc.clone();
        let selection = NonEmptyVec::try_from(vec![
            "e1".to_string(),
            "e2".to_string(),
            "missing".to_string(),
        ])
        .unwrap();

        let err = toggle_edge_directions(&mut doc, &selection).unwrap_err();
        assert!(matches!(err, Error::EdgeNotFound(_)));
        assert_eq!(doc, original_doc, "Document should remain unmodified");
    }
}
