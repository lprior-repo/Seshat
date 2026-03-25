#![allow(unexpected_cfgs)]

pub mod domain;
pub mod error;
pub mod executor;
pub mod parser;
pub mod show;

// Re-export public types that were originally in the root
pub use domain::{
    parse_depth, Cli, Depth, DepthError, ParseDepthError, ShowCommand, ShowSource, Subcommand,
};
pub use error::{Error, ExecutionError, ParseError, ShowError};
pub use executor::execute;
pub use parser::parse_args;
pub use show::{
    execute_show, load_document_from_path, load_document_from_reader, serialize_document,
};
