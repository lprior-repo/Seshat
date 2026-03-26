use super::*;
#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;

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
    fn parse_args_returns_cli_when_minimum_boundary() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into()];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        assert_eq!(result, Cli::Bare);
        Ok(())
    }

    #[test]
    fn parse_args_returns_cli_with_help_flag_when_h_arg_passed() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "-h".into()];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        if let Cli::Help(help_str) = result {
            assert!(help_str.contains("Usage:"));
            Ok(())
        } else {
            Err("Expected Cli::Help".to_string())
        }
    }

    #[test]
    fn parse_args_returns_cli_with_version_flag_when_version_arg_passed() -> Result<(), String> {
        let args: Vec<OsString> = vec!["seshat".into(), "--version".into()];
        let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
        if let Cli::Version(version_str) = result {
            assert!(version_str.contains("seshat"));
            Ok(())
        } else {
            Err("Expected Cli::Version".to_string())
        }
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
