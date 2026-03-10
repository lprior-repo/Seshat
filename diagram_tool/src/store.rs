//! `SQLite` storage module
//!
//! Provides SQLite-based storage.

#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use crate::models::envelope::EventEnvelope;
use crate::store_async::{self, AsyncStoreError, EventRecord};
use std::path::Path;

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// A validated event ready to be appended
pub struct ValidEvent(pub EventEnvelope);

impl ValidEvent {
    pub const fn new(envelope: EventEnvelope) -> Self {
        Self(envelope)
    }

    pub fn into_inner(self) -> EventEnvelope {
        self.0
    }
}

/// A session for read-only operations
pub struct ReadOnlySession {
    pub pool: sqlx::SqlitePool,
}

/// A session for read-write operations
pub struct ReadWriteSession {
    pub pool: sqlx::SqlitePool,
}

impl ReadWriteSession {
    pub const fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

pub struct StoreBootstrap {
    pub pool: sqlx::SqlitePool,
    pub db_path: std::path::PathBuf,
    pub schema_version: i32,
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|_| unreachable!());
    rt.block_on(f)
}

pub fn bootstrap_store(db_path: &Path) -> Result<StoreBootstrap, AsyncStoreError> {
    let b = block_on(store_async::bootstrap_async_store(db_path))?;
    Ok(StoreBootstrap {
        pool: b.pool,
        db_path: b.db_path,
        schema_version: b.schema_version,
    })
}

pub fn append_event(
    session: &mut ReadWriteSession,
    event: ValidEvent,
    expected_revision: Option<i64>,
) -> Result<store_async::AsyncAppendResult, AsyncStoreError> {
    block_on(store_async::append_event_async(
        &session.pool,
        event.into_inner(),
        expected_revision,
    ))
}

pub fn append_idempotent(
    session: &mut ReadWriteSession,
    event: ValidEvent,
) -> Result<store_async::AsyncAppendResult, AsyncStoreError> {
    block_on(store_async::append_idempotent_async(
        &session.pool,
        event.into_inner(),
    ))
}

pub fn open_recovery_mode(db_path: &Path) -> Result<ReadOnlySession, store_async::AsyncStoreError> {
    let handle = block_on(store_async::open_recovery_mode_async(db_path))?;
    Ok(ReadOnlySession { pool: handle })
}

pub fn fetch_all_events(session: &ReadOnlySession) -> Result<Vec<EventRecord>, AsyncStoreError> {
    block_on(store_async::fetch_all_events(&session.pool))
}

pub fn fetch_latest_revision(session: &ReadOnlySession) -> Result<i64, AsyncStoreError> {
    block_on(store_async::fetch_latest_revision(&session.pool))
}

pub fn fetch_events_since(
    session: &ReadOnlySession,
    revision: i64,
) -> Result<Vec<EventRecord>, AsyncStoreError> {
    block_on(store_async::fetch_events_since(&session.pool, revision))
}

pub fn integrity_check(db_path: &Path) -> Result<Vec<String>, AsyncStoreError> {
    block_on(store_async::integrity_check_async(db_path))
}
