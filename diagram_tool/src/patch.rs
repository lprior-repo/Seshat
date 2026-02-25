#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use anyhow::Result;
use json_patch::Patch;

/// Pure calculation to apply an AI patch.
pub fn patch_doc(doc: &DiagramDocument, patch: &Patch) -> Result<DiagramDocument> {
    let mut doc_val = match serde_json::to_value(doc) {
        Ok(v) => v,
        Err(e) => return Err(anyhow::Error::new(e).context("Failed to serialize document")),
    };

    match json_patch::patch(&mut doc_val, patch) {
        Ok(()) => {}
        Err(e) => return Err(anyhow::Error::new(e).context("Failed to apply patch")),
    }

    match serde_json::from_value(doc_val) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow::Error::new(e).context("Failed to deserialize document")),
    }
}
