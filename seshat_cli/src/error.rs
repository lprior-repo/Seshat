/// Represents errors that occur during argument parsing.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    NoArguments,
    Clap(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoArguments => write!(f, "error: no arguments provided"),
            Self::Clap(msg) => write!(f, "{msg}"),
        }
    }
}

/// Represents errors that occur during command execution.
#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionError {
    SimulatedFailure,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SimulatedFailure => {
                write!(f, "Execution failed: Subcommand 'simulate_failure' aborted")
            }
        }
    }
}

/// Consolidated error type for the CLI.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    ArgumentParse(ParseError),
    CommandExecution(ExecutionError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgumentParse(e) => write!(f, "{e}"),
            Self::CommandExecution(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}
impl std::error::Error for ParseError {}
impl std::error::Error for ExecutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_returns_formatted_string_when_argument_parse_variant() {
        let err = Error::ArgumentParse(ParseError::Clap("foo error".to_string()));
        assert_eq!(err.to_string(), "foo error");
    }

    #[test]
    fn error_display_returns_formatted_string_when_command_execution_variant() {
        let err = Error::CommandExecution(ExecutionError::SimulatedFailure);
        assert_eq!(
            err.to_string(),
            "Execution failed: Subcommand 'simulate_failure' aborted"
        );
    }
}
