#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::apply::ApplyCommand;
use super::export::ExportCommand;
use super::import::ImportCommand;
use super::layout::LayoutCommand;
use super::patch::PatchCommand;
use super::render::RenderCommand;
use super::validate::ValidateCommand;

#[derive(Parser, Debug)]
#[command(name = "seshat")]
#[command(about = "Seshat diagram tool CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Validate a diagram document")]
    Validate(ValidateArgs),

    #[command(about = "Apply changes to a diagram")]
    Apply(ApplyArgs),

    #[command(about = "Apply a JSON patch to a diagram")]
    Patch(PatchArgs),

    #[command(about = "Render diagram to PNG or SVG")]
    Render(RenderArgs),

    #[command(about = "Auto-arrange nodes using DAG layout")]
    Layout(LayoutArgs),

    #[command(about = "Export diagram to JSON")]
    Export(ExportArgs),

    #[command(about = "Import diagram from JSON")]
    Import(ImportArgs),
}

#[derive(Parser, Debug)]
pub struct ValidateArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ApplyArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

#[derive(Parser, Debug)]
pub struct PatchArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub patch: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct RenderArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct LayoutArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    #[arg(long, value_name = "FORMAT", default_value = "json")]
    pub format: String,
}

#[derive(Parser, Debug)]
pub struct ImportArgs {
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,
}

pub trait Command {
    fn name(&self) -> &'static str;
    fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl Commands {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Commands::Validate(args) => ValidateCommand::new(args.input.clone()).execute(),
            Commands::Apply(args) => ApplyCommand::new(args.input.clone()).execute(),
            Commands::Patch(args) => {
                PatchCommand::new(args.input.clone(), args.patch.clone(), args.output.clone())
                    .execute()
            }
            Commands::Render(args) => {
                RenderCommand::new(args.input.clone(), args.output.clone()).execute()
            }
            Commands::Layout(args) => {
                LayoutCommand::new(args.input.clone(), args.output.clone()).execute()
            }
            Commands::Export(args) => {
                ExportCommand::new(args.input.clone(), args.format.clone()).execute()
            }
            Commands::Import(args) => ImportCommand::new(args.input.clone()).execute(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Commands::Validate(_) => "validate",
            Commands::Apply(_) => "apply",
            Commands::Patch(_) => "patch",
            Commands::Render(_) => "render",
            Commands::Layout(_) => "layout",
            Commands::Export(_) => "export",
            Commands::Import(_) => "import",
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::cli::commands::Command;
    use diagram_models::document::{DiagramDocument, DocumentData, EditorState, Revision};
    use diagram_models::physical_io;
    use im::HashMap;
    use tempfile::tempdir;

    fn create_test_doc() -> DiagramDocument {
        DiagramDocument {
            version: 2,
            revision: Revision::INITIAL,
            document: DocumentData {
                nodes: HashMap::new(),
                edges: HashMap::new(),
            },
            editor_state: EditorState::default(),
        }
    }

    // --- Commands::name() for each variant ---

    #[test]
    fn commands_name_validate() {
        let cmd = Commands::Validate(ValidateArgs {
            input: std::path::PathBuf::from("test.json"),
        });
        assert_eq!(cmd.name(), "validate");
    }

    #[test]
    fn commands_name_apply() {
        let cmd = Commands::Apply(ApplyArgs {
            input: std::path::PathBuf::from("test.json"),
        });
        assert_eq!(cmd.name(), "apply");
    }

    #[test]
    fn commands_name_patch() {
        let cmd = Commands::Patch(PatchArgs {
            input: std::path::PathBuf::from("in.json"),
            patch: std::path::PathBuf::from("patch.json"),
            output: std::path::PathBuf::from("out.json"),
        });
        assert_eq!(cmd.name(), "patch");
    }

    #[test]
    fn commands_name_render() {
        let cmd = Commands::Render(RenderArgs {
            input: std::path::PathBuf::from("test.json"),
            output: std::path::PathBuf::from("out.svg"),
        });
        assert_eq!(cmd.name(), "render");
    }

    #[test]
    fn commands_name_layout() {
        let cmd = Commands::Layout(LayoutArgs {
            input: std::path::PathBuf::from("test.json"),
            output: std::path::PathBuf::from("out.json"),
        });
        assert_eq!(cmd.name(), "layout");
    }

    #[test]
    fn commands_name_export() {
        let cmd = Commands::Export(ExportArgs {
            input: std::path::PathBuf::from("test.json"),
            format: "json".to_string(),
        });
        assert_eq!(cmd.name(), "export");
    }

    #[test]
    fn commands_name_import() {
        let cmd = Commands::Import(ImportArgs {
            input: std::path::PathBuf::from("test.json"),
        });
        assert_eq!(cmd.name(), "import");
    }

    // --- Commands::execute() ---

    #[test]
    fn commands_execute_validate_succeeds() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("valid.json");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = Commands::Validate(ValidateArgs { input: input_path });
        let result = cmd.execute();
        assert!(result.is_ok(), "Validate command should succeed");
    }

    #[test]
    fn commands_execute_render_succeeds() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let output_path = dir.path().join("output.svg");

        let doc = create_test_doc();
        physical_io::save_document(&input_path, &doc).unwrap();

        let cmd = Commands::Render(RenderArgs {
            input: input_path,
            output: output_path.clone(),
        });
        let result = cmd.execute();
        assert!(result.is_ok(), "Render command should succeed");
        assert!(output_path.exists(), "SVG output should be created");
    }

    // --- Clap parsing ---

    #[test]
    fn clap_parses_validate_command() {
        let cli = Cli::try_parse_from(["seshat", "validate", "-i", "/tmp/test.json"]);
        assert!(cli.is_ok(), "should parse validate command");

        let cli = cli.unwrap();
        assert!(cli.command.is_some());

        match cli.command.unwrap() {
            Commands::Validate(args) => {
                assert_eq!(args.input, std::path::PathBuf::from("/tmp/test.json"));
            }
            other => panic!("expected Validate, got: {other:?}"),
        }
    }

    #[test]
    fn clap_parses_render_command() {
        let cli = Cli::try_parse_from([
            "seshat",
            "render",
            "-i",
            "/tmp/input.json",
            "-o",
            "/tmp/output.svg",
        ]);
        assert!(cli.is_ok(), "should parse render command");

        let cli = cli.unwrap();
        match cli.command.unwrap() {
            Commands::Render(args) => {
                assert_eq!(args.input, std::path::PathBuf::from("/tmp/input.json"));
                assert_eq!(args.output, std::path::PathBuf::from("/tmp/output.svg"));
            }
            other => panic!("expected Render, got: {other:?}"),
        }
    }

    #[test]
    fn clap_parses_export_command() {
        let cli = Cli::try_parse_from([
            "seshat",
            "export",
            "-i",
            "/tmp/test.json",
            "--format",
            "json",
        ]);
        assert!(cli.is_ok(), "should parse export command");

        let cli = cli.unwrap();
        match cli.command.unwrap() {
            Commands::Export(args) => {
                assert_eq!(args.input, std::path::PathBuf::from("/tmp/test.json"));
                assert_eq!(args.format, "json");
            }
            other => panic!("expected Export, got: {other:?}"),
        }
    }

    #[test]
    fn clap_parses_no_subcommand() {
        let cli = Cli::try_parse_from(["seshat"]);
        assert!(cli.is_ok(), "should parse with no subcommand");

        let cli = cli.unwrap();
        assert!(cli.command.is_none(), "no subcommand should yield None");
    }
}
