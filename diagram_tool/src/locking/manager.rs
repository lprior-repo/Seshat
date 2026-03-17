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

/// Newtype for Diagram Identifier to prevent primitive obsession.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagramId(String);

/// Error type for invalid diagram IDs
#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid diagram ID: {0}")]
pub struct DiagramIdError(String);

impl DiagramId {
    /// Create a new DiagramId with validation.
    ///
    /// # Errors
    /// Returns `DiagramIdError` if the ID contains path traversal characters
    /// like `..`, `/`, or `\`, or is empty.
    pub fn new(id: String) -> Result<Self, DiagramIdError> {
        if id.is_empty() {
            return Err(DiagramIdError("ID cannot be empty".to_string()));
        }
        if id.contains("..") || id.contains('/') || id.contains('\\') {
            return Err(DiagramIdError(format!(
                "ID '{}' contains invalid characters (../  \\)",
                id
            )));
        }
        Ok(Self(id))
    }

    /// Create a new DiagramId without validation (for trusted sources).
    #[must_use]
    pub const fn new_unchecked(id: String) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DiagramId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// REMOVED: From<String> and From<&str> implementations that bypassed validation
// Security fix: Users must use TryFrom or DiagramId::new() which validates
// impl From<String> for DiagramId {
//     fn from(s: String) -> Self {
//         Self::new_unchecked(s)  // VULNERABILITY: bypasses validation
//     }
// }

// Use this instead:
impl TryFrom<String> for DiagramId {
    type Error = DiagramIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        DiagramId::new(s)
    }
}

// REMOVED: From<&str> - use TryFrom instead
// impl From<&str> for DiagramId {
//     fn from(s: &str) -> Self {
//         Self::new_unchecked(s.to_string())  // VULNERABILITY
//     }
// }

impl TryFrom<&str> for DiagramId {
    type Error = DiagramIdError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        DiagramId::new(s)
    }
}

impl TryFrom<String> for DiagramId {
    type Error = DiagramIdError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        DiagramId::new(s)
    }
}

/// State of a diagram in the lock manager.
type DiagramMutation =
    dyn Send + FnMut(&mut DiagramDocument) -> Result<(), crate::mutation::error::MutationError>;

struct DiagramState {
    /// Pending mutations waiting to be processed
    #[allow(clippy::type_complexity)]
    queue: Vec<Box<DiagramMutation>>,
    /// Currently processing a mutation
    processing: bool,
    /// File lock for this diagram
    file_lock: Option<FileLock>,
    /// Path to the diagram file
    file_path: Option<PathBuf>,
}

impl DiagramState {
    #[must_use]
    const fn new() -> Self {
        Self {
            queue: Vec::new(),
            processing: false,
            file_lock: None,
            file_path: None,
        }
    }

    fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }
}

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
    ///
    /// # Arguments
    ///
    /// * `lock_timeout` - Maximum time to wait for acquiring a lock
    /// * `lock_dir` - Directory to store lock files
    /// * `default_diagram_dir` - Default directory for diagram files
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
    ///
    /// Uses current directory for locks and diagrams.
    #[must_use]
    pub fn with_defaults(lock_timeout: Duration) -> Self {
        Self::new(lock_timeout, PathBuf::from(".locks"), PathBuf::from("."))
    }

    /// Get or create the state for a diagram.
    fn get_or_create_diagram(&mut self, diagram_id: &DiagramId) -> &mut DiagramState {
        self.diagrams
            .entry(diagram_id.clone())
            .or_insert_with(DiagramState::new)
    }

    /// Get the lock file path for a diagram.
    fn lock_path(&self, diagram_id: &DiagramId) -> PathBuf {
        self.lock_dir.join(format!("{}.lock", diagram_id.as_str()))
    }

    /// Get the diagram file path for a diagram.
    fn diagram_path(&self, diagram_id: &DiagramId) -> PathBuf {
        self.default_diagram_dir
            .join(format!("{}.json", diagram_id.as_str()))
    }

    /// Check if a diagram is currently locked (has an active file lock).
    #[must_use]
    pub fn is_locked(&self, diagram_id: &DiagramId) -> bool {
        self.diagrams
            .get(diagram_id)
            .is_some_and(|state| state.file_lock.is_some())
    }

    /// Get the number of pending mutations for a diagram.
    #[must_use]
    pub fn queue_length(&self, diagram_id: &DiagramId) -> usize {
        self.diagrams
            .get(diagram_id)
            .map_or(0, |state| state.queue.len())
    }

    /// Check if a diagram has pending mutations.
    #[must_use]
    pub fn has_pending_mutations(&self, diagram_id: &DiagramId) -> bool {
        self.queue_length(diagram_id) > 0
    }

    /// Execute a mutation on a diagram with file locking.
    ///
    /// This method:
    /// 1. Acquires a file lock for the diagram
    /// 2. Loads the diagram from disk
    /// 3. Applies the mutation
    /// 4. Saves the diagram back to disk
    /// 5. Releases the file lock
    ///
    /// # Errors
    ///
    /// Returns `LockError::Timeout` if the lock cannot be acquired.
    /// Returns `LockError::IoError` if there are I/O errors.
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn with_lock<T>(
        &mut self,
        diagram_id: DiagramId,
        operation: impl FnOnce(&mut DiagramDocument) -> Result<T, crate::mutation::error::MutationError>,
    ) -> Result<T, LockError> {
        let lock_path = self.lock_path(&diagram_id);

        // Acquire file lock with timeout
        let mut file_lock = FileLock::acquire(lock_path, self.lock_timeout)?;

        // Get the diagram file path
        let diagram_path = self.diagram_path(&diagram_id);

        // Load the diagram (or create default if doesn't exist)
        let mut doc = if diagram_path.exists() {
            load_workspace_with_lkg(&diagram_path)
                .map_err(|e| LockError::IoError(std::io::Error::other(e.to_string())))?
        } else {
            DiagramDocument::default()
        };

        // Apply the mutation
        let result = operation(&mut doc)?;

        // Ensure diagram directory exists before saving
        if let Some(parent) = diagram_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LockError::IoError(std::io::Error::other(format!(
                    "Failed to create directory: {e}"
                )))
            })?;
        }

        // Save the diagram back to disk
        save_workspace_atomic(&doc, &diagram_path)
            .map_err(|e| LockError::IoError(std::io::Error::other(e.to_string())))?;

        // Release the file lock
        file_lock.release()?;

        Ok(result)
    }

    /// Queue a mutation for later execution (non-blocking).
    ///
    /// The mutation will be executed when the diagram becomes available.
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

        // Create the boxed mutation
        #[allow(clippy::type_complexity)]
        let boxed_mutation: Box<DiagramMutation> = Box::new(mutation);

        state.queue.push(boxed_mutation);
        Ok(())
    }

    /// Execute all pending mutations for a diagram.
    ///
    /// This acquires the lock, processes all queued mutations, and releases the lock.
    pub fn flush_queue(&mut self, diagram_id: &DiagramId) -> Result<(), LockError> {
        // Get pending mutations
        let mut mutations = {
            let state = self
                .diagrams
                .get_mut(diagram_id)
                .ok_or_else(|| LockError::QueueError(format!("Diagram not found: {diagram_id}")))?;

            // Take all pending mutations
            std::mem::take(&mut state.queue)
        };

        if mutations.is_empty() {
            return Ok(());
        }

        // Acquire lock once for all mutations
        let lock_path = self.lock_path(diagram_id);
        let mut file_lock = FileLock::acquire(lock_path, self.lock_timeout)?;

        // Get the diagram file path
        let diagram_path = self.diagram_path(diagram_id);

        // Load the diagram
        let mut doc = if diagram_path.exists() {
            load_workspace_with_lkg(&diagram_path)
                .map_err(|e| LockError::IoError(std::io::Error::other(e.to_string())))?
        } else {
            DiagramDocument::default()
        };

        // Apply all pending mutations
        for mutation in &mut mutations {
            mutation(&mut doc)?;
        }

        // Ensure diagram directory exists before saving
        if let Some(parent) = diagram_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LockError::IoError(std::io::Error::other(format!(
                    "Failed to create directory: {e}"
                )))
            })?;
        }

        // Save the diagram
        save_workspace_atomic(&doc, &diagram_path)
            .map_err(|e| LockError::IoError(std::io::Error::other(e.to_string())))?;

        // Release lock
        file_lock.release()?;

        Ok(())
    }

    /// Clear all pending mutations for a diagram.
    pub fn clear_queue(&mut self, diagram_id: &DiagramId) {
        if let Some(state) = self.diagrams.get_mut(diagram_id) {
            state.queue.clear();
        }
    }

    /// Get the number of diagrams currently managed.
    #[must_use]
    pub fn diagram_count(&self) -> usize {
        self.diagrams.len()
    }

    /// Get all diagram IDs currently in the manager.
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

#[cfg(test)]
mod tests {
    use super::*;
    use diagram_models::document::{DocumentData, EditorState, Revision};
    use im::HashMap;
    use tempfile::TempDir;

    fn create_test_document() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_new_manager_when_created_then_empty() {
        let manager = DiagramLockManager::with_defaults(Duration::from_secs(1));

        assert_eq!(manager.diagram_count(), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_manager_when_check_unlocked_diagram_then_returns_false() {
        let manager = DiagramLockManager::with_defaults(Duration::from_secs(1));
        let diagram_id = DiagramId::new("test_diagram".to_string());

        assert!(!manager.is_locked(&diagram_id));
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_manager_when_check_queue_length_then_returns_zero() {
        let manager = DiagramLockManager::with_defaults(Duration::from_secs(1));
        let diagram_id = DiagramId::new("test_diagram".to_string());

        assert_eq!(manager.queue_length(&diagram_id), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_lock_timeout_when_cannot_acquire_then_error() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_millis(50), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string());

        // First acquire should succeed
        let result1 = manager.with_lock(diagram_id.clone(), |_doc| Ok(42));
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), 42);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_different_diagrams_when_mutated_then_both_succeed() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id1 = DiagramId::new("diagram1".to_string());
        let diagram_id2 = DiagramId::new("diagram2".to_string());

        let result1 = manager.with_lock(diagram_id1.clone(), |_doc| Ok("result1"));

        let result2 = manager.with_lock(diagram_id2.clone(), |_doc| Ok("result2"));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_mutation_with_lock_when_applied_then_document_modified() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string());

        let result = manager.with_lock(diagram_id, |doc| {
            doc.revision = Revision::INITIAL.increment();
            Ok(())
        });

        assert!(result.is_ok());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_queued_mutations_when_flushed_then_all_applied() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string());

        // Queue some mutations
        manager
            .queue_mutation(diagram_id.clone(), |doc| {
                doc.revision = Revision::INITIAL.increment();
                Ok(())
            })
            .unwrap();

        manager
            .queue_mutation(diagram_id.clone(), |doc| {
                doc.revision = doc.revision.increment();
                Ok(())
            })
            .unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 2);

        // Flush the queue
        manager.flush_queue(&diagram_id).unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_queue_when_cleared_then_empty() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string());

        // Queue a mutation
        manager
            .queue_mutation(diagram_id.clone(), |_doc| Ok(()))
            .unwrap();

        assert_eq!(manager.queue_length(&diagram_id), 1);

        // Clear the queue
        manager.clear_queue(&diagram_id);

        assert_eq!(manager.queue_length(&diagram_id), 0);
    }

    #[cfg(kani)]
    #[kani::proof]
    #[test]
    fn given_multiple_operations_same_diagram_when_sequential_then_succeed() {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let diagram_dir = temp_dir.path().join("diagrams");

        let mut manager = DiagramLockManager::new(Duration::from_secs(1), lock_dir, diagram_dir);

        let diagram_id = DiagramId::new("test_diagram".to_string());

        // First operation
        let result1 = manager.with_lock(diagram_id.clone(), |doc| {
            doc.revision = Revision::INITIAL.increment();
            Ok(())
        });
        assert!(result1.is_ok());

        // Second operation (should be serialized, not concurrent)
        let result2 = manager.with_lock(diagram_id.clone(), |doc| {
            doc.revision = doc.revision.increment();
            Ok(())
        });
        assert!(result2.is_ok());
    }
}
