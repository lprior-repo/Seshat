import re

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Change async fn test_name() -> Result<(), Box<dyn std::error::Error>> {
    content = content.replace("Result<(), Box<dyn std::error::Error>>", "Result<(), Box<dyn std::error::Error + Send + Sync>>")
    
    with open(filepath, 'w') as f:
        f.write(content)

fix_file('diagram_tool/tests/io_005_performance.rs')
