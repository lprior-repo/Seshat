#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use crate::models::document::DiagramDocument;
use crate::export::png::export_png;
use crate::export::svg::generate_svg_string;
use crate::patch::patch_doc;
use crate::layout::grid::calculate_grid_layout;
use crate::models::schema::validate_schema;
use std::fs::File;
use std::io::{BufReader, Write};
use anyhow::{Result, Context};
use json_patch::Patch;
use std::path::Path;

#[derive(Parser, Debug, Clone)]
#[command(name = "diagram_tool")]
#[command(version = "0.1.0")]
#[command(about = "Diagram Tool CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Render {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String
    },
    Patch {
        #[arg(long)]
        input: String,
        #[arg(long)]
        patch: String,
        #[arg(long)]
        output: String
    },
    Layout {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String
    },
    Validate {
        #[arg(long)]
        input: String
    },
}

pub fn run_cli(cli: &Cli) {
    if let Some(cmd) = &cli.command {
        if let Err(e) = execute_command(cmd) {
            eprintln!("Error: {e:#}");
            std::process::exit(1);
        }
    }
}

fn execute_command(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Render { input, output } => {
            let doc = load_doc(input)?;
            if Path::new(output).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("png")) {
                export_png(&doc, output)?;
            } else if Path::new(output).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("svg")) {
                let svg = generate_svg_string(&doc);
                let mut file = File::create(output).context("Failed to create SVG file")?;
                file.write_all(svg.as_bytes()).context("Failed to write SVG content")?;
            } else {
                eprintln!("Unknown output format. Use .png or .svg extension.");
            }
        },
        Commands::Patch { input, patch, output } => {
            let doc = load_doc(input)?;
            let patch_file = File::open(patch).context("Failed to open patch file")?;
            let patch_data: Patch = serde_json::from_reader(BufReader::new(patch_file)).context("Failed to parse patch JSON")?;
            let patched_doc = patch_doc(&doc, &patch_data)?;
            save_doc(&patched_doc, output)?;
        },
        Commands::Layout { input, output } => {
            let doc = load_doc(input)?;
            let laid_out_doc = calculate_grid_layout(&doc, 200.0);
            save_doc(&laid_out_doc, output)?;
        },
        Commands::Validate { input } => {
            let doc = load_doc(input)?;
            validate_schema(&doc)?;
            println!("Validation successful.");
        }
    }
    Ok(())
}

fn load_doc(path: &str) -> Result<DiagramDocument> {
    let file = File::open(path).with_context(|| format!("Failed to open input file: {path}"))?;
    let doc: DiagramDocument = serde_json::from_reader(BufReader::new(file)).with_context(|| format!("Failed to parse document from: {path}"))?;
    Ok(doc)
}

fn save_doc(doc: &DiagramDocument, path: &str) -> Result<()> {
    let file = File::create(path).with_context(|| format!("Failed to create output file: {path}"))?;
    serde_json::to_writer_pretty(file, doc).with_context(|| format!("Failed to write document to: {path}"))?;
    Ok(())
}
