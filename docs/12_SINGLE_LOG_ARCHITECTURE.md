# The Restate Single-Log Architecture

Seshat’s backend is built on the philosophy that **"Every System is a Log"**. To manage the inherent complexity of a Two-Way Sync system where Humans (UI) and AI agents (CLI) are concurrently modifying the same architecture diagram, we avoid distributed coordination entirely by relying on **Restate** and a single Write-Ahead Log (WAL).

## 1. Coordination Avoidance
In traditional distributed applications, ensuring that an AI agent doesn't overwrite a human's changes requires distributed locks, complex retry loops, and fencing tokens. This leads to race conditions and "zombie" processes corrupting state.

By routing everything through a single log:
- **There are no distributed locks.** The log provides a single, linearizable history of events as the absolute ground truth.
- **State is Virtualized.** Diagram documents act as **Virtual Objects** in Restate. Handlers that mutate the diagram act like methods on that object, processing events sequentially from the log.

## 2. Durable Execution & The Step Journal
When an AI agent performs a complex, multi-step refactoring of a diagram:
1. Every intermediate step is recorded as a **conditional append** to the log.
2. The AI agent's execution context maintains a **Step Journal**.
3. If the agent crashes or times out midway through the refactor, the retry execution reads the journal. It automatically skips the steps already completed, preventing duplicate nodes or corrupted edge bindings.

## 3. Human Priority via Conditional Appends
Because the Human UI and the AI CLI share the same log, we enforce **Human Priority** without locking.
- When the human drags a node, the UI appends a `NodeMoved` event to the log.
- When the AI calculates a layout change and attempts to save it, it performs a **conditional append** (e.g., "Append this AI patch *only if* the latest revision is still X").
- If the human moved the node in the split second before the AI's append, the database revision has advanced. The conditional append fails immediately, protecting the human's work.

## 4. The Validation CLI & Rich Diffing
When an AI's conditional append is rejected due to human priority, Seshat does not just throw a generic error. It returns a **Rich Diff Context**.

Before or during application, the AI can also invoke a dry-run validation:
```bash
seshat validate patch.json
```

If rejected, the log generates a diff detailing exactly what changed between the AI's assumed state and the human's actual state. The AI uses this diff to recalculate its geometry (e.g., routing arrows around the human's new node position) and retries the operation, all powered seamlessly by the step journal.