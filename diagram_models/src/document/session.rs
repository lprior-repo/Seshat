use std::path::PathBuf;

use crate::document::{DiagramDocument, Revision};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSession {
    doc: DiagramDocument,
    file_path: Option<PathBuf>,
    last_saved_revision: Revision,
}

impl DocumentSession {
    #[must_use]
    pub const fn new(doc: DiagramDocument) -> Self {
        let last_saved_revision = doc.revision;
        Self {
            doc,
            file_path: None,
            last_saved_revision,
        }
    }

    #[must_use]
    pub const fn from_file(doc: DiagramDocument, path: PathBuf) -> Self {
        let last_saved_revision = doc.revision;
        Self {
            doc,
            file_path: Some(path),
            last_saved_revision,
        }
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.doc.revision != self.last_saved_revision
    }

    #[must_use]
    pub fn mark_saved(&self) -> Self {
        Self {
            doc: self.doc.clone(),
            file_path: self.file_path.clone(),
            last_saved_revision: self.doc.revision,
        }
    }

    #[must_use]
    pub const fn document(&self) -> &DiagramDocument {
        &self.doc
    }

    #[must_use]
    pub const fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    #[must_use]
    pub const fn last_saved_revision(&self) -> Revision {
        self.last_saved_revision
    }

    #[must_use]
    pub fn with_document(&self, doc: DiagramDocument) -> Self {
        Self {
            doc,
            file_path: self.file_path.clone(),
            last_saved_revision: self.last_saved_revision,
        }
    }

    #[must_use]
    pub fn set_file_path(&self, path: PathBuf) -> Self {
        Self {
            doc: self.doc.clone(),
            file_path: Some(path),
            last_saved_revision: self.last_saved_revision,
        }
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    fn make_doc_with_revision(rev: u64) -> DiagramDocument {
        let mut doc = DiagramDocument::default();
        doc.revision = Revision::new(rev);
        doc
    }

    #[kani::proof]
    fn kani_is_dirty_matches_revision_comparison() {
        let doc_rev: u64 = kani::any();
        let saved_rev: u64 = kani::any();
        let doc = make_doc_with_revision(doc_rev);
        let session = DocumentSession {
            doc,
            file_path: None,
            last_saved_revision: Revision::new(saved_rev),
        };
        assert_eq!(
            session.is_dirty(),
            session.document().revision != session.last_saved_revision()
        );
    }

    #[kani::proof]
    fn kani_mark_saved_syncs_revision() {
        let doc_rev: u64 = kani::any();
        let saved_rev: u64 = kani::any();
        let doc = make_doc_with_revision(doc_rev);
        let session = DocumentSession {
            doc,
            file_path: None,
            last_saved_revision: Revision::new(saved_rev),
        };
        let saved = session.mark_saved();
        assert_eq!(saved.last_saved_revision(), session.document().revision);
        assert_eq!(saved.is_dirty(), false);
    }

    #[kani::proof]
    fn kani_with_document_preserves_session_metadata() {
        let doc_rev: u64 = kani::any();
        let saved_rev: u64 = kani::any();
        let new_doc_rev: u64 = kani::any();
        let doc = make_doc_with_revision(doc_rev);
        let session = DocumentSession {
            doc,
            file_path: Some(PathBuf::from("/test.json")),
            last_saved_revision: Revision::new(saved_rev),
        };
        let new_doc = make_doc_with_revision(new_doc_rev);
        let result = session.with_document(new_doc);
        assert_eq!(result.file_path(), session.file_path());
        assert_eq!(result.last_saved_revision(), session.last_saved_revision());
    }
}
