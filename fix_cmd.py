import re

with open("diagram_tool/src/ui/commands.rs", "r") as f:
    content = f.read()

# Remove the whole // Additional copy/paste tests (bd-2b4) section up to but not including #[cfg(test)] mod proptests
start_marker = "    // Additional copy/paste tests (bd-2b4)"
end_marker = "    #[test]"
proptests_marker = "#[cfg(test)]\n#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]\nmod proptests {"

start_idx = content.find(start_marker)
if start_idx != -1:
    # find the next closing brace for the `mod tests` block
    # It should be right before `mod proptests`
    end_idx = content.find("}\n\n#[cfg(test)]", start_idx)
    if end_idx != -1:
        content = content[:start_idx] + content[end_idx:]

with open("diagram_tool/src/ui/commands.rs", "w") as f:
    f.write(content)
