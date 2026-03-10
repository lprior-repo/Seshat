# AI CLI Contract (JSON API)

Seshat is designed to be a two-way sync between Human (UI) and AI (CLI). AI interacts with the rigorous SQLite WAL-mode backend speaking purely in JSON specifications.

## The Spec

The core atomic units are `Node` and `Edge`. The full schema aligns with `DiagramDocument`.

### Node JSON Example
```json
{
  "id": "node-123",
  "kind": "node",
  "label": "Authentication Service",
  "x": 100.0,
  "y": 200.0,
  "width": 150.0,
  "height": 50.0,
  "locked": false,
  "metadata": {
    "provider": "aws"
  }
}
```

### Edge JSON Example
```json
{
  "id": "edge-456",
  "source": "node-123",
  "target": "node-789",
  "label": "Authenticates",
  "style": "solid",
  "directed": true,
  "thickness": 1.5
}
```

## CLI Interactions
AI agents should use the `seshat` CLI to read/write state without touching the database directly. 
*(Note: Refer to `diagram_tool/src/cli.rs` for exact subcommands currently implemented)*.

### Exporting Graph State
```bash
seshat export --format json > current_state.json
```

### Applying Changes
AI proposes architectural changes by supplying a patch or a full sub-graph:
```bash
seshat apply my_proposal.json
```

## Conflict Resolution
If an AI proposes a change that violates constraints (e.g., creating a cycle in a strictly DAG subgraph), the CLI will reject the patch and return a structured JSON error. The AI must parse this error and correct its proposal.
