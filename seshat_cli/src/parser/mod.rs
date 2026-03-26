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
    #[command(name = "patch")]
    Patch {
        /// Path to the input diagram file. If omitted, reads from stdin.
        #[arg(long, short = 'i')]
        input: Option<std::path::PathBuf>,
        /// Path to the JSON patch file.
        #[arg(long, short = 'p')]
        patch: std::path::PathBuf,
        /// Path to the output diagram file. If omitted, writes to stdout.
        #[arg(long, short = 'o')]
        output: Option<std::path::PathBuf>,
    },
    #[command(name = "apply")]
    Apply {
        /// Path to the input diagram file. If omitted, reads from stdin.
        #[arg(long, short = 'i')]
        input: Option<std::path::PathBuf>,
        /// Path to the JSON proposal file.
        #[arg(long, short = 'p')]
        proposal: std::path::PathBuf,
    },
    #[command(name = "layout")]
    Layout {
        /// Path to the input diagram file.
        #[arg(long, short = 'i')]
        input: std::path::PathBuf,
        /// Path to the output diagram file.
        #[arg(long, short = 'o')]
        output: std::path::PathBuf,
    },
    #[command(name = "render")]
    Render {
        /// Path to the input diagram file. If omitted, reads from stdin.
        #[arg(long, short = 'i')]
        input: Option<std::path::PathBuf>,
        /// Path to the output image file (PNG or SVG).
        #[arg(long, short = 'o')]
        output: std::path::PathBuf,
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
        ClapSubcommandEnum::Patch {
            input,
            patch,
            output,
        } => Subcommand::Patch(crate::patch::map_patch_subcommand(input, patch, output)),
        ClapSubcommandEnum::Apply { input, proposal } => {
            Subcommand::Apply(crate::apply::map_apply_subcommand(input, proposal))
        }
        ClapSubcommandEnum::Layout { input, output } => {
            Subcommand::Layout(crate::domain::LayoutCommand { input, output })
        }
        ClapSubcommandEnum::Render { input, output } => {
            Subcommand::Render(crate::render::map_render_subcommand(input, output))
        }
    }
}

#[cfg(any(test, kani))]
mod tests;
