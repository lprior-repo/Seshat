use std::path::PathBuf;

use diagram_models::document::DiagramDocument;

use super::file_lock::FileLock;

/// State of a diagram in the lock manager.
pub type DiagramMutation =
    dyn Send + FnMut(&mut DiagramDocument) -> Result<(), crate::mutation::error::MutationError>;

pub(crate) struct DiagramState {
    /// Pending mutations waiting to be processed
    #[allow(clippy::type_complexity)]
    pub(crate) queue: Vec<Box<DiagramMutation>>,
    /// Currently processing a mutation
    pub(crate) processing: bool,
    /// File lock for this diagram
    pub(crate) file_lock: Option<FileLock>,
    /// Path to the diagram file
    pub(crate) file_path: Option<PathBuf>,
}

impl DiagramState {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            queue: Vec::new(),
            processing: false,
            file_lock: None,
            file_path: None,
        }
    }

    pub(crate) fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }
}
