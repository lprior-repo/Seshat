//! Diagram lock manager - provides per-diagram queue and file lock discipline.
//!
//! This module implements:
//! - Per-diagram mutation serialization via queues
//! - File-level locking for cross-process safety  
//! - Parallel work across different diagrams
//! - Integration with atomic persistence from `cli_persistence`

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::cli_persistence::{load_workspace_with_lkg, save_workspace_atomic};
use diagram_models::document::DiagramDocument;

use super::error::LockError;
use super::file_lock::FileLock;
use super::id::DiagramId;
use super::state::DiagramState;

/// Lock manager for all diagrams.
///
/// Provides:
/// - Per-diagram mutation queues for serialization
/// - File-level locking for cross-process safety
/// - Parallel work across different diagrams
/// - Integration with atomic persistence
pub struct DiagramLockManager {
    /// Diagram states indexed by diagram ID
    diagrams: HashMap<DiagramId, DiagramState>,
    /// Timeout for lock acquisition
    lock_timeout: Duration,
    /// Directory for lock files
    lock_dir: PathBuf,
    /// Default directory for diagram files
    default_diagram_dir: PathBuf,
}

impl DiagramLockManager {
    /// Create a new lock manager.
    #[must_use]
    pub fn new(lock_timeout: Duration, lock_dir: PathBuf, default_diagram_dir: PathBuf) -> Self {
        Self {
            diagrams: HashMap::new(),
            lock_timeout,
            lock_dir,
            default_diagram_dir,
        }
    }

    /// Create a new lock manager with default directories.
    #[must_use]
    pub fn with_defaults(lock_timeout: Duration) -> Self {
        Self::new(lock_timeout, PathBuf::from(".locks"), PathBuf::from("."))
    }

    fn get_or_create_diagram(&mut self, diagram_id: &DiagramId) -> &mut DiagramState {
        self.diagrams
            .entry(diagram_id.clone())
            .or_insert_with(DiagramState::new)
    }

    fn lock_path(&self, diagram_id: &DiagramId) -> PathBuf {
        self.lock_dir.join(format!("{}.lock", diagram_id.as_str()))
    }

    fn diagram_path(&self, diagram_id: &DiagramId) -> PathBuf {
        self.default_diagram_dir
            .join(format!("{}.json", diagram_id.as_str()))
    }

    #[must_use]
    pub fn is_locked(&self, diagram_id: &DiagramId) -> bool {
        self.diagrams
            .get(diagram_id)
            .is_some_and(|state| state.file_lock.is_some())
    }

    #[must_use]
    pub fn queue_length(&self, diagram_id: &DiagramId) -> usize {
        self.diagrams
            .get(diagram_id)
            .map_or(0, |state| state.queue.len())
    }

    #[must_use]
    pub fn has_pending_mutations(&self, diagram_id: &DiagramId) -> bool {
        self.queue_length(diagram_id) > 0
    }

    fn with_document_transaction<T, F>(
        lock_path: PathBuf,
        diagram_path: PathBuf,
        lock_timeout: Duration,
        mut operation: F,
    ) -> Result<T, LockError>
    where
        F: FnMut(&mut DiagramDocument) -> Result<T, crate::mutation::error::MutationError>,
    {
        let mut file_lock = FileLock::acquire(lock_path, lock_timeout)?;

        let mut doc = if diagram_path.exists() {
            load_workspace_with_lkg(&diagram_path)
                .map_err(|e| LockError::IoError(std::io::Error::other(e.to_string())))?
        } else {
            DiagramDocument::default()
        };

        let result = operation(&mut doc)?;

        if let Some(parent) = diagram_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LockError::IoError(std::io::Error::other(format!(
                    "Failed to create directory: {e}"
                )))
            })?;
        }

        save_workspace_atomic(&doc, &diagram_path)
            .map_err(|e| LockError::IoError(std::io::Error::other(e.to_string())))?;

        file_lock.release()?;

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn with_lock<T>(
        &mut self,
        diagram_id: DiagramId,
        operation: impl FnOnce(&mut DiagramDocument) -> Result<T, crate::mutation::error::MutationError>,
    ) -> Result<T, LockError> {
        let lock_path = self.lock_path(&diagram_id);
        let diagram_path = self.diagram_path(&diagram_id);

        let mut opt_op = Some(operation);
        Self::with_document_transaction(lock_path, diagram_path, self.lock_timeout, |doc| {
            if let Some(op) = opt_op.take() {
                op(doc)
            } else {
                unreachable!("Operation called twice in with_lock");
            }
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    pub fn queue_mutation(
        &mut self,
        diagram_id: DiagramId,
        mutation: impl FnMut(&mut DiagramDocument) -> Result<(), crate::mutation::error::MutationError>
            + Send
            + 'static,
    ) -> Result<(), LockError> {
        let state = self.get_or_create_diagram(&diagram_id);
        state.queue.push(Box::new(mutation));
        Ok(())
    }

    pub fn flush_queue(&mut self, diagram_id: &DiagramId) -> Result<(), LockError> {
        let mut mutations = {
            let state = self
                .diagrams
                .get_mut(diagram_id)
                .ok_or_else(|| LockError::QueueError(format!("Diagram not found: {diagram_id}")))?;
            std::mem::take(&mut state.queue)
        };

        if mutations.is_empty() {
            return Ok(());
        }

        let lock_path = self.lock_path(diagram_id);
        let diagram_path = self.diagram_path(diagram_id);

        Self::with_document_transaction(lock_path, diagram_path, self.lock_timeout, |doc| {
            for mutation in &mut mutations {
                mutation(doc)?;
            }
            Ok(())
        })
    }

    pub fn clear_queue(&mut self, diagram_id: &DiagramId) {
        if let Some(state) = self.diagrams.get_mut(diagram_id) {
            state.queue.clear();
        }
    }

    #[must_use]
    pub fn diagram_count(&self) -> usize {
        self.diagrams.len()
    }

    #[must_use]
    pub fn diagram_ids(&self) -> Vec<DiagramId> {
        self.diagrams.keys().cloned().collect()
    }
}

impl Default for DiagramLockManager {
    fn default() -> Self {
        Self::with_defaults(Duration::from_secs(5))
    }
}
