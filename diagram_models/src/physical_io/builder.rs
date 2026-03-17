use crate::document::DiagramDocument;

pub struct DiagramBuilder {
    doc: DiagramDocument,
}

impl DiagramBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            doc: DiagramDocument::default(),
        }
    }

    #[must_use]
    pub fn build(self) -> DiagramDocument {
        self.doc
    }
}

impl Default for DiagramBuilder {
    fn default() -> Self {
        Self::new()
    }
}
