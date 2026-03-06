import re

with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "r") as f:
    content = f.read()

# I want to remove the specific duplicate proptests that are already in math.rs
# I'll look for them by name and use regex to remove the entire `proptest! { ... }` block
# or `#[test] fn ...` block.

tests_to_remove = [
    "prop_safe_zoom_rejects_extreme_values",
    "prop_within_handles_nan_subgraph_coords",
    "prop_within_handles_nan_node_coords",
    "prop_within_degenerate_rectangles",
    "prop_within_infinite_dims",
    "prop_within_exact_boundary",
    "prop_within_node_on_edge",
    "prop_within_exceeds_by_epsilon",
    "prop_safe_zoom_boundary",
    "prop_overflow_safety",
    "prop_subnormal_floats",
]

for test_name in tests_to_remove:
    # A typical block is:
    #     proptest! {
    #         ...
    #         fn prop_...(...) {
    #             ...
    #         }
    #     }
    # We can match `proptest! {` down to the closing `}` if we balance braces, or just use a regex if the structure is simple.
    # Actually, simpler: since each is in a `proptest! { ... }` block, let's find `fn <test_name>` and then remove the enclosing `proptest! { ... }`.

    idx = content.find("fn " + test_name)
    if idx == -1:
        continue

    start_idx = content.rfind("    proptest! {", 0, idx)
    if start_idx == -1:
        continue

    # find the matching closing brace for proptest! {
    brace_count = 0
    in_block = False
    end_idx = -1
    for i in range(start_idx, len(content)):
        if content[i] == "{":
            brace_count += 1
            in_block = True
        elif content[i] == "}":
            brace_count -= 1

        if in_block and brace_count == 0:
            end_idx = i + 1
            break

    if end_idx != -1:
        content = content[:start_idx] + content[end_idx:]

with open("diagram_tool/src/ui/canvas/interaction_reducer.rs", "w") as f:
    f.write(content)
