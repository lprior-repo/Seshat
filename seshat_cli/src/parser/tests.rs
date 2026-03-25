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

// =========================================================================
// BEHAVIOR 76: parse_args returns Apply subcommand when "apply" is first arg
// =========================================================================

#[test]
fn parse_args_returns_apply_subcommand_when_apply_is_first_arg() -> Result<(), String> {
    let args: Vec<OsString> = vec![
        "seshat".into(),
        "apply".into(),
        "--file".into(),
        "/tmp/p.json".into(),
        "--doc".into(),
        "/tmp/doc.json".into(),
    ];
    let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
    match result {
        Cli::Run(Subcommand::Apply(cmd, doc_path)) => {
            assert_eq!(
                cmd.input_source,
                crate::apply::ApplySource::File(std::path::PathBuf::from("/tmp/p.json"))
            );
            assert_eq!(doc_path, std::path::PathBuf::from("/tmp/doc.json"));
        }
        other => return Err(format!("expected Subcommand::Apply, got: {other:?}")),
    }
    Ok(())
}

// =========================================================================
// BEHAVIOR 77: parse_args returns Apply with file source when --file provided
// =========================================================================

#[test]
fn parse_args_returns_apply_with_file_source_when_file_provided() -> Result<(), String> {
    let args: Vec<OsString> = vec![
        "seshat".into(),
        "apply".into(),
        "--file".into(),
        "/some/proposal.json".into(),
        "--doc".into(),
        "/some/doc.json".into(),
    ];
    let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
    match result {
        Cli::Run(Subcommand::Apply(cmd, _)) => {
            assert!(
                matches!(cmd.input_source, crate::apply::ApplySource::File(_)),
                "expected File source, got: {:?}",
                cmd.input_source
            );
        }
        other => return Err(format!("expected Subcommand::Apply, got: {other:?}")),
    }
    Ok(())
}

// =========================================================================
// BEHAVIOR 78: parse_args returns Apply with stdin source when --file omitted
// =========================================================================

#[test]
fn parse_args_returns_apply_with_stdin_source_when_file_omitted() -> Result<(), String> {
    let args: Vec<OsString> = vec![
        "seshat".into(),
        "apply".into(),
        "--doc".into(),
        "/some/doc.json".into(),
    ];
    let result = parse_args(args.into_iter()).map_err(|e| e.to_string())?;
    match result {
        Cli::Run(Subcommand::Apply(cmd, _)) => {
            assert_eq!(
                cmd.input_source,
                crate::apply::ApplySource::Stdin,
                "expected Stdin source when --file is omitted"
            );
        }
        other => return Err(format!("expected Subcommand::Apply, got: {other:?}")),
    }
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
