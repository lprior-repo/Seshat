import os
import re


def main():
    os.makedirs("diagram_tool/src/geometry/snap", exist_ok=True)
    os.makedirs("diagram_tool/src/geometry/tests", exist_ok=True)

    with open("diagram_tool/src/geometry/mod.rs", "r") as f:
        mod_lines = f.readlines()

    with open("diagram_tool/src/geometry/snap.rs", "r") as f:
        snap_lines = f.readlines()

    # We will write the implementation to new files, and tests to tests/mod.rs and tests/snap_tests.rs

    # Let's find the start of tests in mod.rs
    mod_test_start = 0
    for i, line in enumerate(mod_lines):
        if line.startswith("#[cfg(test)]"):
            mod_test_start = i
            break

    mod_impl = mod_lines[:mod_test_start]
    mod_tests = mod_lines[mod_test_start:]

    # Find start of tests in snap.rs
    snap_test_start = 0
    for i, line in enumerate(snap_lines):
        if line.startswith("#[cfg(test)]"):
            snap_test_start = i
            break

    snap_impl = snap_lines[:snap_test_start]
    snap_tests = snap_lines[snap_test_start:]

    # Let's construct primitives.rs
    # We will just write a new mod.rs and split things using python
    # Wait, instead of perfectly parsing, I can just create the files based on the content I know is there.

    # This python script approach might be too complex to get right for every struct.
    pass


if __name__ == "__main__":
    main()
