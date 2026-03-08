import re
import sys

def fix_unwraps(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Change async fn test_name() { to async fn test_name() -> Result<(), Box<dyn std::error::Error>> {
    content = re.sub(r'async fn ([a-zA-Z0-9_]+)\(\)\s*\{', r'async fn \1() -> Result<(), Box<dyn std::error::Error>> {', content)
    
    # Add Ok(()) at the end of each test function
    content = re.sub(r'((\s*pool\.close\(\)\.await;|\s*Ok\(\(\)\)\n\s*)*)\}', r'\1    Ok(())\n}', content)
    # the above regex is bad, let's just do it simpler: find all async fn xxx() -> Result<...> { ... } and ensure they return Ok(())
    # better yet, replace .unwrap() with ? but phase5 tests are already Result?
    pass

def fix_map_err(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    content = re.sub(r'\.map_err\(\|e\|\s*StoreError::Io\(e\)\)', r'.map_err(StoreError::Io)', content)
    
    with open(filepath, 'w') as f:
        f.write(content)

fix_map_err('diagram_tool/tests/phase4_model_updates.rs')
