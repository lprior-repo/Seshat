use std::num::ParseIntError;
use std::path::PathBuf;

/// Source for the show command — either a filesystem path or stdin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShowSource {
    /// Read from the filesystem at the given path.
    File(PathBuf),
    /// Read from stdin.
    Stdin,
}

/// Domain command for the show subcommand. Data-only: no I/O handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowCommand {
    pub source: ShowSource,
}

/// Source for the patch command input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchSource {
    /// Read input from the given path.
    File(PathBuf),
    /// Read input from stdin.
    Stdin,
}

/// Target for the patch command output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchTarget {
    /// Write output to the given path.
    File(PathBuf),
    /// Write output to stdout.
    Stdout,
}

/// Domain command for the patch subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchCommand {
    pub input: PatchSource,
    pub patch: PathBuf,
    pub output: PatchTarget,
}

/// Source for the apply command input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplySource {
    /// Read input from the given path.
    File(PathBuf),
    /// Read input from stdin.
    Stdin,
}

/// Domain command for the apply subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyCommand {
    pub document: ApplySource,
    pub proposal: PathBuf,
}

/// Source for the render command input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderSource {
    /// Read input from the given path.
    File(PathBuf),
    /// Read input from stdin.
    Stdin,
}

/// Domain command for the render subcommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCommand {
    pub input: RenderSource,
    pub output: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DepthError {
    Negative,
    ExceedsMax,
}

impl std::fmt::Display for DepthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negative => write!(f, "depth cannot be negative"),
            Self::ExceedsMax => write!(f, "max nesting depth exceeded"),
        }
    }
}

impl std::error::Error for DepthError {}

/// Depth level for the complex state subcommand.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Depth(i32);

impl Depth {
    /// Attempts to create a new `Depth`, validating boundaries.
    ///
    /// # Errors
    /// Returns an error if the depth is negative or exceeds the maximum (254).
    pub const fn try_new(val: i32) -> Result<Self, DepthError> {
        match val {
            v if v < 0 => Err(DepthError::Negative),
            v if v > 254 => Err(DepthError::ExceedsMax),
            v => Ok(Self(v)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCommand {
    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Subcommand {
    ValidCommand,
    SimulateFailure,
    ComplexState { depth: Depth },
    Show(ShowCommand),
    Patch(PatchCommand),
    Apply(ApplyCommand),
    Layout(LayoutCommand),
    Render(RenderCommand),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Cli {
    Run(Subcommand),
    Help(String),
    Version(String),
    Bare,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseDepthError {
    ParseInt(String),
    Domain(DepthError),
}

impl std::fmt::Display for ParseDepthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseInt(e) => write!(f, "{e}"),
            Self::Domain(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ParseDepthError {}

/// Parses a string into a `Depth`.
///
/// # Errors
/// Returns an error if the string is not a valid integer, or if it violates `Depth` limits.
pub fn parse_depth(s: &str) -> Result<Depth, ParseDepthError> {
    let val: i32 = s
        .parse()
        .map_err(|e: ParseIntError| ParseDepthError::ParseInt(e.to_string()))?;
    Depth::try_new(val).map_err(ParseDepthError::Domain)
}
