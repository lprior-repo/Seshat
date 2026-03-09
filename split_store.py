import os
import re
import math

with open("diagram_tool/src/store.rs", "r") as f:
    source = f.read()

try:
    main_source, tests_source = source.split("#[cfg(test)]\nmod tests {", 1)
except ValueError:
    print("Could not find tests module")
    exit(1)

# Remove the trailing '}' from tests_source
tests_source = tests_source.rsplit("}", 1)[0]

# Extract tests
# A test usually starts with #[test] or /// comments followed by #[test]
# We can split by #[test]
test_chunks = re.split(r'(?=#\[test\])', tests_source)

# test_chunks[0] might contain some use statements
header = test_chunks[0]
tests = test_chunks[1:]

os.makedirs("diagram_tool/src/store/tests", exist_ok=True)

file_idx = 1
current_file_lines = 0
current_file_content = header

for test in tests:
    test_lines = test.count('\n')
    if current_file_lines + test_lines > 250 and current_file_lines > 0:
        with open(f"diagram_tool/src/store/tests/part_{file_idx}.rs", "w") as f:
            f.write("use super::*;\nuse std::path::{Path, PathBuf};\nuse tempfile::TempDir;\nuse rusqlite::Connection;\n" + current_file_content)
        file_idx += 1
        current_file_lines = 0
        current_file_content = header
    
    current_file_content += test
    current_file_lines += test_lines

if current_file_content.strip() != header.strip():
    with open(f"diagram_tool/src/store/tests/part_{file_idx}.rs", "w") as f:
        f.write("use super::*;\nuse std::path::{Path, PathBuf};\nuse tempfile::TempDir;\nuse rusqlite::Connection;\n" + current_file_content)

# create mod.rs for tests
with open("diagram_tool/src/store/tests/mod.rs", "w") as f:
    f.write("#![cfg(test)]\n")
    f.write("use super::*;\n")
    for i in range(1, file_idx + 1):
        f.write(f"mod part_{i};\n")

print(f"Tests split into {file_idx} files.")
