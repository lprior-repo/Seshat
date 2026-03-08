import os
import re

ALLOW_BLOCK = """#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::redundant_clone,
    clippy::bool_assert_comparison,
    clippy::module_inception,
    clippy::unnecessary_wraps,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::manual_midpoint,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::missing_const_for_fn,
    clippy::unnecessary_lazy_evaluations,
    clippy::trivially_copy_pass_by_ref,
    clippy::items_after_statements,
    clippy::ref_option,
    clippy::match_same_arms,
    clippy::ignored_unit_patterns,
    clippy::option_if_let_else,
    clippy::let_underscore_future,
    clippy::unused_async,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::collapsible_if,
    clippy::manual_try_fold
)]"""

for root, dirs, files in os.walk("diagram_tool"):
    for file in files:
        if not file.endswith(".rs"):
            continue
        path = os.path.join(root, file)
        with open(path, "r") as f:
            content = f.read()
            
        # Add to the top of the file if it's a test-only file (tests/ directory or *_tests.rs)
        if "tests/" in path or file.endswith("_tests.rs") or file == "io_tests.rs" or file == "subgraph_persistence_tests.rs" or file == "contracts.rs":
            if "#![allow(clippy::pedantic" not in content:
                content = ALLOW_BLOCK + "\n" + content
                with open(path, "w") as f:
                    f.write(content)
                continue
                
        # Add inside mod tests {
        new_content = re.sub(r'(mod\s+(?:tests|proptests|test_[a-zA-Z0-9_]+)\s*\{)', r'\1\n    ' + ALLOW_BLOCK.replace('\n', '\n    '), content)
        if new_content != content:
            with open(path, "w") as f:
                f.write(new_content)
