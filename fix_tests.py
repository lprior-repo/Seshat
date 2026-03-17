import re

with open("canvas_domain/src/perf/tests.rs", "r") as f:
    content = f.read()

# Replace `use super::*;` with explicit imports since we are in `perf::tests::tests`
imports = """use crate::perf::*;
    use diagram_models::document::OrderedFloat;
    use crate::{CanvasCoord, ScreenCoord};
"""

content = content.replace("use super::*;", imports)

with open("canvas_domain/src/perf/tests.rs", "w") as f:
    f.write(content)
