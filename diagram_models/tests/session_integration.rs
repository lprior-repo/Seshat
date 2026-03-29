#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::expect_used, clippy::panic, clippy::unwrap_used, clippy::similar_names, clippy::redundant_clone)]
use diagram_models::document::{
    DiagramDocument, DocumentSession, Edge, EdgeStyle, Node, NodeId, NodeKind, OrderedFloat,
    Revision,
};
use im::HashMap;
use proptest::prelude::*;
use std::path::PathBuf;

fn make_doc_with_revision(rev: u64) -> DiagramDocument {
    DiagramDocument {
        revision: Revision::new(rev),
        ..Default::default()
    }
}

fn make_node(id: &str, x: f64) -> (NodeId, Node) {
    let node = Node {
        kind: NodeKind::Node,
        icon: String::new(),
        label: id.to_string(),
        x: OrderedFloat::new_unchecked(x),
        y: OrderedFloat::new_unchecked(20.0),
        width: OrderedFloat::new_unchecked(100.0),
        height: OrderedFloat::new_unchecked(50.0),
        font_size: None,
        font_weight: None,
        lock_state: diagram_models::document::LockState::Unlocked,
        parent: None,
        dag_rank: None,
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        z_index: 0,
        style: Some(diagram_models::document::NodeStyle::default()),
        collapsed: None,
    };
    (NodeId::new(id.to_string()), node)
}

fn make_edge(source: &str, target: &str) -> Edge {
    Edge {
        source: NodeId::new(source.to_string()),
        target: NodeId::new(target.to_string()),
        label: String::new(),
        style: EdgeStyle::default(),
        arrow_type: diagram_models::document::ArrowType::default(),
        label_offset_t: OrderedFloat::new_unchecked(0.5),
        color: None,
        thickness: OrderedFloat::new_unchecked(1.5),
        directed: true,
        bend_points: im::Vector::new(),
        tags: im::Vector::new(),
        metadata: HashMap::new(),
        font_size: None,
        source_port: None,
        target_port: None,
    }
}

fn make_doc_with_nodes_and_edges(
    rev: u64,
    node_count: usize,
    edge_count: usize,
) -> DiagramDocument {
    let mut doc = make_doc_with_revision(rev);
    for i in 0..node_count {
        let (id, node) = make_node(&format!("n{i}"), 10.0 * i as f64);
        doc.document.nodes = doc.document.nodes.update(id, node);
    }
    for i in 0..edge_count {
        let source_str = format!("n{}", i);
        let target_str = format!("n{}", (i + 1) % node_count.max(1));
        let edge = make_edge(&source_str, &target_str);
        let eid = diagram_models::document::EdgeId::new(format!("e{i}"));
        doc.document.edges = doc.document.edges.update(eid, edge);
    }
    doc
}

fn make_doc_with_node(rev: u64) -> DiagramDocument {
    let mut doc = make_doc_with_revision(rev);
    let (id, node) = make_node("n1", 10.0);
    doc.document.nodes = doc.document.nodes.update(id, node);
    doc
}

fn arb_session(doc_rev: u64, saved_rev: u64) -> DocumentSession {
    DocumentSession::new(make_doc_with_revision(saved_rev))
        .with_document(make_doc_with_revision(doc_rev))
}

#[test]
fn from_file_preserves_revision_with_populated_document() {
    let doc = make_doc_with_nodes_and_edges(5, 2, 1);
    let path = PathBuf::from("/tmp/populated.json");
    let session = DocumentSession::from_file(doc, path);
    assert_eq!(session.last_saved_revision(), Revision::new(5));
    assert_eq!(session.document().document.nodes.len(), 2);
    assert_eq!(session.document().document.edges.len(), 1);
}

#[test]
fn mark_saved_clears_dirty_and_preserves_file_path() {
    let clean = DocumentSession::from_file(make_doc_with_revision(1), PathBuf::from("/a/b.json"));
    let dirty = clean.with_document(make_doc_with_revision(3));
    assert!(dirty.is_dirty());
    let saved = dirty.mark_saved();
    assert!(!saved.is_dirty());
    assert_eq!(saved.file_path(), Some(&PathBuf::from("/a/b.json")));
    assert_eq!(saved.last_saved_revision(), Revision::new(3));
}

#[test]
fn mark_saved_on_unsaved_session_preserves_none_path() {
    let clean = DocumentSession::new(make_doc_with_revision(0));
    let dirty = clean.with_document(make_doc_with_revision(2));
    assert!(dirty.is_dirty());
    let saved = dirty.mark_saved();
    assert!(!saved.is_dirty());
    assert_eq!(saved.file_path(), None);
    assert_eq!(saved.last_saved_revision(), Revision::new(2));
}

#[test]
fn mark_saved_on_clean_session_is_idempotent() {
    let session =
        DocumentSession::from_file(make_doc_with_revision(5), PathBuf::from("/saved.json"));
    assert!(!session.is_dirty());
    let saved = session.mark_saved();
    assert!(!saved.is_dirty());
    assert_eq!(saved.file_path(), Some(&PathBuf::from("/saved.json")));
    assert_eq!(saved.last_saved_revision(), Revision::new(5));
}

#[test]
fn mark_saved_returns_new_instance_original_unchanged() {
    let clean = DocumentSession::new(make_doc_with_revision(1));
    let dirty = clean.with_document(make_doc_with_revision(3));
    let saved = dirty.mark_saved();
    assert!(dirty.is_dirty());
    assert_eq!(dirty.last_saved_revision(), Revision::new(1));
    assert!(!saved.is_dirty());
    assert_eq!(saved.last_saved_revision(), Revision::new(3));
}

#[test]
fn mark_saved_preserves_document_identity() {
    let clean = DocumentSession::new(make_doc_with_revision(0));
    let dirty = clean.with_document(make_doc_with_revision(2));
    let saved = dirty.mark_saved();
    assert_eq!(
        saved.document().revision,
        dirty.document().revision,
        "mark_saved must preserve document content"
    );
    assert!(!saved.is_dirty());
}

#[test]
fn from_file_preserves_file_path_across_mark_saved() {
    let session =
        DocumentSession::from_file(make_doc_with_revision(3), PathBuf::from("/persistent.json"));
    let saved = session.mark_saved();
    assert_eq!(saved.file_path(), Some(&PathBuf::from("/persistent.json")));
    assert!(!saved.is_dirty());
}

#[test]
fn with_document_preserves_path_and_saved_revision_recalculates_dirty() {
    let session = DocumentSession::from_file(make_doc_with_revision(3), PathBuf::from("/x.json"));
    assert_eq!(session.file_path(), Some(&PathBuf::from("/x.json")));
    assert_eq!(session.last_saved_revision(), Revision::new(3));
    let new_session = session.with_document(make_doc_with_revision(5));
    assert_eq!(new_session.file_path(), Some(&PathBuf::from("/x.json")));
    assert_eq!(new_session.last_saved_revision(), Revision::new(3));
    assert!(new_session.is_dirty());
}

#[test]
fn with_document_same_revision_stays_clean() {
    let session = DocumentSession::new(make_doc_with_revision(3));
    let new_session = session.with_document(make_doc_with_revision(3));
    assert!(!new_session.is_dirty());
}

#[test]
fn with_document_preserves_doc_identity() {
    let doc = make_doc_with_revision(5);
    let session = DocumentSession::new(doc.clone());
    let same_session = session.with_document(doc);
    assert_eq!(
        same_session.document().revision,
        session.document().revision,
        "with_document must preserve document content"
    );
    assert_eq!(same_session.is_dirty(), session.is_dirty());
    assert_eq!(
        same_session.last_saved_revision(),
        session.last_saved_revision()
    );
    assert_eq!(same_session.file_path(), session.file_path());
}

#[test]
fn with_document_returns_new_instance_original_unchanged() {
    let doc = make_doc_with_revision(3);
    let session = DocumentSession::new(doc);
    let replaced = session.with_document(make_doc_with_revision(5));
    assert_eq!(session.document().revision, Revision::new(3));
    assert!(!session.is_dirty());
    assert_eq!(replaced.document().revision, Revision::new(5));
    assert!(replaced.is_dirty());
}

#[test]
fn set_file_path_preserves_doc_and_revision_recalculates_dirty() {
    let clean = DocumentSession::new(make_doc_with_revision(2));
    let dirty = clean.with_document(make_doc_with_revision(4));
    let renamed = dirty.set_file_path(PathBuf::from("/new/path.json"));
    assert_eq!(renamed.file_path(), Some(&PathBuf::from("/new/path.json")));
    assert_eq!(renamed.last_saved_revision(), Revision::new(2));
    assert!(renamed.is_dirty());
}

#[test]
fn set_file_path_on_dirty_session_does_not_reset_dirty_state() {
    let session = DocumentSession::from_file(make_doc_with_revision(2), PathBuf::from("/old.json"));
    let dirty = session.with_document(make_doc_with_revision(4));
    let renamed = dirty.set_file_path(PathBuf::from("/new.json"));
    assert_eq!(renamed.file_path(), Some(&PathBuf::from("/new.json")));
    assert!(renamed.is_dirty());
}

#[test]
fn set_file_path_on_clean_session_preserves_clean_state() {
    let session = DocumentSession::new(make_doc_with_revision(3));
    let renamed = session.set_file_path(PathBuf::from("/new.json"));
    assert!(!renamed.is_dirty());
    assert_eq!(renamed.file_path(), Some(&PathBuf::from("/new.json")));
    assert_eq!(renamed.last_saved_revision(), Revision::new(3));
}

#[test]
fn set_file_path_clears_path_on_dirty_session() {
    let session = DocumentSession::from_file(make_doc_with_revision(2), PathBuf::from("/old.json"));
    let dirty = session.with_document(make_doc_with_revision(4));
    let renamed = dirty.set_file_path(PathBuf::new());
    assert_eq!(renamed.file_path(), Some(&PathBuf::new()));
    assert!(renamed.is_dirty());
    assert_eq!(renamed.last_saved_revision(), Revision::new(2));
}

#[test]
fn set_file_path_returns_new_instance_original_unchanged() {
    let session = DocumentSession::new(make_doc_with_revision(0));
    let renamed = session.set_file_path(PathBuf::from("/renamed.json"));
    assert_eq!(session.file_path(), None);
    assert!(!session.is_dirty());
    assert_eq!(renamed.file_path(), Some(&PathBuf::from("/renamed.json")));
    assert!(!renamed.is_dirty());
}

#[test]
fn set_file_path_on_clean_session_preserves_existing_path_when_new_path_equals_old() {
    let session =
        DocumentSession::from_file(make_doc_with_revision(5), PathBuf::from("/same.json"));
    let renamed = session.set_file_path(PathBuf::from("/same.json"));
    assert_eq!(renamed.file_path(), Some(&PathBuf::from("/same.json")));
    assert!(!renamed.is_dirty());
    assert_eq!(renamed.last_saved_revision(), Revision::new(5));
}

#[test]
fn set_file_path_preserves_clean_state_when_unchanged() {
    let session =
        DocumentSession::from_file(make_doc_with_revision(3), PathBuf::from("/keep.json"));
    let renamed = session.set_file_path(PathBuf::from("/keep.json"));
    assert!(!renamed.is_dirty());
    assert_eq!(renamed.last_saved_revision(), Revision::new(3));
    assert_eq!(renamed.file_path(), Some(&PathBuf::from("/keep.json")));
}

#[test]
fn unsaved_session_can_be_dirty_without_file_path() {
    let clean = DocumentSession::new(DiagramDocument::default());
    let dirty = clean.with_document(make_doc_with_revision(1));
    assert!(dirty.is_dirty());
    assert_eq!(dirty.file_path(), None);
}

#[test]
fn clean_session_has_matching_revisions() {
    let session_a = DocumentSession::new(DiagramDocument::default());
    assert!(!session_a.is_dirty());
    assert_eq!(
        session_a.document().revision,
        session_a.last_saved_revision()
    );

    let session_b = DocumentSession::from_file(make_doc_with_revision(5), PathBuf::from("/b.json"));
    assert!(!session_b.is_dirty());
    assert_eq!(
        session_b.document().revision,
        session_b.last_saved_revision()
    );

    let clean = DocumentSession::new(make_doc_with_revision(1));
    let dirty = clean.with_document(make_doc_with_revision(3));
    let session_c = dirty.mark_saved();
    assert!(!session_c.is_dirty());
    assert_eq!(
        session_c.document().revision,
        session_c.last_saved_revision()
    );
}

#[test]
fn new_sets_last_saved_revision_to_doc_revision() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert_eq!(session.last_saved_revision(), Revision::INITIAL);
}

#[test]
fn new_creates_non_dirty_session() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert!(!session.is_dirty());
}

#[test]
fn new_with_non_default_revision_preserves_revision() {
    let doc = make_doc_with_revision(42);
    let session = DocumentSession::new(doc);
    assert_eq!(session.last_saved_revision(), Revision::new(42));
    assert!(!session.is_dirty());
}

#[test]
fn new_with_empty_nodes_and_edges() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert!(!session.is_dirty());
    assert!(session.document().document.nodes.is_empty());
    assert!(session.document().document.edges.is_empty());
}

#[test]
fn new_with_revision_max() {
    let doc = make_doc_with_revision(u64::MAX);
    let session = DocumentSession::new(doc);
    assert_eq!(session.last_saved_revision(), Revision::new(u64::MAX));
    assert!(!session.is_dirty());
}

#[test]
fn new_session_has_initial_revision_when_doc_is_default() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert_eq!(session.last_saved_revision(), Revision::INITIAL);
    assert_eq!(session.last_saved_revision().value(), 0);
}

#[test]
fn from_file_sets_file_path() {
    let doc = DiagramDocument::default();
    let path = PathBuf::from("/tmp/test.json");
    let session = DocumentSession::from_file(doc, path.clone());
    assert_eq!(session.file_path(), Some(&path));
}

#[test]
fn from_file_creates_non_dirty_session() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::from_file(doc, PathBuf::from("/tmp/test.json"));
    assert!(!session.is_dirty());
}

#[test]
fn from_file_with_empty_path_string() {
    let doc = DiagramDocument::default();
    let path = PathBuf::from("");
    let session = DocumentSession::from_file(doc, path.clone());
    assert_eq!(session.file_path(), Some(&path));
    assert!(!session.is_dirty());
}

#[test]
fn from_file_with_absolute_path() {
    let doc = DiagramDocument::default();
    let path = PathBuf::from("/home/user/documents/diagram.json");
    let session = DocumentSession::from_file(doc, path.clone());
    assert_eq!(session.file_path(), Some(&path));
    assert!(!session.is_dirty());
}

#[test]
fn is_dirty_returns_false_when_revisions_match() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert!(!session.is_dirty());
}

#[test]
fn is_dirty_with_equal_non_zero_revisions_returns_false() {
    let doc = make_doc_with_revision(10);
    let session = DocumentSession::new(doc);
    assert!(!session.is_dirty());
}

#[test]
fn is_dirty_returns_true_when_revisions_differ() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    let modified_doc = make_doc_with_revision(1);
    let modified_session = session.with_document(modified_doc);
    assert!(modified_session.is_dirty());
}

#[test]
fn document_returns_reference_to_wrapped_doc() {
    let doc = make_doc_with_node(3);
    let session = DocumentSession::new(doc);
    let ref1 = session.document() as *const DiagramDocument;
    let ref2 = session.document() as *const DiagramDocument;
    assert_eq!(
        ref1, ref2,
        "document() must return a stable reference to the same inner document on every call"
    );
    assert_eq!(session.document().revision, Revision::new(3));
    assert_eq!(session.document().document.nodes.len(), 1);
}

#[test]
fn document_returns_same_document_passed_to_from_file() {
    let doc = make_doc_with_nodes_and_edges(7, 2, 1);
    let session = DocumentSession::from_file(doc, PathBuf::from("/test.json"));
    let ref1 = session.document() as *const DiagramDocument;
    let ref2 = session.document() as *const DiagramDocument;
    assert_eq!(
        ref1, ref2,
        "document() must return a stable reference to the same inner document on every call"
    );
    assert_eq!(session.document().revision, Revision::new(7));
    assert_eq!(session.document().document.nodes.len(), 2);
    assert_eq!(session.document().document.edges.len(), 1);
}

#[test]
fn document_preserves_identity_via_ptr_eq() {
    let doc = DiagramDocument::default();
    let session1 = DocumentSession::new(doc.clone());
    let session2 = DocumentSession::new(doc);
    assert!(
        std::ptr::eq(session1.document(), session1.document()),
        "document() must return a stable reference across calls"
    );
    assert!(
        !std::ptr::eq(session1.document(), session2.document()),
        "different sessions must own independent document copies"
    );
}

#[test]
fn new_session_has_no_file_path() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert_eq!(session.file_path(), None);
}

#[test]
fn last_saved_revision_returns_stored_value() {
    let doc = make_doc_with_revision(7);
    let session = DocumentSession::new(doc);
    assert_eq!(session.last_saved_revision(), Revision::new(7));
}

#[test]
fn last_saved_revision_on_new_session_equals_initial() {
    let doc = DiagramDocument::default();
    let session = DocumentSession::new(doc);
    assert_eq!(session.last_saved_revision(), Revision::INITIAL);
    assert_eq!(session.last_saved_revision().value(), 0);
}

proptest! {
    #[test]
    fn prop_new_always_non_dirty(rev in 0u64..1000) {
        let doc = make_doc_with_revision(rev);
        let session = DocumentSession::new(doc);
        prop_assert!(!session.is_dirty());
    }

    #[test]
    fn prop_from_file_always_non_dirty(rev in 0u64..1000, path in "[a-zA-Z0-9/_.]{0,50}") {
        let doc = make_doc_with_revision(rev);
        let session = DocumentSession::from_file(doc, PathBuf::from(path));
        prop_assert!(!session.is_dirty());
    }

    #[test]
    fn prop_mark_saved_always_clears_dirty(doc_rev in 0u64..1000, saved_rev in 0u64..1000) {
        let session = arb_session(doc_rev, saved_rev);
        let saved_session = session.mark_saved();
        prop_assert!(!saved_session.is_dirty());
        prop_assert_eq!(saved_session.last_saved_revision(), session.document().revision);
    }

    #[test]
    fn prop_with_document_preserves_path_and_revision(doc_rev in 0u64..1000, saved_rev in 0u64..1000, new_rev in 0u64..1000) {
        let session = arb_session(doc_rev, saved_rev)
            .set_file_path(PathBuf::from("/test.json"));
        let new_doc = make_doc_with_revision(new_rev);
        let new_session = session.with_document(new_doc);
        prop_assert_eq!(new_session.file_path(), session.file_path());
        prop_assert_eq!(new_session.last_saved_revision(), session.last_saved_revision());
    }

    #[test]
    fn prop_set_file_path_preserves_dirty_state(doc_rev in 0u64..1000, saved_rev in 0u64..1000, path in "[a-zA-Z0-9/_.]{0,50}") {
        let session = arb_session(doc_rev, saved_rev);
        let expected_dirty = session.is_dirty();
        let new_session = session.set_file_path(PathBuf::from(path));
        prop_assert_eq!(new_session.is_dirty(), expected_dirty);
        prop_assert_eq!(new_session.last_saved_revision(), session.last_saved_revision());
        prop_assert_eq!(new_session.document().revision, session.document().revision);
    }

    #[test]
    fn prop_is_dirty_equivalence(doc_rev in 0u64..1000, saved_rev in 0u64..1000) {
        let session = arb_session(doc_rev, saved_rev);
        prop_assert_eq!(
            session.is_dirty(),
            session.document().revision != session.last_saved_revision()
        );
    }
}

#[test]
fn is_dirty_equivalence_at_u64_max_boundary() {
    let session_max_dirty = arb_session(u64::MAX, 0);
    assert!(session_max_dirty.is_dirty());

    let session_max_clean = arb_session(u64::MAX, u64::MAX);
    assert!(!session_max_clean.is_dirty());
}
