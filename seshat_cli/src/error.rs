/// Errors that can occur during execution of the `show` subcommand.
#[derive(Debug, PartialEq, Eq)]
pub enum ShowError {
    /// The specified file path does not exist or is not accessible.
    FileNotFound(std::path::PathBuf),
    /// An I/O error occurred while reading the file or stdin.
    IoError(String),
    /// The input bytes were not valid UTF-8.
    InvalidUtf8,
    /// The input was empty (zero bytes after trimming).
    EmptyInput,
    /// The input was not valid JSON.
    JsonDeserialize(String),
    /// The deserialized document failed `DiagramDocument` structural invariants.
    InvalidDocument(String),
    /// `serde_json` serialization of the loaded document failed.
    SerializationFailure(String),
    /// Writing the JSON to stdout failed.
    StdoutWriteFailure(String),
}

impl std::fmt::Display for ShowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => {
                write!(f, "error: show: file not found: {}", path.display())
            }
            Self::IoError(msg) => write!(f, "error: show: I/O error: {msg}"),
            Self::InvalidUtf8 => write!(f, "error: show: invalid utf-8 in input"),
            Self::EmptyInput => write!(f, "error: show: empty input"),
            Self::JsonDeserialize(msg) => write!(f, "error: show: JSON parse error: {msg}"),
            Self::InvalidDocument(msg) => write!(f, "error: show: invalid document: {msg}"),
            Self::SerializationFailure(msg) => {
                write!(f, "error: show: serialization failure: {msg}")
            }
            Self::StdoutWriteFailure(msg) => write!(f, "error: show: stdout write failure: {msg}"),
        }
    }
}

impl std::error::Error for ShowError {}

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
    Show(ShowError),
    Apply(crate::apply::ApplyCommandError),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SimulatedFailure => {
                write!(f, "Execution failed: Subcommand 'simulate_failure' aborted")
            }
            Self::Show(e) => write!(f, "{e}"),
            Self::Apply(e) => write!(f, "{e}"),
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
