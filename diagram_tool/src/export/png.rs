#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::models::document::DiagramDocument;
use crate::export::svg::generate_svg_string;
use tiny_skia::{Pixmap, Transform};
use resvg::usvg; 
use anyhow::{Result, Context};

/// Export document to PNG file.
pub fn export_png(doc: &DiagramDocument, path: &str) -> Result<()> {
    let svg_data = generate_svg_string(doc);
    
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_str(&svg_data, &opt).context("Failed to parse SVG")?;
    
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).context("Failed to create pixmap")?;
    
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
    
    pixmap.save_png(path).context("Failed to save PNG")?;
    Ok(())
}
