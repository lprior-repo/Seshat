#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[path = "../graph/mod.rs"]
pub mod graph;

#[path = "../hooks/mod.rs"]
pub mod hooks;

#[path = "../ui/mod.rs"]
pub mod ui;

#[path = "../errors.rs"]
pub mod errors;

#[path = "../flow_extender.rs"]
pub mod flow_extender;

#[path = "../metrics.rs"]
pub mod metrics;

pub use metrics::MetricsStore;
