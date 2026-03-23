# ADR-015: Ghost Diff System

## Status

**Accepted** (2026-03-23)

## Context

Seshat supports AI-assisted diagram editing via CLI agents. Without safeguards, AI agents could silently overwrite user changes, leading to lost work when AI and human edit concurrently, no visibility into what AI wants to change, and trust issues where users don't know what the AI did.

We need a mechanism that shows AI proposals BEFORE they're applied, letting users review and accept or reject them. This pattern is inspired by GitHub Copilot's inline suggestions (ghost text), Figma's multiplayer cursors showing others' edits, and Google Docs' suggestion mode for proposed changes.

## Decision

### The Ghost Diff Pattern

AI proposals appear as "ghost" previews — dashed, semi-transparent renderings of proposed changes. The user sees the possible future state and decides whether to accept it.

Current state appears with solid lines and full opacity. Ghost previews use dashed lines at 50% opacity. New nodes get a green "+" badge, modified nodes get a yellow "~" badge, and deleted nodes get a red "×" badge. Move operations show a dashed outline at the new position with a faint line connecting old and new positions. Add operations fade in with a dashed border. Delete operations show strikethrough at 30% opacity.

### Data Structures

The `ProposedChanges` struct (in a new file `diagram_models/src/proposed_changes.rs`) represents a complete proposal from an AI agent. It contains the `base_revision` that must match the current document at apply time, the `proposer` identifying the AI agent, a `proposed_at` timestamp, an ordered vector of `ProposedChange` items, and a human-readable `summary` for UI display.

The `ProposedChange` enum represents atomic changes that can be independently accepted or rejected. Variants include `MoveNode` (with from and to positions), `AddNode` (with the full node), `DeleteNode` (with the node as it existed for undo/diff display), `UpdateNodeLabel`, `UpdateNodeProperty` (generic), `AddEdge`, `DeleteEdge`, `UpdateEdgeRouting` (bend points), and `UpdateEdgeLabel`. Each variant includes both the "from" and "to" states for diffing.

The `GhostDiffState` struct (in `diagram_tool/src/ui/ghost_diff/state.rs`) manages UI state. It tracks the pending proposal and a map of change indices to user toggle states (accepted/rejected/undecided). This state is intentionally ephemeral — it does not survive page reload, tab close, or document switch.

### State Machine

The ghost diff system has five states: IDLE (no pending proposal), REVIEWING (proposal displayed, user reviewing), APPLYING (changes being applied), APPLIED (changes applied, dirty flag set), and CLEARED (proposal discarded).

Transitions occur on these events: `ProposalReceived` moves from IDLE to REVIEWING. `AcceptAll` or `AcceptSelected` moves from REVIEWING to APPLYING. `RejectAll` or `DocumentChanged` moves from REVIEWING to CLEARED. `ApplySuccess` moves from APPLYING to APPLIED. `ApplyFailed` moves from APPLYING to CLEARED. APPLIED and CLEARED immediately return to IDLE.

### Revision Matching and Conflict Prevention

Every proposal includes a `base_revision`. This is critical for preventing blind overwrites. When applying a proposal, we first check if `proposal.base_revision == document.revision`. If they don't match, the proposal is stale and rejected immediately with the expected and current revision values.

The `apply_proposal()` function in `diagram_tool/src/ui/ghost_diff/apply.rs` implements this. It returns an `ApplyResult` enum with three variants: `Applied` (all changes applied successfully), `Stale` (revision mismatch), or `PartialConflict` (some changes couldn't be applied due to user modifications).

Conflict reasons include: `NodeModifiedByUser` (user changed the node since the proposal was generated), `NodeDeletedByUser` (user deleted the node), `EdgeModifiedByUser`, and `InvalidEdgeTarget` (target node doesn't exist).

### Visual Rendering

The visual representation uses CSS classes for Dioxus rendering. Ghost nodes have dashed borders, 50% opacity, and a pulsing animation. New ghost nodes get a green border. Deleted ghost nodes have 30% opacity with strikethrough. Ghost edges use dashed stroke patterns.

Badges are positioned absolutely at the top-right of nodes. The "+" badge is green for additions, "~" is yellow for modifications, and "×" is red for deletions.

### User Actions

Users can take several actions. "Accept All" applies all changes, increments the revision, clears ghost state, marks the document dirty, and shows a toast notification. "Reject All" clears ghost state without modifying the document. "Review Selected" opens a modal with a checkbox list for individual toggles. "Accept Selected" applies only the checked changes.

### Edge Cases

When a user modifies the same node that an AI proposed a change for, we detect this by comparing the node state to the proposal's "from" value. We auto-skip that change and show a toast: "Skipped 1 change — you already modified it."

If a user partially accepts changes and then a new AI proposal arrives, we clear the old ghost state entirely. The new proposal starts fresh.

For large proposals with 50+ changes, we group by type in the UI and show a summary like "3 moves, 2 additions, 1 deletion" with expandable sections.

If a user closes a tab with a pending ghost diff, we silently discard it. No prompt. This is intentional — ghost diffs are transient by design.

If a proposal arrives while the user is already reviewing another, the new proposal wins. Only one proposal can be pending at a time.

### Persistence Policy

Ghost diffs are NOT persisted anywhere. They don't go to the filesystem (that's for source of truth), not to SQLite (that's for crash recovery), and not even temporarily to WAL beyond the time it takes to reach the UI. They exist only in-memory while the tab is open.

This decision is intentional. Stale proposals become invalid quickly. Users have a mental model that AI suggested something they haven't looked at yet. Managing proposal lifecycle across sessions adds complexity without clear benefit. Each new session gets a clean slate.

### CLI Integration

AI agents submit proposals via CLI. First they run `seshat show --json` to get the current state. Then they generate a proposal file. Then they run `seshat propose --input proposal.json --base-revision 42` to submit.

On success, the CLI returns a status of "queued" with a proposal ID and change count. If the revision is stale, it returns a status of "rejected" with the expected and current revisions and a hint to get the latest state.

### Contracts

The system maintains several invariants: only one pending proposal at a time, ghost state is cleared on document reload and tab close, and proposals with stale revision are rejected.

For the apply operation, preconditions are: `proposal.base_revision == document.revision` and at least one change selected. Postconditions on success: `document.revision` is incremented, dirty flag is set, and ghost state is cleared. Postconditions on reject: no document changes and ghost state is cleared.

### Module Structure

A new `diagram_models/src/proposed_changes.rs` file contains `ProposedChanges` and `ProposedChange`. A new `diagram_tool/src/ui/ghost_diff/` directory contains `mod.rs` (exports), `state.rs` (state types), `apply.rs` (application logic), `render.rs` (Dioxus components), `toast.rs` (notifications), and `tests.rs` (integration tests).

## Consequences

### Positive

Users have full control and can see exactly what the AI proposes before accepting. Transparent AI behavior builds trust. Human priority is enforced — users always win in conflicts. The mental model is simple: ghost diffs are transient. Users get granular control with accept/reject at the individual change level.

### Negative

If a user closes a tab without reviewing, the ghost diff is gone. This is an intentional trade-off for simplicity. The UI requires a visual diff rendering layer, adding complexity. Stale proposals mean AI may need to regenerate if the user changes the document during review.

### Risks

Large proposals may overwhelm users, mitigated by grouping and summary views. Complex proposals may have rendering latency, mitigated by lazy loading and virtual scrolling.

---

## Machine-Readable Spec (JSONL)

```jsonl
{"type":"adr","id":"adr-015","title":"Ghost Diff System","status":"accepted","date":"2026-03-23","context":{"problem":"AI agents could silently overwrite user changes without safeguards. Need visibility and user control over AI proposals.","inspiration":["GitHub Copilot inline suggestions (ghost text)","Figma multiplayer cursors","Google Docs suggestion mode"]},"decision":{"pattern":"Ghost Diff","description":"AI proposals appear as dashed, semi-transparent previews. User sees possible future state and decides to accept/reject."},"visual_spec":{"current_state":{"style":"solid lines, full opacity"},"ghost_preview":{"style":"dashed, 50% opacity","animation":"pulse at new positions"},"badges":{"new":{"symbol":"+","color":"#22c55e (green)"},"modified":{"symbol":"~","color":"#eab308 (yellow)"},"deleted":{"symbol":"×","color":"#ef4444 (red)"}},"change_visualizations":[{"type":"MoveNode","visual":"Dashed outline at new position, faint line from old to new","badge":"→","animation":"Pulse at new position"},{"type":"AddNode","visual":"Dashed outline, 50% opacity","badge":"+","animation":"Fade in"},{"type":"DeleteNode","visual":"Solid with strikethrough, 30% opacity","badge":"×","animation":"Fade out preview"},{"type":"UpdateNodeLabel","visual":"Inline diff: ~~old~~ **new**","badge":"~","animation":"None"},{"type":"AddEdge","visual":"Dashed line with arrow, 50% opacity","badge":"+","animation":"Draw animation"},{"type":"DeleteEdge","visual":"Solid with strikethrough","badge":"×","animation":"Fade out"},{"type":"UpdateEdgeRouting","visual":"Dashed curve overlay","badge":"~","animation":"Morph animation"}]},"data_structures":{"proposed_changes":{"location":"diagram_models/src/proposed_changes.rs","rust_type":"ProposedChanges","description":"Complete proposal from AI agent containing multiple atomic changes","fields":{"base_revision":{"type":"Revision","constraint":"Must match current doc revision at apply time"},"proposer":{"type":"AuthorId","description":"AI agent identifier"},"proposed_at":{"type":"Timestamp","description":"Unix timestamp when generated"},"changes":{"type":"ImVector<ProposedChange>","description":"Ordered list of atomic changes"},"summary":{"type":"String","description":"Human-readable for UI display"}},"invariants":["IC1: base_revision must match current document revision at apply time","IC2: All changes are atomic (apply individually)","IC3: proposer identifies the AI agent","IC4: proposed_at is a valid Unix timestamp"]},"proposed_change":{"location":"diagram_models/src/proposed_changes.rs","rust_type":"ProposedChange enum","variants":[{"name":"MoveNode","fields":{"node_id":"NodeId","from":"SerializedPoint","to":"SerializedPoint"}},{"name":"AddNode","fields":{"node":"Node"}},{"name":"DeleteNode","fields":{"node_id":"NodeId","was":"Node"},"note":"Includes deleted entity for undo/diff"},{"name":"UpdateNodeLabel","fields":{"node_id":"NodeId","from":"String","to":"String"}},{"name":"UpdateNodeProperty","fields":{"node_id":"NodeId","property":"String","from":"serde_json::Value","to":"serde_json::Value"}},{"name":"AddEdge","fields":{"edge_id":"EdgeId","edge":"Edge"}},{"name":"DeleteEdge","fields":{"edge_id":"EdgeId","was":"Edge"}},{"name":"UpdateEdgeRouting","fields":{"edge_id":"EdgeId","from_bend_points":"ImVector<SerializedPoint>","to_bend_points":"ImVector<SerializedPoint>"}},{"name":"UpdateEdgeLabel","fields":{"edge_id":"EdgeId","from":"String","to":"String"}}]},"ghost_diff_state":{"location":"diagram_tool/src/ui/ghost_diff/state.rs","rust_type":"GhostDiffState","description":"UI state for managing ghost diff visualization","fields":{"pending":{"type":"Option<PendingProposal>","description":"The pending proposal if any"},"toggled":{"type":"HashMap<usize, bool>","description":"Map of change index to user toggle state"}},"methods":{"has_pending":"() -> bool","change_count":"() -> usize","accepted_indices":"() -> Vec<usize>","rejected_indices":"() -> Vec<usize>","undecided_indices":"() -> Vec<usize>"},"lifecycle":["Created when ProposedChanges received from CLI/WAL","Updated as user toggles individual changes","Destroyed when user accepts/rejects OR document reloads"],"not_persisted":true,"ephemeral_scope":["Does not survive page reload","Does not survive tab close","Does not survive document switch"]},"pending_proposal":{"location":"diagram_tool/src/ui/ghost_diff/state.rs","rust_type":"PendingProposal","fields":{"proposal":"ProposedChanges","received_at":"std::time::Instant","document_revision_at_receipt":"Revision"}}},"state_machine":{"states":["IDLE - No pending proposal","REVIEWING - Proposal displayed, user reviewing","APPLYING - Changes being applied to document","APPLIED - Changes applied, dirty flag set","CLEARED - Proposal discarded"],"transitions":[{"from":"IDLE","to":"REVIEWING","event":"ProposalReceived"},{"from":"REVIEWING","to":"APPLYING","event":"AcceptAll or AcceptSelected"},{"from":"REVIEWING","to":"CLEARED","event":"RejectAll or DocumentChanged"},{"from":"APPLYING","to":"APPLIED","event":"ApplySuccess"},{"from":"APPLYING","to":"CLEARED","event":"ApplyFailed (conflict)"},{"from":"APPLIED","to":"IDLE","event":"immediate"},{"from":"CLEARED","to":"IDLE","event":"immediate"}]},"revision_matching":{"purpose":"Prevent blind overwrites","mechanism":"Every proposal includes base_revision. If doc.revision != proposal.base_revision, reject as stale."},"apply_function":{"location":"diagram_tool/src/ui/ghost_diff/apply.rs","signature":"apply_proposal(doc: &mut DiagramDocument, proposal: &ProposedChanges, accepted_indices: &[usize]) -> ApplyResult","preconditions":["PA1: proposal.base_revision equals doc.revision","PA2: All changes in accepted_indices are valid indices","PA3: Document is not currently locked"],"postconditions":["QA1: On success, doc.revision is incremented","QA2: On success, dirty flag is set","QA3: On partial conflict, unconflicting changes are applied"],"apply_result_variants":[{"name":"Applied","fields":{"applied_count":"usize","new_revision":"Revision"}},{"name":"Stale","fields":{"expected_revision":"Revision","current_revision":"Revision"}},{"name":"PartialConflict","fields":{"applied_indices":"Vec<usize>","skipped_indices":"Vec<usize>","skip_reasons":"HashMap<usize, ConflictReason>"}}],"conflict_reasons":[{"name":"NodeModifiedByUser","fields":{"node_id":"NodeId"}},{"name":"NodeDeletedByUser","fields":{"node_id":"NodeId"}},{"name":"EdgeModifiedByUser","fields":{"edge_id":"EdgeId"}},{"name":"InvalidEdgeTarget","fields":{"edge_id":"EdgeId","missing_node":"NodeId"}}]},"user_actions":[{"action":"Accept All","trigger":"Button click","steps":["Apply all changes","Increment revision","Clear ghost state","Mark dirty","Toast: Applied N changes"]},{"action":"Reject All","trigger":"Button click","steps":["Clear ghost state","No document changes","Toast: Changes discarded"]},{"action":"Review Selected","trigger":"Button click","steps":["Open modal with checkbox list","Allow individual toggles"]},{"action":"Toggle Individual","trigger":"Checkbox click","steps":["Update toggled map","Update visual preview"]},{"action":"Accept Selected","trigger":"Button in modal","steps":["Apply only checked changes","Increment revision","Clear ghost state"]}],"edge_cases":[{"scenario":"User modifies same node AI proposed change for","detection":"Compare node state to proposal's from value","behavior":"Auto-skip that change","toast":"Skipped 1 change — you already modified it."},{"scenario":"Partial accept, then new AI proposal","detection":"New ProposalReceived event","behavior":"Clear old ghost state entirely. New proposal starts fresh."},{"scenario":"50+ changes in proposal","detection":"changes.len() > 50","behavior":"Group by type in UI. Summary: 3 moves, 2 additions, 1 deletion. Expandable sections."},{"scenario":"Tab close with pending ghost diff","detection":"beforeunload event","behavior":"Silent discard. No prompt. Intentional design."},{"scenario":"Proposal arrives while reviewing another","detection":"New ProposalReceived while REVIEWING","behavior":"Replace. Only one proposal at a time. New proposal wins."},{"scenario":"Apply fails partway through","detection":"PartialConflict result","behavior":"Show detailed error modal with skip reasons. User can retry or cancel."}],"module_structure":{"diagram_models/src/":["proposed_changes.rs - NEW: ProposedChanges, ProposedChange enum","proposed_changes_tests.rs - NEW: Unit tests"],"diagram_tool/src/ui/ghost_diff/":["mod.rs - Exports","state.rs - GhostDiffState, PendingProposal","apply.rs - apply_proposal, ApplyResult, ConflictReason","render.rs - Dioxus components for ghost visualization","toast.rs - Toast notifications for accept/reject","tests.rs - Integration tests"]},"cli_integration":{"commands":[{"name":"show","usage":"seshat show --json","description":"AI reads current state"},{"name":"propose","usage":"seshat propose --input proposal.json --base-revision 42","description":"AI submits proposal"}],"cli_response_success":{"status":"queued","proposal_id":"prop-abc123","base_revision":42,"change_count":3},"cli_response_stale":{"status":"rejected","reason":"stale_revision","expected_revision":42,"current_revision":45,"hint":"Run 'seshat show --json' to get latest state"}},"persistence_policy":{"filesystem":"NO - File is source of truth, ghost diffs are transient","sqlite_cache":"NO - Cache is for recovery, not proposals","wal_restate":"YES (temporarily) - Proposal traverses WAL to reach UI, then discarded","in_memory":"YES (ephemeral) - Only while tab is open"},"why_not_persist":["Stale proposals become invalid quickly","User mental model: AI suggested something, I haven't looked at it yet","Complexity: managing proposal lifecycle across sessions","Fresh start: new session = clean slate"],"contracts":{"invariants":["IG1: Only one pending proposal at a time","IG2: Ghost state cleared on document reload","IG3: Ghost state cleared on tab close","IG4: Proposals with stale revision are rejected"],"preconditions_apply":["PGA1: proposal.base_revision == document.revision","PGA2: At least one change selected for apply"],"postconditions_apply_success":["QGA1: document.revision incremented","QGA2: Dirty flag set","QGA3: Ghost state cleared"],"postconditions_reject":["QGR1: No document changes","QGR2: Ghost state cleared"]},"consequences":{"positive":["User control: Users see exactly what AI proposes","Trust: Transparent AI behavior","Human priority: User always wins in conflicts","Simple mental model: Ghost diffs are transient","Granular control: Accept/reject at individual change level"],"negative":["Lost work: If user closes tab without reviewing, ghost diff is gone","UI complexity: Requires visual diff rendering layer","Stale proposals: AI may need to regenerate if user changes doc during review"],"risks":[{"risk":"Overwhelm from large proposals","mitigation":"Grouping by type, summary view"},{"risk":"Latency for complex proposals","mitigation":"Lazy loading, virtual scrolling"}]},"related":["ADR-014: Diagram JSON Schema","ADR-016: Persistence Strategy","12_SINGLE_LOG_ARCHITECTURE.md","10_AI_CLI_CONTRACT.md"]}
```
