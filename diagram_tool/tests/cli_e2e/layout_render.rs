use crate::common::E2eTest;
use std::fs;

#[cfg_attr(kani, kani::proof)]
#[test]
fn valid_document_layout_contains_nodes_and_edges() {
    let ctx = E2eTest::setup("layout");
    ctx.write_sample();

    let output_path = ctx.dir.join("layout.json");
    let res = ctx.run_diagram_tool(&[
        "layout",
        "--input",
        &ctx.input.to_string_lossy(),
        "--output",
        &output_path.to_string_lossy(),
    ]);

    assert!(res.success(), "layout command should succeed");

    let laid_out = fs::read_to_string(output_path).unwrap();
    assert!(
        laid_out.contains(r#""nodes""#),
        "output should contain nodes"
    );
    assert!(
        laid_out.contains(r#""edges""#),
        "output should contain edges"
    );
}

#[cfg_attr(kani, kani::proof)]
#[test]
fn valid_document_render_svg_is_generated() {
    let ctx = E2eTest::setup("render-svg");
    ctx.write_sample();

    let output_path = ctx.dir.join("diagram.svg");
    let res = ctx.run_diagram_tool(&[
        "render",
        "--input",
        &ctx.input.to_string_lossy(),
        "--output",
        &output_path.to_string_lossy(),
    ]);

    assert!(res.success(), "render svg command should succeed");

    let svg = fs::read_to_string(output_path).unwrap();
    assert!(svg.contains("<svg"), "svg output should contain svg root");
}
