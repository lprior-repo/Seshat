import re

with open('diagram_tool/src/models/projection/ops/node_ops.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'DomainOp::NodeAdd \{\s*id,\s*x,\s*y,\s*width,\s*height,\s*label,\s*\} => apply_node_add\(state, id, \*x, \*y, \*width, \*height, label\),',
    r'DomainOp::NodeAdd {\n            id,\n            x,\n            y,\n            width,\n            height,\n            label,\n        } => apply_node_add(state, id.as_str(), *x, *y, *width, *height, label),',
    content
)
content = re.sub(r'DomainOp::NodeMove \{ id, x, y \} => apply_node_move\(state, id, \*x, \*y\),', r'DomainOp::NodeMove { id, x, y } => apply_node_move(state, id.as_str(), *x, *y),', content)
content = re.sub(r'DomainOp::NodeDelete \{ id \} => apply_node_delete\(state, id\),', r'DomainOp::NodeDelete { id } => apply_node_delete(state, id.as_str()),', content)
content = re.sub(r'DomainOp::NodeRestore \{ id \} => apply_node_restore\(state, id\),', r'DomainOp::NodeRestore { id } => apply_node_restore(state, id.as_str()),', content)

with open('diagram_tool/src/models/projection/ops/node_ops.rs', 'w') as f:
    f.write(content)
