use crate::domain::{parse_depth, Cli, Depth, Subcommand};
use crate::error::{Error, ParseError};
use clap::{Parser, Subcommand as ClapSubcommandMacro};
use std::ffi::OsString;

#[derive(Parser)]
#[command(
    name = "seshat",
    version,
    disable_colored_help = true,
    arg_required_else_help = true
)]
struct ClapCli {
    #[command(subcommand)]
    subcommand: Option<ClapSubcommandEnum>,
}

#[derive(ClapSubcommandMacro)]
enum ClapSubcommandEnum {
    #[command(name = "valid-command", hide = true)]
    ValidCommand,
    #[command(name = "simulate-failure", hide = true)]
    SimulateFailure,
    #[command(name = "complex-state", hide = true)]
    ComplexState {
        #[arg(long, allow_negative_numbers = true, value_parser = parse_depth)]
        depth: Depth,
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
            if let Some(cmd) = clap_cli.subcommand {
                Ok(Cli::Run(map_subcommand(&cmd)))
            } else {
                Ok(Cli::Bare)
            }
        },
    )
}

fn map_clap_error(e: &clap::Error) -> Result<Cli, Error> {
    match e.kind() {
        clap::error::ErrorKind::DisplayHelp => Ok(Cli::Help),
        clap::error::ErrorKind::DisplayVersion => Ok(Cli::Version),
        _ => Err(Error::ArgumentParse(ParseError::Clap(e.to_string()))),
    }
}

fn map_subcommand(cmd: &ClapSubcommandEnum) -> Subcommand {
    match cmd {
        ClapSubcommandEnum::ValidCommand => Subcommand::ValidCommand,
        ClapSubcommandEnum::SimulateFailure => Subcommand::SimulateFailure,
        ClapSubcommandEnum::ComplexState { depth } => Subcommand::ComplexState { depth: *depth },
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
mod tests {
    use super::*;

    #[test]
    fn get_version_returns_version_string_when_called() {
        let version = get_version();
        assert_eq!(version, "seshat 0.1.0\n");
    }

    #[test]
    fn get_help_returns_usage_string_when_called() {
        let help = get_help();
        assert_eq!(&help[0..6], "Usage:");
    }

    #[test]
    fn parse_args_returns_cli_with_complex_state_when_valid_depth_provided() -> Result<(), String> {
        let args: Vec<OsString> = vec![
            "seshat".into(),
            "complex-state".into(),
            "--depth".into(),
            "42".into(),
        ];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(
            result,
            Cli::Run(Subcommand::ComplexState {
                depth: Depth::try_new(42).map_err(|e| e.to_string())?
            })
        );
        Ok(())
    }

    #[test]
    fn parse_args_returns_error_when_one_below_minimum_boundary() {
        let args: Vec<OsString> = vec![];
        let result = parse_args(args.into_iter());
        assert_eq!(result, Err(Error::ArgumentParse(ParseError::NoArguments)));
    }

    #[test]
    fn parse_args_returns_error_when_minimum_boundary() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into()];
        let result = parse_args(args.into_iter());
        if let Err(Error::ArgumentParse(ParseError::Clap(msg))) = result {
            assert!(msg.contains("Usage: seshat"));
            Ok(())
        } else {
            Err("Expected clap error".to_string())
        }
    }

    #[test]
    fn parse_args_returns_cli_with_help_flag_when_h_arg_passed() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "-h".into()];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(result, Cli::Help);
        Ok(())
    }

    #[test]
    fn parse_args_returns_cli_with_version_flag_when_version_arg_passed() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "--version".into()];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(result, Cli::Version);
        Ok(())
    }

    #[test]
    fn parse_args_returns_cli_when_valid_subcommand_provided() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "valid-command".into()];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(result, Cli::Run(Subcommand::ValidCommand));
        Ok(())
    }

    #[test]
    fn parse_args_returns_error_when_unknown_subcommand_provided() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "unrecognized-cmd".into()];
        let result = parse_args(args.into_iter());
        if let Err(Error::ArgumentParse(ParseError::Clap(msg))) = result {
            assert!(msg.contains("unrecognized subcommand 'unrecognized-cmd'"));
            Ok(())
        } else {
            Err("Expected clap error".to_string())
        }
    }

    #[test]
    fn parse_args_returns_error_when_unknown_flag_provided() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "--unknown-flag".into()];
        let result = parse_args(args.into_iter());
        if let Err(Error::ArgumentParse(ParseError::Clap(msg))) = result {
            assert!(msg.contains("unexpected argument '--unknown-flag' found"));
            Ok(())
        } else {
            Err("Expected clap error".to_string())
        }
    }

    #[test]
    fn parse_args_returns_error_when_underflow_potential_boundary() -> Result<(), String> {
        let args: Vec<OsString> = vec![
            "seshat".into(),
            "complex-state".into(),
            "--depth".into(),
            "-2147483649".into(),
        ];
        let result = parse_args(args.into_iter());
        if let Err(Error::ArgumentParse(ParseError::Clap(msg))) = result {
            assert!(msg.contains("invalid value '-2147483649' for '--depth <DEPTH>':"));
            Ok(())
        } else {
            Err("Expected clap error".to_string())
        }
    }

    #[test]
    fn parse_args_returns_error_when_one_below_minimum_boundary_depth() -> Result<(), String> {
        let args: Vec<OsString> = vec![
            "seshat".into(),
            "complex-state".into(),
            "--depth".into(),
            "-1".into(),
        ];
        let result = parse_args(args.into_iter());
        if let Err(Error::ArgumentParse(ParseError::Clap(msg))) = result {
            assert!(msg.contains("depth cannot be negative"));
            Ok(())
        } else {
            Err("Expected clap error".to_string())
        }
    }

    #[test]
    fn parse_args_returns_error_when_overflow_one_above_max_boundary_depth() -> Result<(), String> {
        let args: Vec<OsString> = vec![
            "seshat".into(),
            "complex-state".into(),
            "--depth".into(),
            "255".into(),
        ];
        let result = parse_args(args.into_iter());
        if let Err(Error::ArgumentParse(ParseError::Clap(msg))) = result {
            assert!(msg.contains("max nesting depth exceeded"));
            Ok(())
        } else {
            Err("Expected clap error".to_string())
        }
    }

    #[test]
    fn parse_args_returns_success_when_depth_is_zero_minimum_boundary() -> Result<(), String> {
        let args: Vec<OsString> = vec![
            "seshat".into(),
            "complex-state".into(),
            "--depth".into(),
            "0".into(),
        ];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(
            result,
            Cli::Run(Subcommand::ComplexState {
                depth: Depth::try_new(0).map_err(|e| e.to_string())?
            })
        );
        Ok(())
    }

    #[test]
    fn parse_args_returns_success_when_depth_is_254_maximum_boundary() -> Result<(), String> {
        let args: Vec<OsString> = vec![
            "seshat".into(),
            "complex-state".into(),
            "--depth".into(),
            "254".into(),
        ];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(
            result,
            Cli::Run(Subcommand::ComplexState {
                depth: Depth::try_new(254).map_err(|e| e.to_string())?
            })
        );
        Ok(())
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_args_never_panics(
            args in prop::collection::vec(any::<String>(), 0..100)
        ) {
            match parse_args(args.into_iter().map(|s: String| OsString::from(s))) {
                Ok(_) => prop_assert!(true),
                Err(e) => prop_assert!(matches!(e, Error::ArgumentParse(_) | Error::CommandExecution(_))),
            }
        }
    }
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    #[test]
    fn no_panic_in_parse_args() {
        let args = vec![OsString::from("arg1"), OsString::from("arg2")];
        match parse_args(args.into_iter()) {
            Ok(_) => assert!(true),
            Err(e) => assert!(matches!(
                e,
                Error::ArgumentParse(_) | Error::CommandExecution(_)
            )),
        }
    }
}
