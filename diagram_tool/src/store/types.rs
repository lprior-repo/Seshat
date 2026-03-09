use super::errors::StoreError;
use serde::Serialize;
use std::num::NonZeroI64;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
pub struct StoreRevision(NonZeroI64);

impl StoreRevision {
    pub fn new(rev: i64) -> Result<Self, StoreError> {
        NonZeroI64::new(rev)
            .filter(|r| r.get() >= 1)
            .map(Self)
            .ok_or_else(|| StoreError::ValidationFailed("revision must be at least 1".to_string()))
    }
    pub fn get(&self) -> i64 {
        self.0.get()
    }
}
impl PartialEq<i64> for StoreRevision {
    fn eq(&self, other: &i64) -> bool {
        self.0.get() == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Timestamp(NonZeroI64);

impl Timestamp {
    pub fn new(ts: i64) -> Result<Self, StoreError> {
        NonZeroI64::new(ts)
            .filter(|t| t.get() > 0)
            .map(Self)
            .ok_or_else(|| StoreError::ValidationFailed("timestamp must be positive".to_string()))
    }
    pub fn get(&self) -> i64 {
        self.0.get()
    }
}
impl PartialEq<i64> for Timestamp {
    fn eq(&self, other: &i64) -> bool {
        self.0.get() == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct OpId(String);

impl OpId {
    pub fn new(id: String) -> Result<Self, StoreError> {
        if id.is_empty() {
            Err(StoreError::ValidationFailed(
                "op_id must not be empty".to_string(),
            ))
        } else {
            Ok(Self(id))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_inner(self) -> String {
        self.0
    }
}
impl PartialEq<&str> for OpId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
impl PartialEq<String> for OpId {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendOutcome {
    pub revision: StoreRevision,
    pub op_id: OpId,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendResult {
    pub revision: StoreRevision,
    pub op_id: OpId,
    pub timestamp: Timestamp,
}

impl From<AppendResult> for AppendOutcome {
    fn from(result: AppendResult) -> Self {
        Self {
            revision: result.revision,
            op_id: result.op_id,
            timestamp: result.timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchAppendResult {
    pub start_revision: i64,
    pub end_revision: i64,
    pub count: usize,
    pub op_ids: Vec<String>,
    pub last_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub op_id: OpId,
    pub revision: StoreRevision,
    pub timestamp: Timestamp,
    pub payload: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    Exact,
    Conflict,
}

pub struct StoreConnection {
    pub conn: rusqlite::Connection,
}
