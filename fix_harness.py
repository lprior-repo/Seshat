import re

with open('diagram_tool/src/models/harness.rs', 'r') as f:
    content = f.read()

# Fix NodeId issues in DomainOp::NodeAdd and NodeMove where `id` is a variable
content = re.sub(r'DomainOp::NodeAdd \{\s*id,\s*x:', r'DomainOp::NodeAdd {\n                id: NodeId::new(id),\n                x:', content)
content = re.sub(r'DomainOp::NodeMove \{\s*id,\s*x:', r'DomainOp::NodeMove {\n                    id: NodeId::new(id),\n                    x:', content)

# Fix id: format!("node-{}", i)
content = re.sub(r'id: format!\("node-\{\}", i\)', r'id: NodeId::new(format!("node-{}", i))', content)

# Fix strings
content = re.sub(r'id: "node-crash-1"\.to_string\(\)', r'id: NodeId::new("node-crash-1".to_string())', content)
content = re.sub(r'id: "node-1"\.to_string\(\)', r'id: NodeId::new("node-1".to_string())', content)
content = re.sub(r'id: "node-2"\.to_string\(\)', r'id: NodeId::new("node-2".to_string())', content)
content = re.sub(r'id: "edge-1"\.to_string\(\)', r'id: EdgeId::new("edge-1".to_string())', content)
content = re.sub(r'source: "source-node"\.to_string\(\)', r'source: NodeId::new("source-node".to_string())', content)
content = re.sub(r'target: "target-node"\.to_string\(\)', r'target: NodeId::new("target-node".to_string())', content)
content = re.sub(r'id: "obs-node"\.to_string\(\)', r'id: NodeId::new("obs-node".to_string())', content)
content = re.sub(r'vec!\["node-a"\.to_string\(\), "node-c"\.to_string\(\)\]', r'vec![NodeId::new("node-a".to_string()), NodeId::new("node-c".to_string())]', content)

with open('diagram_tool/src/models/harness.rs', 'w') as f:
    f.write(content)
