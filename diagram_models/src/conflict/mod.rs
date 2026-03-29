//! Conflict types
#![allow(dead_code)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use thiserror::Error;

const HUMAN_EDIT_WINDOW_SECS: u64 = 30;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictError {
    #[error("human priority block: {0}")]
    HumanPriorityBlock(String),
    #[error("missing entity: {0}")]
    MissingEntity(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictDecision {
    Allow,
    Reject {
        reason: ConflictError,
        conflicting_entities: Vec<String>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ProjectionState {
    human_edit_windows: im::HashMap<String, HumanEditWindow>,
    processed_ops: HashSet<String>,
}

#[derive(Debug, Clone)]
struct HumanEditWindow {
    entity_id: String,
    last_edit_time: Instant,
    author_id: String,
}

impl HumanEditWindow {
    fn new(entity_id: String, author_id: String) -> Self {
        Self {
            entity_id,
            last_edit_time: Instant::now(),
            author_id,
        }
    }
    fn is_active(&self) -> bool {
        Instant::now().duration_since(self.last_edit_time)
            < Duration::from_secs(HUMAN_EDIT_WINDOW_SECS)
    }
    fn refresh(&mut self) {
        self.last_edit_time = Instant::now();
    }
}

impl ProjectionState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register_human_edit(&mut self, entity_id: &str, author_id: &str) {
        if let Some(window) = self.human_edit_windows.get_mut(entity_id) {
            window.refresh();
        } else {
            self.human_edit_windows.insert(
                entity_id.to_string(),
                HumanEditWindow::new(entity_id.to_string(), author_id.to_string()),
            );
        }
    }
    #[must_use]
    pub fn has_active_human_edit(&self, entity_id: &str) -> bool {
        self.human_edit_windows
            .get(entity_id)
            .is_some_and(|w| w.is_active())
    }
    #[must_use]
    pub fn active_human_edit_entities(&self) -> Vec<String> {
        self.human_edit_windows
            .iter()
            .filter(|(_, w)| w.is_active())
            .map(|(id, _)| id.clone())
            .collect()
    }
    pub fn cleanup_expired(&mut self) {
        self.human_edit_windows.retain(|_, w| w.is_active());
    }
    pub fn mark_processed(&mut self, op_id: &str) {
        self.processed_ops.insert(op_id.to_string());
    }
    #[must_use]
    pub fn is_processed(&self, op_id: &str) -> bool {
        self.processed_ops.contains(op_id)
    }
}

pub mod resolution;
pub use resolution::*;
