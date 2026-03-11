import os
import re

patterns = [
    r'\.unwrap\(\)',
    r'\.expect\(',
    r'panic!\(',
    r'unsafe \{'
]
regex = re.compile('|'.join(patterns))

def process_file(filepath):
    results = []
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except Exception:
        return results

    in_test = False
    test_brace_level = 0
    brace_level = 0

    for i, line in enumerate(lines):
        line_num = i + 1
        stripped = line.strip()
        
        # very basic test ignore logic
        if '#[cfg(test)]' in stripped or '#[test]' in stripped or '#[kani::proof]' in stripped:
            in_test = True
            test_brace_level = brace_level
        
        if in_test:
            if '{' in stripped:
                brace_level += stripped.count('{')
            if '}' in stripped:
                brace_level -= stripped.count('}')
                if brace_level <= test_brace_level:
                    in_test = False
            continue
        
        if '{' in stripped:
            brace_level += stripped.count('{')
        if '}' in stripped:
            brace_level -= stripped.count('}')

        if regex.search(line) and not in_test:
            # check if it's just a comment
            if stripped.startswith('//'):
                continue
            # if it's unwrap_or_else, we might skip but let's see
            if 'unwrap_or_else' in line:
                continue
            results.append((filepath, line_num, line.strip()))

    return results

for root, dirs, files in os.walk('diagram_tool/src'):
    if 'tests' in root.split(os.sep):
        continue
    for file in files:
        if file.endswith('.rs') and not file.endswith('test.rs') and not file.endswith('tests.rs'):
            matches = process_file(os.path.join(root, file))
            for m in matches:
                print(f"{m[0]}:{m[1]}: {m[2]}")

