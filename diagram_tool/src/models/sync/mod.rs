//! Sync module - file-watch tail ingestion
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

use crate::models::envelope::parse_event_envelope;
use crate::models::projection::EventRecord;

#[derive(Debug, Error, Clone)]
pub enum SyncError {
    #[error("failed to initialize file watcher")] WatchInit,
    #[error("watcher runtime error")] WatchRuntime,
    #[error("I/O error: {0}")] Io(String),
    #[error("SQLite error: {0}")] Sqlite(String),
    #[error("failed to decode event: {0}")] Decode(String),
    #[error("channel closed")] ChannelClosed,
}

impl From<io::Error> for SyncError { fn from(err: io::Error) -> Self { SyncError::Io(err.to_string()) } }

#[cfg(not(target_arch = "wasm32"))]
pub struct WatcherHandle { watcher: notify::RecommendedWatcher, active: Arc<AtomicBool>, watch_path: PathBuf }
#[cfg(target_arch = "wasm32")]
pub struct WatcherHandle { active: Arc<AtomicBool> }

impl WatcherHandle { #[must_use] pub fn is_active(&self) -> bool { self.active.load(Ordering::SeqCst) } }

#[derive(Debug, Clone)]
pub enum SyncMessage { EventsUpdated(Vec<u64>), Error(String) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary { pub events_applied: usize, pub from_revision: u64, pub to_revision: u64, pub affected_entities: Vec<String> }

pub mod ops;
pub use ops::*;
