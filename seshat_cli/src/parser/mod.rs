use crate::domain::{parse_depth, Cli, Depth, Subcommand};
use crate::error::{Error, ParseError};
use clap::{Parser, Subcommand as ClapSubcommandMacro};
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "seshat", version, disable_colored_help = true)]
struct ClapCli {
    #[command(subcommand)]
    subcommand: Option<ClapSubcommandEnum>,
}

#[derive(ClapSubcommandMacro, Clone)]
enum ClapSubcommandEnum {
    #[command(name = "valid-command")]
    ValidCommand,
    #[command(name = "simulate-failure")]
    SimulateFailure,
    #[command(name = "complex-state")]
    ComplexState {
        #[arg(long, allow_negative_numbers = true, value_parser = parse_depth)]
        depth: Depth,
    },
    #[command(name = "show")]
    Show {
        /// Path to the diagram file. If omitted, reads from stdin.
        #[arg(long, short = 'f')]
        file: Option<std::path::PathBuf>,
        /// Output as JSON (required flag for AI agent compatibility).
        #[arg(long, required = true)]
        json: bool,
    },
    #[command(name = "apply")]
    Apply {
        /// Path to the proposal JSON file. If omitted, reads from stdin.
        #[arg(long, short = 'f')]
        file: Option<std::path::PathBuf>,
        /// Path to the diagram document to validate against.
        #[arg(long, required = true)]
        doc: std::path::PathBuf,
    },
}

/// Parses an iterator of string arguments into a `Cli` command.
///
/// # Errors
/// Returns an `Error` if arguments are invalid or missing.
pub fn parse_args(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Cli, Error> {
    let args_vec: Vec<OsString> = args.collect();

    if args_vec.is_empty() || (args_vec.len() == 1 && args_vec[0].is_empty()) {
        return Err(Error::ArgumentParse(ParseError::NoArguments));
    }

    ClapCli::try_parse_from(args_vec).map_or_else(
        |e| map_clap_error(&e),
        |clap_cli| {
            Ok(clap_cli
                .subcommand
                .map_or(Cli::Bare, |cmd| Cli::Run(map_subcommand(cmd))))
        },
    )
}

fn map_clap_error(e: &clap::Error) -> Result<Cli, Error> {
    match e.kind() {
        clap::error::ErrorKind::DisplayHelp => Ok(Cli::Help(e.to_string())),
        clap::error::ErrorKind::DisplayVersion => Ok(Cli::Version(e.to_string())),
        _ => Err(Error::ArgumentParse(ParseError::Clap(e.to_string()))),
    }
}

fn map_subcommand(cmd: ClapSubcommandEnum) -> Subcommand {
    match cmd {
        ClapSubcommandEnum::ValidCommand => Subcommand::ValidCommand,
        ClapSubcommandEnum::SimulateFailure => Subcommand::SimulateFailure,
        ClapSubcommandEnum::ComplexState { depth } => Subcommand::ComplexState { depth },
        ClapSubcommandEnum::Show { file, json: _ } => {
            Subcommand::Show(crate::show::map_show_subcommand(file))
        }
        ClapSubcommandEnum::Apply { file, doc } => {
            Subcommand::Apply(crate::apply::map_apply_subcommand(file), doc)
        }
    }
}

#[must_use]
pub fn get_help() -> String {
    use clap::CommandFactory;
    ClapCli::command().render_help().to_string()
}

#[must_use]
pub fn get_version() -> String {
    use clap::CommandFactory;
    ClapCli::command().render_version()
}

#[cfg(test)]
mod proptests;
#[cfg(test)]
mod tests;
#[cfg(kani)]
mod verification;
