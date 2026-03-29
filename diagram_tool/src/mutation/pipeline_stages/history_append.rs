#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]

use crate::mutation::pipeline::RevisionPolicy;
use diagram_models::document::DiagramDocument;

/// Append history by updating the revision.
#[must_use]
pub fn append_stage(
    next: DiagramDocument,
    current: &DiagramDocument,
    policy: RevisionPolicy,
) -> DiagramDocument {
    let revision = match policy {
        RevisionPolicy::Increment => current.revision.increment(),
        RevisionPolicy::Preserve => current.revision,
    };
    DiagramDocument { revision, ..next }
}
