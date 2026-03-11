import os
import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Skip if already has kani proofs
    if '#[kani::proof]' in content:
        return

    # Find #[test] and optionally #[tokio::test]
    # Replace with #[cfg(kani)]\n#[kani::proof]\n#[test]
    # We must respect indentation
    
    lines = content.split('\n')
    new_lines = []
    modified = False
    
    for line in lines:
        stripped = line.strip()
        if stripped == '#[test]' or stripped == '#[tokio::test]':
            indent = line[:len(line) - len(stripped)]
            new_lines.append(f"{indent}#[cfg(kani)]")
            new_lines.append(f"{indent}#[kani::proof]")
            new_lines.append(line)
            modified = True
        else:
            new_lines.append(line)
            
    if modified:
        with open(filepath, 'w') as f:
            f.write('\n'.join(new_lines))
        print(f"Updated {filepath}")

for root, dirs, files in os.walk('diagram_tool'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))
