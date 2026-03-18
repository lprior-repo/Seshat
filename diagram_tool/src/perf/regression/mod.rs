//! Performance regression testing infrastructure.

mod report;
mod result;
mod test;

pub use report::{MachineInfo, PerformanceReport};
pub use result::RegressionResult;
pub use test::RegressionTest;
