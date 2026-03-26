use crate::error::ShowError;
use diagram_models::document::DiagramDocument;

/// Serializes a `DiagramDocument` to a compact JSON string.
///
/// # Errors
/// - `ShowError::SerializationFailure` — `serde_json` internal error (unreachable in practice).
pub fn serialize_document(doc: &DiagramDocument) -> Result<String, ShowError> {
    serde_json::to_string(doc).map_err(|e| ShowError::SerializationFailure(e.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {
    use super::*;

    #[test]
    fn serialize_document_returns_compact_json_when_given_default_document() {
        let doc = DiagramDocument::default();
        let result = serialize_document(&doc);
        assert!(result.is_ok());
        let json = result.unwrap();
        let deserialized = serde_json::from_str::<DiagramDocument>(&json);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), doc);
    }

    #[test]
    fn serialize_document_output_contains_version_zero_when_document_version_is_zero() {
        let doc = DiagramDocument {
            version: 0,
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().contains("\"version\":0"));
    }

    #[test]
    fn serialize_document_output_contains_version_one_when_document_version_is_one() {
        let doc = DiagramDocument {
            version: 1,
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        assert!(result.is_ok());
        assert!(result.as_ref().unwrap().contains("\"version\":1"));
    }

    #[test]
    fn serialize_document_output_contains_node_id_when_document_has_nodes() {
        use diagram_models::document::types::OrderedFloat;
        use diagram_models::document::{LockState, Node, NodeId, NodeKind};
        let mut doc = DiagramDocument::default();
        let node_id = NodeId::new("node-abc".to_string());
        let node = Node {
            kind: NodeKind::Node,
            icon: String::new(),
            label: "test".to_string(),
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
        };
        doc.document.nodes.insert(node_id, node);
        let result = serialize_document(&doc);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("node-abc"));
        assert!(json.contains("\"nodes\":"));
    }

    #[test]
    fn serialize_document_output_contains_no_newlines_or_indentation() {
        let doc = DiagramDocument::default();
        let result = serialize_document(&doc);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(!json.contains('\n'));
        assert!(!json.contains("  "));
    }

    #[test]
    fn serialize_document_round_trips_to_identical_document_when_serialized_then_deserialized() {
        let doc = DiagramDocument::default();
        let json_result = serialize_document(&doc);
        assert!(json_result.is_ok());
        let json = json_result.unwrap();
        let deserialized = serde_json::from_str::<DiagramDocument>(&json);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), doc);
    }

    fn serialize_any<T: serde::Serialize>(val: &T) -> Result<String, ShowError> {
        serde_json::to_string(val).map_err(|e| ShowError::SerializationFailure(e.to_string()))
    }

    struct AlwaysFailsSerialize;

    impl serde::Serialize for AlwaysFailsSerialize {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("injected serialization error"))
        }
    }

    #[test]
    fn serialize_document_returns_serialization_failure_when_serde_json_errors() {
        let failing = AlwaysFailsSerialize;
        let result = serialize_any(&failing);
        assert!(matches!(result, Err(ShowError::SerializationFailure(_))));
        if let Err(ShowError::SerializationFailure(msg)) = result {
            assert!(msg.contains("injected serialization error"));
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::show::loader::load_document_from_reader;
    use proptest::prelude::*;
    use std::io::Cursor;

    proptest! {
        #[test]
        fn proptest_serialize_document_returns_ok_for_any_well_formed_document(
            version in any::<u32>()
        ) {
            let doc = DiagramDocument {
                version,
                ..DiagramDocument::default()
            };
            let result = serialize_document(&doc);
            prop_assert!(result.is_ok());
            if let Ok(ref json) = result {
                prop_assert!(!json.is_empty());
            }
        }

        #[test]
        fn proptest_serialize_then_deserialize_produces_identical_document(
            version in any::<u32>()
        ) {
            let doc = DiagramDocument {
                version,
                ..DiagramDocument::default()
            };
            let json_result = serialize_document(&doc);
            prop_assert!(json_result.is_ok());
            if let Ok(json) = json_result {
                let doc2 = load_document_from_reader(Cursor::new(json.into_bytes()));
                prop_assert_eq!(doc2, Ok(doc));
            }
        }
    }
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_serialize_document_never_panics_for_valid_doc() {
        let doc = DiagramDocument {
            version: kani::any(),
            ..DiagramDocument::default()
        };
        let result = serialize_document(&doc);
        if let Ok(s) = result {
            assert!(!s.is_empty());
        }
    }
}
