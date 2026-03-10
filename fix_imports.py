import os


def fix_imports(filepath, new_import):
    with open(filepath, "r") as f:
        content = f.read()

    # Replace use super::* with the new import
    content = content.replace("use super::*;", new_import)

    # In geometry_tests.rs we also have to delete the duplicate `union` method.
    if "geometry_tests.rs" in filepath:
        # Just find `fn union` and comment it out or remove its block
        lines = content.split("\n")
        out_lines = []
        skip = False
        for line in lines:
            if "fn union(&self, other: &AABB) -> AABB {" in line:
                skip = True

            if skip:
                if "    }" in line and len(line.strip()) == 1:
                    skip = False
                    continue
                continue

            out_lines.append(line)
        content = "\n".join(out_lines)

    with open(filepath, "w") as f:
        f.write(content)


fix_imports("diagram_tool/src/geometry/geometry_tests.rs", "use crate::geometry::*;")
fix_imports(
    "diagram_tool/src/geometry/snap/tests.rs",
    "use crate::geometry::snap::*;\n    use crate::geometry::*;",
)
