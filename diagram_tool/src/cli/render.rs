use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::Write, path::Path};

use crate::cli::common::load_doc;
use crate::cli_persistence::validate_safe_path;
use crate::export::{png::export_png, svg::generate_svg_string};

pub fn handle(input: &str, output: &str) -> Result<()> {
    let doc = load_doc(input)?;

    let output_path = Path::new(output);
    let output_parent = output_path.parent().filter(|p| !p.as_os_str().is_empty());
    let output_base_dir = output_parent.unwrap_or_else(|| Path::new("."));
    validate_safe_path(output_path, output_base_dir)
        .map_err(|e| anyhow!("Invalid output path: {e}"))?;

    if Path::new(output)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
    {
        export_png(&doc, output)?;
    } else if Path::new(output)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
    {
        let svg = generate_svg_string(&doc);
        let mut file = File::create(output).context("Failed to create SVG file")?;
        file.write_all(svg.as_bytes())
            .context("Failed to write SVG content")?;
    } else {
        return Err(anyhow!(
            "unknown output format; expected .png or .svg extension"
        ));
    }
    Ok(())
}
