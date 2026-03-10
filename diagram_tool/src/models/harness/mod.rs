//! Harness types
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VerifyError {
    #[error("determinism failure: {0}")] DeterminismFailure(String),
    #[error("test harness error: {0}")] TestHarness(String),
    #[error("timeout: {0}")] Timeout(String),
    #[error("test failed: {0}")] TestFailure(String),
    #[error("IO error: {0}")] Io(String),
    #[error("SQLite error: {0}")] Sqlite(String),
    #[error("serialization error: {0}")] Serialization(String),
    #[error("conflict policy failure: {0}")] ConflictPolicyFailure(String),
}

impl From<std::io::Error> for VerifyError { fn from(err: std::io::Error) -> Self { Self::Io(err.to_string()) } }
impl From<SqlxError> for VerifyError { fn from(err: SqlxError) -> Self { Self::Sqlite(err.to_string()) } }
impl From<crate::store_async::AsyncStoreError> for VerifyError {
    fn from(err: crate::store_async::AsyncStoreError) -> Self {
        match err { crate::store_async::AsyncStoreError::Io(e) => Self::Io(e.to_string()), crate::store_async::AsyncStoreError::Sqlx(e) => Self::Sqlite(e.to_string()), other => Self::TestFailure(other.to_string()) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzReport { pub seed: u64, pub cases_run: usize, pub projection_hash: String, pub passed: bool, pub error_message: Option<String> }
impl FuzzReport {
    fn passing(seed: u64, cases_run: usize, projection_hash: String) -> Self { Self { seed, cases_run, projection_hash, passed: true, error_message: None } }
    fn failing(seed: u64, cases_run: usize, message: impl Into<String>) -> Self { Self { seed, cases_run, projection_hash: String::new(), passed: false, error_message: Some(message.into()) } }
}

#[derive(Debug, Clone)]
pub struct TestReport { pub passed: bool, pub tests_run: usize, pub failures: Vec<TestFailure> }
#[derive(Debug, Clone)]
pub struct TestFailure { pub test_name: String, pub message: String }

pub mod fixtures;
pub use fixtures::*;
