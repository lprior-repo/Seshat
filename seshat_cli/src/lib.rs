#![allow(unexpected_cfgs)]

pub mod domain;
pub mod error;
pub mod executor;
pub mod parser;

// Re-export public types that were originally in the root
pub use domain::{parse_depth, Cli, Depth, DepthError, ParseDepthError, Subcommand};
pub use error::{Error, ExecutionError, ParseError};
pub use executor::execute;
pub use parser::{get_help, get_version, parse_args};
