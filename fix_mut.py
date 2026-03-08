import re
import sys

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Change let mut bootstrap = bootstrap_store(...) to let bootstrap = ...
    content = re.sub(r'let mut bootstrap = (store::)?bootstrap_store', r'let bootstrap = \1bootstrap_store', content)
    
    with open(filepath, 'w') as f:
        f.write(content)

fix_file('diagram_tool/src/models/export.rs')
fix_file('diagram_tool/src/models/sync.rs')
