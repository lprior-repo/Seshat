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

/// Errors that can occur during execution of the `patch` subcommand.
#[derive(Debug, PartialEq, Eq)]
pub enum PatchError {
    /// The specified file path does not exist or is not accessible.
    FileNotFound(std::path::PathBuf),
    /// An I/O error occurred while reading or writing.
    IoError(String),
    /// The input bytes were not valid UTF-8.
    InvalidUtf8,
    /// The input was empty.
    EmptyInput,
    /// JSON parsing failed.
    JsonDeserialize(String),
    /// Patch application failed.
    ApplyError(String),
    /// Missing a revision test in the patch.
    MissingRevisionTest,
    /// The output document is invalid.
    InvalidDocument(String),
    /// `serde_json` serialization of the modified document failed.
    SerializationFailure(String),
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => {
                write!(f, "error: patch: file not found: {}", path.display())
            }
            Self::IoError(msg) => write!(f, "error: patch: I/O error: {msg}"),
            Self::InvalidUtf8 => write!(f, "error: patch: invalid utf-8 in input"),
            Self::EmptyInput => write!(f, "error: patch: empty input"),
            Self::JsonDeserialize(msg) => write!(f, "error: patch: JSON parse error: {msg}"),
            Self::ApplyError(msg) => write!(f, "error: patch: failed to apply patch: {msg}"),
            Self::MissingRevisionTest => {
                write!(f, "error: patch: patch must include a test for /revision")
            }
            Self::InvalidDocument(msg) => {
                write!(f, "error: patch: invalid document after patch: {msg}")
            }
            Self::SerializationFailure(msg) => {
                write!(f, "error: patch: serialization failure: {msg}")
            }
        }
    }
}

impl std::error::Error for PatchError {}

/// Errors that can occur during execution of the `apply` subcommand.
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyError {
    /// The specified file path does not exist or is not accessible.
    FileNotFound(std::path::PathBuf),
    /// An I/O error occurred while reading or writing.
    IoError(String),
    /// The input bytes were not valid UTF-8.
    InvalidUtf8,
    /// The input was empty.
    EmptyInput,
    /// JSON parsing failed.
    JsonDeserialize(String),
    /// Proposal validation failed.
    InvalidProposal(String),
    /// The input document is invalid.
    InvalidDocument(String),
    /// `serde_json` serialization failed.
    SerializationFailure(String),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => {
                write!(f, "error: apply: file not found: {}", path.display())
            }
            Self::IoError(msg) => write!(f, "error: apply: I/O error: {msg}"),
            Self::InvalidUtf8 => write!(f, "error: apply: invalid utf-8 in input"),
            Self::EmptyInput => write!(f, "error: apply: empty input"),
            Self::JsonDeserialize(msg) => write!(f, "error: apply: JSON parse error: {msg}"),
            Self::InvalidProposal(msg) => write!(f, "error: apply: invalid proposal: {msg}"),
            Self::InvalidDocument(msg) => {
                write!(f, "error: apply: invalid document: {msg}")
            }
            Self::SerializationFailure(msg) => {
                write!(f, "error: apply: serialization failure: {msg}")
            }
        }
    }
}

impl std::error::Error for ApplyError {}

/// Errors that can occur during execution of the `layout` subcommand.
#[derive(Debug, PartialEq, Eq)]
pub enum LayoutError {
    LoadFailed(String),
    SaveFailed(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadFailed(msg) => write!(f, "error: layout: load failed: {msg}"),
            Self::SaveFailed(msg) => write!(f, "error: layout: save failed: {msg}"),
        }
    }
}

impl std::error::Error for LayoutError {}

/// Errors that can occur during execution of the `render` subcommand.
#[derive(Debug, PartialEq, Eq)]
pub enum RenderError {
    /// The specified file path does not exist or is not accessible.
    FileNotFound(std::path::PathBuf),
    /// An I/O error occurred while reading or writing.
    IoError(String),
    /// The input bytes were not valid UTF-8.
    InvalidUtf8,
    /// The input was empty.
    EmptyInput,
    /// JSON parsing failed.
    JsonDeserialize(String),
    /// Format not supported
    UnsupportedFormat(String),
    /// Export rendering failed
    ExportFailure(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => {
                write!(f, "error: render: file not found: {}", path.display())
            }
            Self::IoError(msg) => write!(f, "error: render: I/O error: {msg}"),
            Self::InvalidUtf8 => write!(f, "error: render: invalid utf-8 in input"),
            Self::EmptyInput => write!(f, "error: render: empty input"),
            Self::JsonDeserialize(msg) => write!(f, "error: render: JSON parse error: {msg}"),
            Self::UnsupportedFormat(ext) => {
                write!(f, "error: render: unsupported output format: {ext}")
            }
            Self::ExportFailure(msg) => write!(f, "error: render: export failure: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

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
    Patch(PatchError),
    Apply(ApplyError),
    Layout(LayoutError),
    Render(RenderError),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SimulatedFailure => {
                write!(f, "Execution failed: Subcommand 'simulate_failure' aborted")
            }
            Self::Show(e) => write!(f, "{e}"),
            Self::Patch(e) => write!(f, "{e}"),
            Self::Apply(e) => write!(f, "{e}"),
            Self::Layout(e) => write!(f, "{e}"),
            Self::Render(e) => write!(f, "{e}"),
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

    // ── ShowError Display variants ──────────────────────────────────

    #[test]
    fn show_error_display_file_not_found() {
        let err = ShowError::FileNotFound(std::path::PathBuf::from("/tmp/missing.txt"));
        assert_eq!(
            err.to_string(),
            "error: show: file not found: /tmp/missing.txt"
        );
    }

    #[test]
    fn show_error_display_io_error() {
        let err = ShowError::IoError("permission denied".to_string());
        assert_eq!(err.to_string(), "error: show: I/O error: permission denied");
    }

    #[test]
    fn show_error_display_invalid_utf8() {
        let err = ShowError::InvalidUtf8;
        assert_eq!(err.to_string(), "error: show: invalid utf-8 in input");
    }

    #[test]
    fn show_error_display_empty_input() {
        let err = ShowError::EmptyInput;
        assert_eq!(err.to_string(), "error: show: empty input");
    }

    #[test]
    fn show_error_display_json_deserialize() {
        let err = ShowError::JsonDeserialize("unexpected token".to_string());
        assert_eq!(
            err.to_string(),
            "error: show: JSON parse error: unexpected token"
        );
    }

    #[test]
    fn show_error_display_invalid_document() {
        let err = ShowError::InvalidDocument("missing nodes".to_string());
        assert_eq!(
            err.to_string(),
            "error: show: invalid document: missing nodes"
        );
    }

    #[test]
    fn show_error_display_serialization_failure() {
        let err = ShowError::SerializationFailure("key too deep".to_string());
        assert_eq!(
            err.to_string(),
            "error: show: serialization failure: key too deep"
        );
    }

    #[test]
    fn show_error_display_stdout_write_failure() {
        let err = ShowError::StdoutWriteFailure("broken pipe".to_string());
        assert_eq!(
            err.to_string(),
            "error: show: stdout write failure: broken pipe"
        );
    }

    // ── PatchError Display variants ─────────────────────────────────

    #[test]
    fn patch_error_display_file_not_found() {
        let err = PatchError::FileNotFound(std::path::PathBuf::from("/no/such/file.json"));
        assert_eq!(
            err.to_string(),
            "error: patch: file not found: /no/such/file.json"
        );
    }

    #[test]
    fn patch_error_display_io_error() {
        let err = PatchError::IoError("disk full".to_string());
        assert_eq!(err.to_string(), "error: patch: I/O error: disk full");
    }

    #[test]
    fn patch_error_display_invalid_utf8() {
        let err = PatchError::InvalidUtf8;
        assert_eq!(err.to_string(), "error: patch: invalid utf-8 in input");
    }

    #[test]
    fn patch_error_display_empty_input() {
        let err = PatchError::EmptyInput;
        assert_eq!(err.to_string(), "error: patch: empty input");
    }

    #[test]
    fn patch_error_display_json_deserialize() {
        let err = PatchError::JsonDeserialize("expected array".to_string());
        assert_eq!(
            err.to_string(),
            "error: patch: JSON parse error: expected array"
        );
    }

    #[test]
    fn patch_error_display_apply_error() {
        let err = PatchError::ApplyError("path /nodes/0/id not found".to_string());
        assert_eq!(
            err.to_string(),
            "error: patch: failed to apply patch: path /nodes/0/id not found"
        );
    }

    #[test]
    fn patch_error_display_missing_revision_test() {
        let err = PatchError::MissingRevisionTest;
        assert_eq!(
            err.to_string(),
            "error: patch: patch must include a test for /revision"
        );
    }

    #[test]
    fn patch_error_display_invalid_document() {
        let err = PatchError::InvalidDocument("duplicate id".to_string());
        assert_eq!(
            err.to_string(),
            "error: patch: invalid document after patch: duplicate id"
        );
    }

    #[test]
    fn patch_error_display_serialization_failure() {
        let err = PatchError::SerializationFailure("overflow".to_string());
        assert_eq!(
            err.to_string(),
            "error: patch: serialization failure: overflow"
        );
    }

    // ── ApplyError Display variants ─────────────────────────────────

    #[test]
    fn apply_error_display_file_not_found() {
        let err = ApplyError::FileNotFound(std::path::PathBuf::from("/absent.json"));
        assert_eq!(
            err.to_string(),
            "error: apply: file not found: /absent.json"
        );
    }

    #[test]
    fn apply_error_display_io_error() {
        let err = ApplyError::IoError("read failed".to_string());
        assert_eq!(err.to_string(), "error: apply: I/O error: read failed");
    }

    #[test]
    fn apply_error_display_invalid_utf8() {
        let err = ApplyError::InvalidUtf8;
        assert_eq!(err.to_string(), "error: apply: invalid utf-8 in input");
    }

    #[test]
    fn apply_error_display_empty_input() {
        let err = ApplyError::EmptyInput;
        assert_eq!(err.to_string(), "error: apply: empty input");
    }

    #[test]
    fn apply_error_display_json_deserialize() {
        let err = ApplyError::JsonDeserialize("trailing comma".to_string());
        assert_eq!(
            err.to_string(),
            "error: apply: JSON parse error: trailing comma"
        );
    }

    #[test]
    fn apply_error_display_invalid_proposal() {
        let err = ApplyError::InvalidProposal("missing target path".to_string());
        assert_eq!(
            err.to_string(),
            "error: apply: invalid proposal: missing target path"
        );
    }

    #[test]
    fn apply_error_display_invalid_document() {
        let err = ApplyError::InvalidDocument("no root".to_string());
        assert_eq!(err.to_string(), "error: apply: invalid document: no root");
    }

    #[test]
    fn apply_error_display_serialization_failure() {
        let err = ApplyError::SerializationFailure("recursion limit".to_string());
        assert_eq!(
            err.to_string(),
            "error: apply: serialization failure: recursion limit"
        );
    }

    // ── LayoutError Display variants ────────────────────────────────

    #[test]
    fn layout_error_display_load_failed() {
        let err = LayoutError::LoadFailed("bad json".to_string());
        assert_eq!(err.to_string(), "error: layout: load failed: bad json");
    }

    #[test]
    fn layout_error_display_save_failed() {
        let err = LayoutError::SaveFailed("write refused".to_string());
        assert_eq!(err.to_string(), "error: layout: save failed: write refused");
    }

    // ── RenderError Display variants ────────────────────────────────

    #[test]
    fn render_error_display_file_not_found() {
        let err = RenderError::FileNotFound(std::path::PathBuf::from("/img/out.svg"));
        assert_eq!(
            err.to_string(),
            "error: render: file not found: /img/out.svg"
        );
    }

    #[test]
    fn render_error_display_io_error() {
        let err = RenderError::IoError("cannot open".to_string());
        assert_eq!(err.to_string(), "error: render: I/O error: cannot open");
    }

    #[test]
    fn render_error_display_invalid_utf8() {
        let err = RenderError::InvalidUtf8;
        assert_eq!(err.to_string(), "error: render: invalid utf-8 in input");
    }

    #[test]
    fn render_error_display_empty_input() {
        let err = RenderError::EmptyInput;
        assert_eq!(err.to_string(), "error: render: empty input");
    }

    #[test]
    fn render_error_display_json_deserialize() {
        let err = RenderError::JsonDeserialize("eof".to_string());
        assert_eq!(err.to_string(), "error: render: JSON parse error: eof");
    }

    #[test]
    fn render_error_display_unsupported_format() {
        let err = RenderError::UnsupportedFormat("bmp".to_string());
        assert_eq!(
            err.to_string(),
            "error: render: unsupported output format: bmp"
        );
    }

    #[test]
    fn render_error_display_export_failure() {
        let err = RenderError::ExportFailure("canvas too large".to_string());
        assert_eq!(
            err.to_string(),
            "error: render: export failure: canvas too large"
        );
    }

    // ── ParseError Display variants ─────────────────────────────────

    #[test]
    fn parse_error_display_no_arguments() {
        let err = ParseError::NoArguments;
        assert_eq!(err.to_string(), "error: no arguments provided");
    }

    // ── ExecutionError delegation variants ──────────────────────────

    #[test]
    fn execution_error_display_delegates_show() {
        let err = ExecutionError::Show(ShowError::EmptyInput);
        assert_eq!(err.to_string(), "error: show: empty input");
    }

    #[test]
    fn execution_error_display_delegates_patch() {
        let err = ExecutionError::Patch(PatchError::MissingRevisionTest);
        assert_eq!(
            err.to_string(),
            "error: patch: patch must include a test for /revision"
        );
    }

    #[test]
    fn execution_error_display_delegates_apply() {
        let err = ExecutionError::Apply(ApplyError::InvalidUtf8);
        assert_eq!(err.to_string(), "error: apply: invalid utf-8 in input");
    }

    #[test]
    fn execution_error_display_delegates_layout() {
        let err = ExecutionError::Layout(LayoutError::LoadFailed("oops".to_string()));
        assert_eq!(err.to_string(), "error: layout: load failed: oops");
    }

    #[test]
    fn execution_error_display_delegates_render() {
        let err = ExecutionError::Render(RenderError::UnsupportedFormat("tiff".to_string()));
        assert_eq!(
            err.to_string(),
            "error: render: unsupported output format: tiff"
        );
    }

    // ── Error top-level variants ────────────────────────────────────

    #[test]
    fn error_display_no_arguments() {
        let err = Error::ArgumentParse(ParseError::NoArguments);
        assert_eq!(err.to_string(), "error: no arguments provided");
    }

    #[test]
    fn error_display_delegates_execution_show() {
        let err = Error::CommandExecution(ExecutionError::Show(ShowError::IoError("err".into())));
        assert_eq!(err.to_string(), "error: show: I/O error: err");
    }

    // ── std::error::Error trait bound verification ──────────────────

    #[test]
    fn show_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ShowError::EmptyInput);
        assert_eq!(err.to_string(), "error: show: empty input");
    }

    #[test]
    fn patch_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(PatchError::EmptyInput);
        assert_eq!(err.to_string(), "error: patch: empty input");
    }

    #[test]
    fn apply_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ApplyError::EmptyInput);
        assert_eq!(err.to_string(), "error: apply: empty input");
    }

    #[test]
    fn layout_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(LayoutError::LoadFailed("x".into()));
        assert_eq!(err.to_string(), "error: layout: load failed: x");
    }

    #[test]
    fn render_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(RenderError::EmptyInput);
        assert_eq!(err.to_string(), "error: render: empty input");
    }

    #[test]
    fn parse_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ParseError::NoArguments);
        assert_eq!(err.to_string(), "error: no arguments provided");
    }

    #[test]
    fn execution_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ExecutionError::SimulatedFailure);
        assert_eq!(
            err.to_string(),
            "Execution failed: Subcommand 'simulate_failure' aborted"
        );
    }

    #[test]
    fn error_implements_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(Error::ArgumentParse(ParseError::NoArguments));
        assert_eq!(err.to_string(), "error: no arguments provided");
    }
}
