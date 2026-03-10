//! File locking implementation using OS-level file locks.
//!
//! Provides acquire/release semantics with proper cleanup on drop.
//! Uses the `fs2` crate for cross-platform file locking.

use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::Duration;

use super::error::LockError;
use fs2::FileExt;

/// A file lock that provides exclusive access to a resource.
///
/// The lock is automatically released when the `FileLock` is dropped.
pub struct FileLock {
    #[allow(dead_code)]
    path: PathBuf,
    file: Option<File>,
}

impl FileLock {
    /// Acquire a file lock with the given timeout.
    ///
    /// # Errors
    ///
    /// Returns `LockError::Timeout` if the lock cannot be acquired within the timeout.
    /// Returns `LockError::IoError` if there are I/O errors.
    pub fn acquire(path: PathBuf, timeout: Duration) -> Result<Self, LockError> {
        // Ensure the lock directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(LockError::IoError)?;
        }

        // Open or create the lock file (DO NOT truncate - would lose data from concurrent lock holder)
        let file = OpenOptions::new()
            .create(true)
            .truncate(false) // FIXED: was true which could truncate data from lock holder
            .read(true)
            .write(true)
            .open(&path)
            .map_err(LockError::IoError)?;

        // Try to acquire an exclusive lock with timeout
        let start = std::time::Instant::now();
        let mut retries = 0;

        loop {
            match file.try_lock_exclusive() {
                Ok(()) => {
                    // Successfully acquired lock
                    return Ok(Self {
                        path,
                        file: Some(file),
                    });
                }
                Err(_e) => {
                    // Lock is held by another process
                    if start.elapsed() >= timeout {
                        return Err(LockError::Timeout(format!(
                            "Failed to acquire lock for {} within {:?}",
                            path.display(),
                            timeout
                        )));
                    }

                    // Exponential backoff with jitter
                    let delay = Duration::from_millis(10 << retries.min(10));
                    std::thread::sleep(delay);
                    retries += 1;
                }
            }
        }
    }

    /// Check if the lock file exists and is locked.
    #[allow(dead_code)]
    pub fn is_locked(path: &PathBuf) -> bool {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .is_ok_and(|file| file.try_lock_exclusive().is_err())
    }

    /// Release the lock early (also happens on drop).
    pub fn release(&mut self) -> Result<(), LockError> {
        if let Some(file) = self.file.take() {
            // Release the lock
            file.unlock().map_err(LockError::IoError)?;
            // Close the file
            drop(file);
        }
        Ok(())
    }

    /// Get the path to the lock file.
    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            // Release lock and close file
            let _ = file.unlock();
            // File is automatically closed when dropped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn given_lock_file_when_acquired_then_held() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        let lock = FileLock::acquire(lock_path.clone(), Duration::from_secs(1));

        assert!(lock.is_ok());
    }

    #[test]
    fn given_lock_file_when_dropped_then_released() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        {
            let lock = FileLock::acquire(lock_path.clone(), Duration::from_secs(1)).unwrap();
            assert!(FileLock::is_locked(&lock_path));
        }

        // After drop, lock should be released
        let lock2 = FileLock::acquire(lock_path.clone(), Duration::from_secs(1));
        assert!(lock2.is_ok());
    }

    #[test]
    fn given_lock_timeout_when_cannot_acquire_then_error() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("test.lock");

        // Acquire first lock
        let _lock1 = FileLock::acquire(lock_path.clone(), Duration::from_secs(1)).unwrap();

        // Try to acquire second lock with very short timeout
        let lock2 = FileLock::acquire(lock_path, Duration::from_millis(50));

        assert!(lock2.is_err());
        assert!(matches!(lock2.err(), Some(LockError::Timeout(_))));
    }
}
