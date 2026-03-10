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
AI agents should use the `seshat` CLI to read/write state without touching the database directly. All interactions are backed by a single WAL and durable Restate orchestration.

### Exporting Graph State
```bash
seshat export --format json > current_state.json
```

### Validating Changes Pre-Flight
AI can validate a patch before applying it to ensure it does not conflict with recent Human UI edits:
```bash
seshat validate my_proposal.json
```

### Applying Changes
AI proposes architectural changes by supplying a patch or a full sub-graph:
```bash
seshat apply my_proposal.json
```

## Conflict Resolution & Rich Diffing
Seshat implements a strict **Human Priority** concurrency model. If a Human modifies the diagram while the AI is calculating its patch, the backend's conditional log append will fail.

When a conflict occurs, the CLI rejects the patch and returns a **Rich Diff** structured JSON error. The AI must parse this diff to understand the exact delta and correct its proposal.

### Rich Diff JSON Example
```json
{
  "status": "rejected",
  "reason": "Human Priority Block",
  "conflict_context": {
    "expected_revision": 42,
    "actual_revision": 44,
    "conflicting_entities": ["node-123"],
    "diff": {
      "node-123": {
        "human_state": {
          "x": 500.0,
          "y": 200.0
        },
        "ai_proposed_state": {
          "x": 100.0,
          "y": 200.0
        }
      }
    }
  }
}
```
*Note: The AI agent should read this diff, adjust its routing or placement logic based on the `human_state` coordinates, and re-run the `seshat apply` command.*
