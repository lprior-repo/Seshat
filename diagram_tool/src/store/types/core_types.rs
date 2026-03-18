use crate::store_async::AsyncStoreError as StoreError;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidEvent {
    pub op_id: ValidOperationId,
    pub timestamp: ValidTimestamp,
    pub payload: ValidPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundedBatch<const MIN: usize, const MAX: usize>(Vec<ValidEvent>);

impl<const MIN: usize, const MAX: usize> TryFrom<Vec<ValidEvent>> for BoundedBatch<MIN, MAX> {
    type Error = StoreError;

    fn try_from(vec: Vec<ValidEvent>) -> Result<Self, Self::Error> {
        if vec.len() < MIN {
            return Err(StoreError::EmptyBatch);
        }
        if vec.len() > MAX {
            return Err(StoreError::BatchTooLarge);
        }
        Ok(Self(vec))
    }
}

impl<const MIN: usize, const MAX: usize> BoundedBatch<MIN, MAX> {
    #[must_use]
    pub fn into_inner(self) -> Vec<ValidEvent> {
        self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidTimestamp(NonZeroU64);

impl ValidTimestamp {
    pub fn new(ts: u64) -> Result<Self, StoreError> {
        NonZeroU64::new(ts)
            .map(Self)
            .ok_or(StoreError::InvalidTimestamp)
    }

    #[must_use]
    pub const fn get(&self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidOperationId(String);

impl ValidOperationId {
    pub fn new(id: String) -> Result<Self, StoreError> {
        if id.is_empty() || id.trim().is_empty() || id.contains('\0') {
            return Err(StoreError::InvalidOperationId);
        }
        if id.len() > 255 {
            return Err(StoreError::OperationIdTooLong);
        }
        Ok(Self(id))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidPayload(String);

impl ValidPayload {
    pub fn new(payload: String) -> Result<Self, StoreError> {
        if payload.len() > 100 * 1024 * 1024 {
            return Err(StoreError::PayloadTooLarge);
        }
        Ok(Self(payload))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(u64);

impl Revision {
    pub fn new(rev: u64) -> Result<Self, StoreError> {
        Ok(Self(rev))
    }

    #[must_use]
    pub const fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppendOutcome {
    pub revision: i64,
    pub op_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    pub op_id: String,
    pub revision: i64,
    pub timestamp: i64,
    pub payload: String,
}
