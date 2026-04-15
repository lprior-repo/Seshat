# Gas Town Documentation - Full Site Scrape
# Scraped from gastown.dev on 2026-04-05
# 
# URL Mapping (requested -> actual):
# 1. /docs/overview -> EXISTS
# 2. /docs/installing -> NOT FOUND (no equivalent on site)
# 3. /docs/quickstart -> NOT FOUND (no equivalent on site)
# 4. /docs/glossary -> EXISTS
# 5. /docs/architecture -> EXISTS at /docs/design/architecture
# 6. /docs/concepts/molecules -> EXISTS
# 7. /docs/concepts/escalation -> EXISTS at /docs/design/escalation
# 8. /docs/concepts/scheduler -> EXISTS at /docs/design/scheduler
# 9. /docs/concepts/wasteland -> NOT FOUND (no equivalent on site)
# 10. /docs/concepts/hooks -> NOT FOUND (hooks documented in reference.md)
# 11. /docs/concepts/convoy -> EXISTS
# 12. /docs/concepts/polecat-lifecycle -> EXISTS
# 13. /docs/concepts/agent-providers -> EXISTS at /docs/agent-provider-integration
# 14. /docs/concepts/formulas -> NOT FOUND (formulas covered in molecules and reference)
# 15. /docs/concepts/dashboard -> NOT FOUND (dashboard is gt convoy list)
#
# Additional pages fetched for completeness:
# - /docs/why-these-features
# - /docs/reference
# - /docs/concepts/identity
# - /docs/concepts/propulsion-principle
# - /docs/concepts/integration-branches
# - /docs/CLEANUP
# - /docs/proxy-server
# - /docs/agent-provider-integration

---

# ==============================================================================
# SECTION 1: Overview
# URL: https://gastown.dev/docs/overview
# ==============================================================================

This document provides a conceptual overview of Gas Town's architecture, focusing on the role taxonomy and how different agents interact.

## Why Gas Town Exists

As AI agents become central to engineering workflows, teams face new challenges:
* **Accountability:** Who did what? Which agent introduced this bug?
* **Quality:** Which agents are reliable? Which need tuning?
* **Efficiency:** How do you route work to the right agent?
* **Scale:** How do you coordinate agents across repos and teams?

Gas Town is an orchestration layer that treats AI agent work as structured data. Every action is attributed. Every agent has a track record. Every piece of work has provenance. See Why These Features for the full rationale, and Glossary for terminology.

## Role Taxonomy

Gas Town has several agent types, each with distinct responsibilities and lifecycles.

### Infrastructure Roles

These roles manage the Gas Town system itself:

| Role | Description | Lifecycle |
| --- | --- | --- |
| **Mayor** | Global coordinator at mayor/ | Singleton, persistent |
| **Deacon** | Background supervisor daemon (watchdog chain) | Singleton, persistent |
| **Witness** | Per-rig polecat lifecycle manager | One per rig, persistent |
| **Refinery** | Per-rig merge queue processor | One per rig, persistent |

### Worker Roles

These roles do actual project work:

| Role | Description | Lifecycle |
| --- | --- | --- |
| **Polecat** | Worker with persistent identity, ephemeral sessions | Witness-managed (details) |
| **Crew** | Persistent worker with own clone | Long-lived, user-managed |
| **Dog** | Deacon helper for infrastructure tasks | Persistent identity, Deacon-managed |

## Convoys: Tracking Work

A **convoy** is how you track batched work in Gas Town. When you kick off work - even a single issue - create a convoy to track it.

```
# Create a convoy tracking some issues
gt convoy create "Feature X" gt-abc gt-def --notify overseer

# Check progress
gt convoy status hq-cv-abc

# Dashboard of active convoys
gt convoy list
```

**Why convoys matter:**
* Single view of "what's in flight"
* Cross-rig tracking (convoy in hq-_, issues in gt-_, bd-*)
* Auto-notification when work lands
* Historical record of completed work (`gt convoy list --all`)

The "swarm" is the set of workers currently assigned to a convoy's issues. When issues close, the convoy lands. See Convoys for details.

## Crew vs Polecats

Both do project work, but with key differences:

| Aspect | Crew | Polecat |
| --- | --- | --- |
| **Lifecycle** | Persistent (user controls) | Transient (Witness controls) |
| **Monitoring** | None | Witness watches, nudges, recycles |
| **Work assignment** | Human-directed or self-assigned | Slung via `gt sling` |
| **Git state** | Pushes to main directly | Works on branch, Refinery merges |
| **Cleanup** | Manual | Automatic on completion |
| **Identity** | `/crew/` | `/polecats/` |

**When to use Crew**:
* Exploratory work
* Long-running projects
* Work requiring human judgment
* Tasks where you want direct control

**When to use Polecats**:
* Discrete, well-defined tasks
* Batch work (tracked via convoys)
* Parallelizable work
* Work that benefits from supervision

## Dogs vs Crew

**Dogs are NOT workers**. This is a common misconception.

| Aspect | Dogs | Crew |
| --- | --- | --- |
| **Owner** | Deacon | Human |
| **Purpose** | Infrastructure tasks | Project work |
| **Scope** | Narrow, focused utilities | General purpose |
| **Lifecycle** | Very short (single task) | Long-lived |
| **Example** | Boot (triages Deacon health) | Joe (fixes bugs, adds features) |

Dogs are the Deacon's helpers for system-level tasks:
* **Boot**: Triages Deacon health on daemon tick
* Future dogs might handle: log rotation, health checks, etc.

If you need to do work in another rig, use **worktrees**, not dogs.

## Cross-Rig Work Patterns

When a crew member needs to work on another rig:

### Option 1: Worktrees (Preferred)

Create a worktree in the target rig:

```
# gastown/crew/joe needs to fix a beads bug
gt worktree beads
# Creates ~/gt/beads/crew/gastown-joe/
# Identity preserved: BD_ACTOR = gastown/crew/joe
```

Directory structure:

```
~/gt/beads/crew/gastown-joe/   # joe from gastown working on beads
~/gt/gastown/crew/beads-wolf/  # wolf from beads working on gastown
```

### Option 2: Dispatch to Local Workers

For work that should be owned by the target rig:

```
# Create issue in target rig
bd create --prefix beads "Fix authentication bug"

# Create convoy and sling to target rig
gt convoy create "Auth fix" bd-xyz
gt sling bd-xyz beads
```

### When to Use Which

| Scenario | Approach |
| --- | --- |
| You need to fix something quick | Worktree |
| Work should appear in your CV | Worktree |
| Work should be done by target rig team | Dispatch |
| Infrastructure/system task | Let Deacon handle it |

## Directory Structure

The town root (`~/gt/`) contains infrastructure directories (`mayor/`, `deacon/`) and per-project rigs. Each rig holds a bare repo (`.repo.git/`), a canonical beads database (`mayor/rig/.beads/`), and agent directories (`witness/`, `refinery/`, `crew/`, `polecats/`).

> For the full directory tree, see architecture.md.

## Identity and Attribution

All work is attributed to the actor who performed it:

```
Git commits: Author: gastown/crew/joe
Beads issues: created_by: gastown/crew/joe
Events: actor: gastown/crew/joe
```

Identity is preserved even when working cross-rig:
* `gastown/crew/joe` working in `~/gt/beads/crew/gastown-joe/`
* Commits still attributed to `gastown/crew/joe`
* Work appears on joe's CV, not beads rig's workers

## The Propulsion Principle

All Gas Town agents follow the same core principle:

> **If you find something on your hook, YOU RUN IT.**

This applies regardless of role. The hook is your assignment. Execute it immediately without waiting for confirmation. Gas Town is a steam engine - agents are pistons.

## Model Evaluation and A/B Testing

Gas Town's attribution system enables objective model comparison by tracking completion time, quality signals, and revision count per agent. Deploy different models on similar tasks and compare outcomes with `bd stats`.

See Why These Features for details on work history and capability-based routing.

## Common Mistakes

1. **Using dogs for user work**: Dogs are Deacon infrastructure. Use crew or polecats.
2. **Confusing crew with polecats**: Crew is persistent and human-managed. Polecats are transient and Witness-managed.
3. **Working in wrong directory**: Gas Town uses cwd for identity detection. Stay in your home directory.
4. **Waiting for confirmation when work is hooked**: The hook IS your assignment. Execute immediately.
5. **Creating worktrees when dispatch is better**: If work should be owned by the target rig, dispatch it instead.

---

# ==============================================================================
# SECTION 2: Glossary
# URL: https://gastown.dev/docs/glossary
# ==============================================================================

Gas Town is an agentic development environment for managing multiple Claude Code instances simultaneously using the `gt` and `bd` (Beads) binaries, coordinated with tmux in git-managed directories.

## Core Principles

### MEOW (Molecular Expression of Work)
Breaking large goals into detailed instructions for agents. Supported by Beads, Epics, Formulas, and Molecules. MEOW ensures work is decomposed into trackable, atomic units that agents can execute autonomously.

### GUPP (Gas Town Universal Propulsion Principle)
"If there is work on your Hook, YOU MUST RUN IT." This principle ensures agents autonomously proceed with available work without waiting for external input. GUPP is the heartbeat of autonomous operation.

### NDI (Nondeterministic Idempotence)
The overarching goal ensuring useful outcomes through orchestration of potentially unreliable processes. Persistent Beads and oversight agents (Witness, Deacon) guarantee eventual workflow completion even when individual operations may fail or produce varying results.

## Environments

### Town
The management headquarters (e.g., `~/gt/`). The Town coordinates all workers across multiple Rigs and houses town-level agents like Mayor and Deacon.

### Rig
A project-specific Git repository under Gas Town management. Each Rig has its own Polecats, Refinery, Witness, and Crew members. Rigs are where actual development work happens.

## Town-Level Roles

### Mayor
Chief-of-staff agent responsible for initiating Convoys, coordinating work distribution, and notifying users of important events. The Mayor operates from the town level and has visibility across all Rigs.

### Deacon
Daemon beacon running continuous Patrol cycles. The Deacon ensures worker activity, monitors system health, and triggers recovery when agents become unresponsive. Think of the Deacon as the system's watchdog.

### Dogs
The Deacon's crew of maintenance agents handling background tasks like cleanup, health checks, and system maintenance.

### Boot (the Dog)
A special Dog that checks the Deacon every 5 minutes, ensuring the watchdog itself is still watching. This creates a chain of accountability.

## Rig-Level Roles

### Polecat
Worker agents with persistent identity but ephemeral sessions. Each polecat has a permanent agent bead, CV chain, and work history that accumulates across assignments. Sessions and sandboxes are ephemeral — spawned for specific tasks, cleaned up on completion — but the identity persists. They work in isolated git worktrees to avoid conflicts.

### Refinery
Manages the Merge Queue for a Rig. The Refinery intelligently merges changes from Polecats, handling conflicts and ensuring code quality before changes reach the main branch.

### Witness
Patrol agent that oversees Polecats and the Refinery within a Rig. The Witness monitors progress, detects stuck agents, and can trigger recovery actions.

### Crew
Long-lived, named agents for persistent collaboration. Unlike ephemeral Polecats, Crew members maintain context across sessions and are ideal for ongoing work relationships.

## Work Units

### Bead
Git-backed atomic work unit stored in Dolt. Beads are the fundamental unit of work tracking in Gas Town. They can represent issues, tasks, epics, or any trackable work item.

### Formula
TOML-based workflow source template. Formulas define reusable patterns for common operations like patrol cycles, code review, or deployment.

### Protomolecule
A template class for instantiating Molecules. Protomolecules define the structure and steps of a workflow without being tied to specific work items.

### Molecule
Durable chained Bead workflows. Molecules represent multi-step processes where each step is tracked as a Bead. They survive agent restarts and ensure complex workflows complete.

### Wisp
Ephemeral Beads destroyed after runs. Wisps are lightweight work items used for transient operations that don't need permanent tracking.

### Hook
A special pinned Bead for each agent. The Hook is an agent's primary work queue - when work appears on your Hook, GUPP dictates you must run it.

## Workflow Commands

### Convoy
Primary work-order wrapping related Beads. Convoys group related tasks together and can be assigned to multiple workers. Created with `gt convoy create`.

### Slinging
Assigning work to agents via `gt sling`. When you sling work to a Polecat or Crew member, you're putting it on their Hook for execution.

### Nudging
Real-time messaging between agents with `gt nudge`. Nudges allow immediate communication without going through the mail system.

### Handoff
Agent session refresh via `/handoff`. When context gets full or an agent needs a fresh start, handoff transfers work state to a new session.

### Seance
Communicating with previous sessions via `gt seance`. Allows agents to query their predecessors for context and decisions from earlier work.

### Patrol
Ephemeral loop maintaining system heartbeat. Patrol agents (Deacon, Witness) continuously cycle through health checks and trigger actions as needed.

---

# ==============================================================================
# SECTION 3: Architecture
# URL: https://gastown.dev/docs/design/architecture
# ==============================================================================

Technical architecture for Gas Town multi-agent workspace management.

## Two-Level Beads Architecture

Gas Town uses a two-level beads architecture to separate organizational coordination from project implementation work.

| Level | Location | Prefix | Purpose |
| --- | --- | --- | --- |
| **Town** | `~/gt/.beads/` | `hq-*` | Cross-rig coordination, Mayor mail, agent identity |
| **Rig** | `/mayor/rig/.beads/` | project prefix | Implementation work, MRs, project issues |

### Town-Level Beads (`~/gt/.beads/`)

Organizational chain for cross-rig coordination:
* Mayor mail and messages
* Convoy coordination (batch work across rigs)
* Strategic issues and decisions
* **Town-level agent beads** (Mayor, Deacon)
* **Role definition beads** (global templates)

### Rig-Level Beads (`/mayor/rig/.beads/`)

Project chain for implementation work:
* Bugs, features, tasks for the project
* Merge requests and code reviews
* Project-specific molecules
* **Rig-level agent beads** (Witness, Refinery, Polecats)

## Agent Bead Storage

Agent beads track lifecycle state for each agent. Storage location depends on the agent's scope.

| Agent Type | Scope | Bead Location | Bead ID Format |
| --- | --- | --- | --- |
| Mayor | Town | `~/gt/.beads/` | `hq-mayor` |
| Deacon | Town | `~/gt/.beads/` | `hq-deacon` |
| Boot | Town | `~/gt/.beads/` | `hq-boot` |
| Dogs | Town | `~/gt/.beads/` | `hq-dog-` |
| Witness | Rig | `/.beads/` | `--witness` |
| Refinery | Rig | `/.beads/` | `--refinery` |
| Polecats | Rig | `/.beads/` | `--polecat-` |
| Crew | Rig | `/.beads/` | `--crew-` |

### Role Beads

Role beads are global templates stored in town beads with `hq-` prefix:
* `hq-mayor-role` - Mayor role definition
* `hq-deacon-role` - Deacon role definition
* `hq-boot-role` - Boot role definition
* `hq-witness-role` - Witness role definition
* `hq-refinery-role` - Refinery role definition
* `hq-polecat-role` - Polecat role definition
* `hq-crew-role` - Crew role definition
* `hq-dog-role` - Dog role definition

Each agent bead references its role bead via the `role_bead` field.

## Agent Taxonomy

### Town-Level Agents (Cross-Rig)

| Agent | Role | Persistence |
| --- | --- | --- |
| **Mayor** | Global coordinator, handles cross-rig communication and escalations | Persistent |
| **Deacon** | Daemon beacon — receives heartbeats, runs plugins and monitoring | Persistent |
| **Boot** | Deacon watchdog — spawned by daemon for triage decisions when Deacon is down | Ephemeral |
| **Dogs** | Long-running workers for cross-rig batch work | Variable |

### Rig-Level Agents (Per-Project)

| Agent | Role | Persistence |
| --- | --- | --- |
| **Witness** | Monitors polecat health, handles nudging and cleanup | Persistent |
| **Refinery** | Processes merge queue, runs verification | Persistent |
| **Polecats** | Workers with persistent identity, assigned to specific issues | Persistent identity, ephemeral sessions |
| **Crew** | Human workspaces — full git clones, user-managed lifecycle | Persistent |

## Directory Structure

```
~/gt/                           Town root
├── .beads/                     Town-level beads (hq-* prefix)
│   ├── metadata.json           Beads config (dolt_mode, dolt_database)
│   └── routes.jsonl            Prefix → rig routing table
├── .dolt-data/                 Centralized Dolt data directory
│   ├── hq/                     Town beads database (hq-* prefix)
│   ├── gastown/                Gastown rig database (gt-* prefix)
│   ├── beads/                  Beads rig database (bd-* prefix)
│   └── /                Per-rig databases
├── daemon/                     Daemon runtime state
│   ├── dolt-state.json         Dolt server state (pid, port, databases)
│   ├── dolt-server.log         Server log
│   └── dolt.pid                Server PID file
├── deacon/                     Deacon workspace
│   └── dogs//           Dog worker directories
├── mayor/                      Mayor agent home
│   ├── town.json               Town configuration
│   ├── rigs.json               Rig registry
│   ├── daemon.json             Daemon patrol config
│   └── accounts.json           Claude Code account management
├── settings/                   Town-level settings
│   ├── config.json             Town settings (agents, themes)
│   └── escalation.json         Escalation routes and contacts
├── config/
│   └── messaging.json          Mail lists, queues, channels
└── /                   Project container (NOT a git clone)
    ├── config.json             Rig identity and beads prefix
    ├── mayor/rig/              Canonical clone (beads live here, NOT an agent)
    │   └── .beads/             Rig-level beads (redirected to Dolt)
    ├── refinery/               Refinery agent home
    │   └── rig/                Worktree from mayor/rig
    ├── witness/                Witness agent home (no clone)
    ├── crew/                   Crew parent
    │   └── /          Human workspaces (full clones)
    └── polecats/               Polecats parent
        └── //   Worker worktrees from mayor/rig
```

**Note**: No per-directory CLAUDE.md or AGENTS.md is created. Only `~/gt/CLAUDE.md` (town-root identity anchor) exists on disk. Full context is injected by `gt prime` via SessionStart hook.

### Worktree Architecture

Polecats and refinery are git worktrees, not full clones. This enables fast spawning and shared object storage. The worktree base is `mayor/rig`:

```
// From polecat/manager.go - worktrees are based on mayor/rig
git worktree add -b polecat/- polecats/
```

Crew workspaces (`crew//`) are full git clones for human developers who need independent repos. Polecat sessions are ephemeral and benefit from worktree efficiency.

## Storage Layer: Dolt SQL Server

All beads data is stored in a single Dolt SQL Server process per town. There is no embedded Dolt fallback — if the server is down, `bd` fails fast with a clear error pointing to `gt dolt start`.

```
┌─────────────────────────────────┐
│ Dolt SQL Server (per town)      │
│ Port 3307, managed by daemon    │
│ Data: ~/gt/.dolt-data/          │
└──────────┬──────────────────────┘
           │ MySQL protocol
    ┌──────┼──────┬──────────┐
    │      │      │          │
 USE hq  USE gastown  USE beads  ...
```

Each rig database is a subdirectory under `.dolt-data/`. The daemon monitors the server on every heartbeat and auto-restarts on crash.

For write concurrency, all agents write directly to `main` using transaction discipline (`BEGIN` / `DOLT_COMMIT` / `COMMIT` atomically). This eliminates branch proliferation and ensures immediate cross-agent visibility.

See dolt-storage.md for full details.

## Beads Routing

The `routes.jsonl` file maps issue ID prefixes to rig locations (relative to town root):

```
{"prefix":"hq-","path":"."}
{"prefix":"gt-","path":"gastown/mayor/rig"}
{"prefix":"bd-","path":"beads/mayor/rig"}
```

Routes point to `mayor/rig` because that's where the canonical `.beads/` lives. This enables transparent cross-rig beads operations:

```
bd show hq-mayor    # Routes to town beads (~/.gt/.beads)
bd show gt-xyz      # Routes to gastown/mayor/rig/.beads
```

## Beads Redirects

Worktrees (polecats, refinery, crew) don't have their own beads databases. Instead, they use a `.beads/redirect` file that points to the canonical beads location:

```
polecats/alpha/.beads/redirect → ../../mayor/rig/.beads
refinery/rig/.beads/redirect → ../../mayor/rig/.beads
```

`ResolveBeadsDir()` follows redirect chains (max depth 3) with circular detection. This ensures all agents in a rig share a single beads database via the Dolt server.

## Merge Queue: Batch-then-Bisect

The refinery processes MRs through a batch-then-bisect merge queue (Bors-style). This is a core capability, not a pluggable strategy.

### How It Works

```
MRs waiting: [A, B, C, D]
    ↓
Batch: Rebase A..D as a stack on main
    ↓
Test tip: Run tests on D (tip of stack)
    ↓
If PASS: Fast-forward merge all 4 → done
If FAIL: Binary bisect → test B (midpoint)
    ↓
    If B passes: C or D broke it → bisect [C,D]
    If B fails: A or B broke it → bisect [A,B]
```

### Implementation Phases

| Phase | Bead | What | Status |
| --- | --- | --- | --- |
| 1: GatesParallel | gt-8b2i | Run test + lint concurrently per MR | In progress |
| 2: Batch-then-bisect | gt-i2vm | Bors-style batching with binary bisect | Blocked by Phase 1 |
| 3: Pre-verification | gt-lu84 | Polecats run tests before MR submission | Blocked by Phase 2 |

Gates (test command, lint, etc.) are pluggable. The batching strategy is core.

## Polecat Lifecycle: Self-Managed Completion

Polecats manage their own lifecycle end-to-end. The Witness observes but does NOT gate completion. This prevents the Witness from becoming a bottleneck.

### Polecat Completion Flow

```
Polecat finishes work
  → Push branch to remote
  → Submit MR (bd update --mr-ready)
  → Update bead status
  → Tear down worktree
  → Go idle (available for next assignment)
```

The Witness monitors for stuck/zombie polecats (no activity for extended period) and nudges or escalates. It does NOT process completion — that's the polecat's job.

## Data Plane Lifecycle

All beads data flows through a six-stage lifecycle managed by Dogs:

```
CREATE → LIVE → CLOSE → DECAY → COMPACT → FLATTEN
  │        │       │       │        │         │
 Dolt    active   done   DELETE   REBASE    SQUASH
 commit   work   bead    rows    commits   all history
                                 >7-30d    together to 1 commit
```

Stages 1-3 are automated today. Stages 4-6 are being shipped via Dog automation.

## Deployment Artifacts

Gas Town and Beads are distributed through multiple channels. Tag pushes (`v*`) trigger GitHub Actions release workflows that build and publish everything.

### Gas Town (`gt`)

| Channel | Artifact | Trigger |
| --- | --- | --- |
| **GitHub Releases** | Platform binaries (darwin/linux/windows, amd64/arm64) + checksums | GoReleaser on tag push |
| **Homebrew** | `brew install steveyegge/gastown/gt` — formula auto-updated on release | `update-homebrew` job pushes to `steveyegge/homebrew-gastown` |
| **npm** | `npx @gastown/gt` — wrapper that downloads the correct binary | OIDC trusted publishing (no token) |
| **Local build** | `go build -o $(go env GOPATH)/bin/gt ./cmd/gt` | Manual |

### Beads (`bd`)

| Channel | Artifact | Trigger |
| --- | --- | --- |
| **GitHub Releases** | Platform binaries + checksums | GoReleaser on tag push |
| **Homebrew** | `brew install steveyegge/beads/bd` | `update-homebrew` job |
| **npm** | `npx @beads/bd` — wrapper that downloads the correct binary | OIDC trusted publishing (no token) |
| **PyPI** | `beads-mcp` — MCP server integration | `publish-pypi` job with `PYPI_API_TOKEN` secret |
| **Local build** | `go build -o $(go env GOPATH)/bin/bd ./cmd/bd` | Manual |

### npm Authentication

Both repos use **OIDC trusted publishing** — no `NPM_TOKEN` secret needed. Authentication is handled by GitHub's OIDC provider.

### What the binary embeds

The Go binary is the primary distribution vehicle. It embeds:
* **Role templates** — Agent priming context, served by `gt prime`
* **Formula definitions** — Workflow molecules, served by `bd mol`
* **Doctor checks** — Health diagnostics, including migration checks
* **Default configs** — `daemon.json` lifecycle defaults, operational thresholds

This means upgrading the binary automatically propagates most fixes.


---

# ==============================================================================
# SECTION 4: Molecules
# URL: https://gastown.dev/docs/concepts/molecules
# ==============================================================================

Molecules are workflow templates that coordinate multi-step work in Gas Town.

## Molecule Lifecycle

```
Formula (source TOML)
      ─── "Ice-9"
           │
           ▼
bd cook
Protomolecule (frozen template)
      ─── Solid
           │
           ├─▶ bd mol pour ──▶ Mol (persistent)  ─── Liquid
           └─▶ bd mol wisp --root-only ──▶ Root Wisp (ephemeral)  ─── Vapor
```

**Root-only wisps** (default): Formula steps are NOT materialized as database rows. Only a single root wisp is created. Agents read steps inline from the embedded formula at prime time. This prevents wisp accumulation (~6,000+ rows/day → ~400/day).

**Poured wisps** (`pour = true`): Steps ARE materialized as sub-wisps with checkpoint recovery. If a session dies, completed steps remain closed and work resumes from the last checkpoint. Use pour for expensive, low-frequency workflows where losing progress would be costly (e.g., release workflows).

## Core Concepts

| Term | Description |
| --- | --- |
| **Formula** | Source TOML template defining workflow steps |
| **Protomolecule** | Frozen template ready for instantiation |
| **Molecule** | Active workflow instance (root wisp only) |
| **Wisp** | Ephemeral molecule for patrols and polecat work (never synced) |
| **Root-only** | Only root wisp created; steps read from embedded formula |
| **Pour** | Formula flag (`pour = true`); steps materialized as sub-wisps with checkpoint recovery |

## How Agents See Steps

Agents do NOT use `bd mol current` or `bd close ` for formula workflows. Instead, formula steps are rendered inline when the agent runs `gt prime`:

```
**Formula Checklist** (10 steps from mol-polecat-work):
### Step 1: Load context and verify assignment
Initialize your session and understand your assignment...

### Step 2: Set up working branch
Ensure you're on a clean feature branch...
```

The agent works through the checklist and runs `gt done` (polecats) or `gt patrol report` (patrol agents) when complete.

## Molecule Commands

### Beads Operations (bd)

```
# Formulas
bd formula list           # Available formulas
bd formula show        # Formula details
bd cook              # Formula → Proto

# Molecules (data operations)
bd mol list              # Available protos
bd mol show           # Proto details
bd mol wisp           # Create wisp (root-only by default)
bd mol bond          # Attach to existing mol
```

### Agent Operations (gt)

```
# Hook management
gt hook                  # What's on MY hook?
gt prime                 # Shows inline formula checklist
gt mol attach    # Pin molecule to bead
gt mol detach          # Unpin molecule from bead

# Patrol lifecycle
gt patrol new           # Create patrol wisp and hook it
gt patrol report --summary "..." # Close current patrol, start next cycle
```

## Polecat Workflow

Polecats receive work via their hook — a root wisp attached to an issue. They see the formula checklist inline when they run `gt prime` and work through each step in order.

### Polecat Workflow Summary

```
1. Spawn with work on hook
2. gt prime               # Shows formula checklist inline
3. Work through each step
4. Persist findings: bd update  --notes "..."
5. gt done                # Submit, nuke sandbox, exit
```

### Molecule Types

| Type | Storage | Use Case |
| --- | --- | --- |
| **Root-only Wisp** (`pour = false`) | `.beads/` (ephemeral) | Polecat work, patrols — high frequency, cheap steps |
| **Poured Wisp** (`pour = true`) | `.beads/` (sub-wisps) | Releases, long workflows — low frequency, expensive steps |

**Heuristic**: If you would curse losing the progress after a crash, set `pour = true`. High frequency + cheap steps = inline (default). Low frequency + expensive steps = pour.

## Patrol Workflow

Patrol agents (Deacon, Witness, Refinery) cycle through patrol formulas:

```
1. gt patrol new           # Create root-only patrol wisp
2. gt prime                # Shows patrol checklist inline
3. Work through each step
4. gt patrol report --summary "..."  # Close + start next cycle
```

`gt patrol report` atomically closes the current patrol root and spawns a new one for the next cycle.

## Best Practices

1. **Persist findings early** — `bd update  --notes "..."` before session death
2. **Run `gt done` when complete** — mandatory for polecats (pushes, submits to MQ, nukes)
3. **Use `gt patrol report`** — for patrol agents to cycle (replaces squash+new pattern)
4. **File discovered work** — `bd create` for bugs found, don't fix them yourself

---

# ==============================================================================
# SECTION 5: Escalation
# URL: https://gastown.dev/docs/design/escalation
# ==============================================================================

Reference for the unified escalation system in Gas Town.

## Overview

Gas Town agents escalate issues when automated resolution is not possible. Escalations are severity-routed, tracked as beads, and support stale detection with automatic re-escalation.

## Severity Levels

| Level | Priority | Description | Default Route |
| --- | --- | --- | --- |
| **CRITICAL** | P0 (urgent) | System-threatening, immediate attention | bead + mail + email + SMS |
| **HIGH** | P1 (high) | Important blocker, needs human soon | bead + mail + email |
| **MEDIUM** | P2 (normal) | Standard escalation, human at convenience | bead + mail mayor |

## Tiered Escalation Flow

```
Agent -> gt escalate -s  "description"
    |
    v
[Deacon receives]
    +-- resolves --> updates issue, re-slings work
    +-- cannot --> forwards to Mayor
        +-- resolves --> updates issue, re-slings
        +-- cannot --> forwards to Overseer --> resolves
```

Each tier can resolve OR forward. The chain is tracked via bead comments.

## Configuration

Config file: `~/gt/settings/escalation.json`

### Default Configuration

```json
{
  "type": "escalation",
  "version": 1,
  "routes": {
    "medium": ["bead", "mail:mayor"],
    "high": ["bead", "mail:mayor", "email:human"],
    "critical": ["bead", "mail:mayor", "email:human", "sms:human"]
  },
  "contacts": {
    "human_email": "",
    "human_sms": ""
  },
  "stale_threshold": "4h",
  "max_reescalations": 2
}
```

### Action Types

| Action | Format | Behavior |
| --- | --- | --- |
| `bead` | `bead` | Create escalation bead (always first, implicit) |
| `mail:` | `mail:mayor` | Send gt mail to target |
| `email:human` | `email:human` | Send email to `contacts.human_email` |
| `sms:human` | `sms:human` | Send SMS to `contacts.human_sms` |
| `slack` | `slack` | Post to `contacts.slack_webhook` |
| `log` | `log` | Write to escalation log file |

## Escalation Beads

Escalation beads use `type: escalation` with structured labels for tracking.

### Label Schema

| Label | Values | Purpose |
| --- | --- | --- |
| `severity:` | MEDIUM, HIGH, CRITICAL | Current severity |
| `source::` | plugin:rebuild-gt, patrol:deacon | What triggered it |
| `acknowledged:` | true, false | Has human acknowledged |
| `reescalated:` | true, false | Has been re-escalated |
| `reescalation_count:` | 0, 1, 2, ... | Times re-escalated |
| `original_severity:` | MEDIUM, HIGH | Initial severity |

## Commands

### gt escalate

Create a new escalation.

```
gt escalate -s  "Short description" \
  [-m "Detailed explanation"]
  [--source="plugin:rebuild-gt"]
```

Flags: `-s` severity (required), `-m` body, `--source` origin identifier, `--to` route to tier (deacon/mayor/overseer), `--dry-run`, `--json`.

### gt escalate ack

Acknowledge an escalation (prevents re-escalation).

```
gt escalate ack  [--note="Investigating"]
```

### gt escalate list

```
gt escalate list [--severity=...] [--stale] [--unacked] [--all] [--json]
```

### gt escalate stale

Re-escalate stale (unacked past `stale_threshold`) escalations. Bumps severity (MEDIUM->HIGH->CRITICAL), re-executes route, respects `max_reescalations`.

```
gt escalate stale [--dry-run]
```

### gt escalate close

```
gt escalate close  [--reason="Fixed in commit abc123"]
```

## When to Escalate

### Agents SHOULD escalate when:

* **System errors**: Database corruption, disk full, network failures
* **Security issues**: Unauthorized access attempts, credential exposure
* **Unresolvable conflicts**: Merge conflicts that cannot be auto-resolved
* **Ambiguous requirements**: Spec is unclear, multiple valid interpretations
* **Design decisions**: Architectural choices that need human judgment
* **Stuck loops**: Agent is stuck and cannot make progress
* **Gate timeouts**: Async conditions did not resolve in expected time

### Agents should NOT escalate for:

* **Normal workflow**: Regular work that can proceed without human input
* **Recoverable errors**: Transient failures that will auto-retry
* **Information queries**: Questions that can be answered from context

---

# ==============================================================================
# SECTION 6: Scheduler
# URL: https://gastown.dev/docs/design/scheduler
# ==============================================================================

Config-driven capacity-controlled polecat dispatch.

## Quick Start

Enable deferred dispatch and schedule some work:

```
# 1. Enable deferred dispatch (config-driven, no per-command flag)
gt config set scheduler.max_polecats 5

# 2. Schedule work via gt sling (auto-defers when max_polecats > 0)
gt sling gt-abc gastown                     # Single task bead
gt sling gt-abc gt-def gt-ghi gastown       # Batch task beads
gt sling hq-cv-abc                          # Convoy (schedules all tracked issues)
gt sling gt-epic-123                        # Epic (schedules all children)

# 3. Check what's scheduled
gt scheduler status
gt scheduler list

# 4. Dispatch manually (or let the daemon do it)
gt scheduler run
gt scheduler run --dry-run                  # Preview first
```

### Dispatch Modes

The `scheduler.max_polecats` config value controls dispatch behavior:

| Value | Mode | Behavior |
| --- | --- | --- |
| `-1` (default) | Direct dispatch | `gt sling` dispatches immediately, near-zero overhead |
| `0` | Direct dispatch | Same as `-1` — `gt sling` dispatches immediately |
| `N > 0` | Deferred dispatch | `gt sling` creates sling context bead, daemon dispatches |

No per-invocation flag needed. The same `gt sling` command adapts automatically.

### Common CLI

| Command | Description |
| --- | --- |
| `gt sling  ` | Sling bead (direct or deferred, per config) |
| `gt sling ... ` | Batch sling/schedule multiple beads |
| `gt sling ` | Sling/schedule all tracked issues in convoy |
| `gt sling ` | Sling/schedule all children of epic |
| `gt scheduler status` | Show scheduler state and capacity |
| `gt scheduler list` | List all scheduled beads by rig |
| `gt scheduler run` | Trigger dispatch manually |
| `gt scheduler pause` | Pause all dispatch town-wide |
| `gt scheduler resume` | Resume dispatch |
| `gt scheduler clear` | Remove beads from scheduler |

## Overview

The scheduler solves **back-pressure** and **capacity control** for batched polecat dispatch.

Without the scheduler, slinging N beads spawns N polecats simultaneously, exhausting API rate limits, memory, and CPU. The scheduler introduces a governor: beads enter a waiting state and the daemon dispatches them incrementally, respecting a configurable concurrency cap.

The scheduler integrates into the daemon heartbeat as **step 14** — after all agent health checks, lifecycle processing, and branch pruning. This ensures the system is healthy before spawning new work.

## Sling Context Beads

Scheduling state is stored on **separate ephemeral beads** called sling contexts. The work bead is never modified by the scheduler.

Each sling context bead:
* Is created via `bd create --ephemeral` with label `gt:sling-context`
* Has a `tracks` dependency pointing to the work bead
* Stores all scheduling parameters as JSON in its description
* Is closed when dispatch succeeds, the bead is cleared, or the circuit breaker trips

### Context Fields (JSON)

| Field | Type | Description |
| --- | --- | --- |
| `version` | int | Schema version (currently 1) |
| `work_bead_id` | string | The actual work bead being scheduled |
| `target_rig` | string | Destination rig name |
| `formula` | string | Formula to apply at dispatch (e.g., `mol-polecat-work`) |
| `args` | string | Natural language instructions for executor |
| `vars` | string | Newline-separated formula variables (`key=value`) |
| `enqueued_at` | RFC3339 | Timestamp of schedule |
| `merge` | string | Merge strategy: `direct`, `mr`, `local` |
| `convoy` | string | Convoy bead ID (set after auto-convoy creation) |
| `base_branch` | string | Override base branch for polecat worktree |
| `no_merge` | bool | Skip merge queue on completion |
| `account` | string | Claude Code account handle |
| `agent` | string | Agent/runtime override |
| `hook_raw_bead` | bool | Hook without default formula |
| `owned` | bool | Caller-managed convoy lifecycle |
| `mode` | string | Execution mode: `ralph` (fresh context per step) |
| `dispatch_failures` | int | Consecutive failure count (circuit breaker) |
| `last_failure` | string | Most recent dispatch error message |

## Bead State Machine

A sling context transitions through these states:

```
+------------------+
|                  |  v
| +----------+    dispatch ok    +--------+
| | schedule | CONTEXT |  ---------------->  | CLOSED |
| |-------->  | OPEN   |                   | (done) |
| +----------+                        +--------+
|                  |
| +-- 3 failures --> CLOSED (circuit-broken)
| +-- gt scheduler clear --> CLOSED (cleared)
```

| State | Representation | Trigger |
| --- | --- | --- |
| **SCHEDULED** | Open sling context bead | `scheduleBead()` |
| **DISPATCHED** | Closed sling context (reason: "dispatched") | `dispatchSingleBead()` success |
| **CIRCUIT-BROKEN** | Closed sling context (reason: "circuit-broken") | `dispatch_failures >= 3` |
| **CLEARED** | Closed sling context (reason: "cleared") | `gt scheduler clear` |

Key invariant: the work bead is **never modified** by the scheduler. All state lives on the sling context bead.

## Capacity Management

### Configuration

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `scheduler.max_polecats` | *int | `-1` | Max concurrent polecats (-1=direct, 0=disabled, N=deferred) |
| `scheduler.batch_size` | *int | `1` | Beads dispatched per heartbeat tick |
| `scheduler.spawn_delay` | string | `"0s"` | Delay between spawns (Dolt lock contention) |

## Circuit Breaker

The circuit breaker prevents permanently-failing beads from causing infinite retry loops.

| Property | Value |
| --- | --- |
| Threshold | `maxDispatchFailures = 3` |
| Counter | `dispatch_failures` field in sling context JSON |
| Break action | Close sling context (reason: "circuit-broken") |
| Reset | No automatic reset (manual intervention required) |

## Safety Properties

| Property | Mechanism |
| --- | --- |
| **Schedule idempotency** | Skip if open sling context already exists for work bead |
| **Work bead pristine** | Scheduler never modifies work bead description or labels |
| **Cross-rig guard** | Reject if bead prefix doesn't match target rig (unless `--force`) |
| **Dispatch serialization** | `flock(scheduler-dispatch.lock)` prevents double-dispatch |
| **Atomic scheduling** | Single `bd create --ephemeral` — no two-step write, no rollback |
| **Formula pre-cooking** | `bd cook` at schedule time catches bad protos before daemon dispatch loop |
| **Fresh state on save** | Dispatch re-reads state before saving to avoid clobbering concurrent pause |


---

# ==============================================================================
# SECTION 7: Convoys
# URL: https://gastown.dev/docs/concepts/convoy
# ==============================================================================

Convoys are the primary unit for tracking batched work across rigs.

## Quick Start

```
# Create a convoy tracking some issues
gt convoy create "Feature X" gt-abc gt-def --notify overseer

# Check progress
gt convoy status hq-cv-abc

# List active convoys (the dashboard)
gt convoy list

# See all convoys including landed ones
gt convoy list --all
```

## Concept

A **convoy** is a persistent tracking unit that monitors related issues across multiple rigs. When you kick off work - even a single issue - a convoy tracks it so you can see when it lands and what was included.

```
      🚚 Convoy (hq-cv-abc)
                │
    ┌───────────┼────────────┐
    │           │            │
    ▼           ▼            ▼
┌─────────┐ ┌─────────┐ ┌─────────┐
│ gt-xyz  │ │ gt-def  │ │ bd-abc  │
│ gastown │ │ gastown │ │ beads   │
└────┬────┘ └────┬────┘ └────┬────┘
     │           │           │
     ▼           ▼           ▼
┌─────────┐ ┌─────────┐ ┌─────────┐
│   nux   │ │ furiosa │ │  amber  │
│(polecat)│ │(polecat)│ │(polecat)│
└─────────┘ └─────────┘ └─────────┘
     │
     "the swarm" (ephemeral)
```

## Convoy vs Swarm

| Concept | Persistent? | ID | Description |
| --- | --- | --- | --- |
| **Convoy** | Yes | hq-cv-* | Tracking unit. What you create, track, get notified about. |
| **Swarm** | No | None | Ephemeral. "The workers currently on this convoy's issues." |
| **Stranded Convoy** | Yes | hq-cv-* | A convoy with ready work but no polecats assigned. Needs attention. |

When you "kick off a swarm", you're really:
1. Creating a convoy (the tracking unit)
2. Assigning polecats to the tracked issues
3. The "swarm" is just those polecats while they're working

When issues close, the convoy lands and notifies you. The swarm dissolves.

## Convoy Lifecycle

```
OPEN ──(all issues close)──► LANDED/CLOSED
  ↑                               │
  └──(add more issues)───────────┘ (auto-reopens)
```

| State | Description |
| --- | --- |
| `open` | Active tracking, work in progress |
| `closed` | All tracked issues closed, notification sent |

Adding issues to a closed convoy reopens it automatically.

## Commands

### Create a Convoy

```
# Track multiple issues across rigs
gt convoy create "Deploy v2.0" gt-abc bd-xyz --notify gastown/joe

# Track a single issue (still creates convoy for dashboard visibility)
gt convoy create "Fix auth bug" gt-auth-fix

# With default notification (from config)
gt convoy create "Feature X" gt-a gt-b gt-c
```

### Add Issues

```
# Add issues to existing convoy
gt convoy add hq-cv-abc gt-new-issue
gt convoy add hq-cv-abc gt-issue1 gt-issue2 gt-issue3

# Adding to closed convoy requires reopening first
bd update hq-cv-abc --status=open
gt convoy add hq-cv-abc gt-followup-fix
```

### Check Status

```
# Show issues and active workers (the swarm)
gt convoy status hq-abc

# All active convoys (the dashboard)
gt convoy status
```

Example output:

```
🚚 hq-cv-abc: Deploy v2.0
   Status: ●
   Progress: 2/4 completed
   Created: 2025-12-30T10:15:00-08:00

   Tracked Issues:
     ✓ gt-xyz: Update API endpoint [task]
     ✓ bd-abc: Fix validation [bug]
     ○ bd-ghi: Update docs [task]
     ○ gt-jkl: Deploy to prod [task]
```

### List Convoys (Dashboard)

```
# Active convoys (default) - the primary attention view
gt convoy list

# All convoys including landed
gt convoy list --all

# Only landed convoys
gt convoy list --status=closed

# JSON output
gt convoy list --json
```

## Notifications

When a convoy lands (all tracked issues closed), subscribers are notified.

## Auto-Convoy on Sling

When you sling a single issue without an existing convoy:

```
gt sling bd-xyz beads/amber
```

This auto-creates a convoy so all work appears in the dashboard:
1. Creates convoy: "Work: bd-xyz"
2. Tracks the issue
3. Assigns the polecat

Even "swarm of one" gets convoy visibility.

## Cross-Rig Tracking

Convoys live in town-level beads (`hq-cv-*` prefix) and can track issues from any rig:

```
# Track issues from multiple rigs
gt convoy create "Full-stack feature" \
  gt-frontend-abc \
  gt-backend-def \
  bd-docs-xyz
```

The `tracks` relation is:
* **Non-blocking**: doesn't affect issue workflow
* **Additive**: can add issues anytime
* **Cross-rig**: convoy in hq-_, issues in gt-_, bd-*, etc.

---

# ==============================================================================
# SECTION 8: Polecat Lifecycle
# URL: https://gastown.dev/docs/concepts/polecat-lifecycle
# ==============================================================================

Understanding the three-layer architecture of polecat workers.

## Overview

Polecats have three distinct lifecycle layers that operate independently. The key design principle: **polecats are persistent**. They survive work completion and can be reused across assignments.

## The Four Operating States

Polecats have four operating states:

| State | Description | How it happens |
| --- | --- | --- |
| **Working** | Actively doing assigned work | Normal operation after `gt sling` |
| **Idle** | Work completed, sandbox preserved for reuse | After `gt done` completes successfully |
| **Stalled** | Session stopped mid-work | Interrupted, crashed, or timed out without being nudged |
| **Zombie** | Completed work but failed to exit | `gt done` failed during cleanup |

**State cycle (happy path):**

```
┌──────────┐
┌───>│   IDLE   │<──── sync sandbox to main, clear hook
│    └────┬─────┘
│         │ gt sling
│         v
│    ┌──────────┐
│    │ WORKING  │<──── session active, hook set
│    └────┬─────┘
│         │ gt done
│         v
│    ┌──────────┐
└────┤   IDLE   │──── push branch, submit MR, go idle
     └──────────┘
```

No `nuke` in the happy path. Polecats cycle: IDLE -> WORKING -> IDLE.

**Key distinctions:**
* **Working** = actively executing. Session alive, hook set, doing work.
* **Idle** = work done, session killed, sandbox preserved. Ready for next `gt sling`.
* **Stalled** = supposed to be working, but stopped. Needs Witness intervention.
* **Zombie** = finished work, tried to exit, but cleanup failed. Stuck in limbo.

## The Persistent Polecat Model (gt-4ac)

**Polecats persist after completing work.** When a polecat finishes its assignment:
1. Signals completion via `gt done`
2. Pushes branch, submits MR to merge queue
3. Clears its hook (work is done)
4. Sets agent state to "idle"
5. Kills its own session
6. **Sandbox (worktree) is preserved for reuse**

The next `gt sling` reuses idle polecats before allocating new ones, avoiding the overhead of creating fresh worktrees.

### Why Persistent?

* **Faster turnaround** — Reusing an existing worktree is faster than creating one
* **Preserved identity** — The polecat's agent bead, CV chain, and work history persist
* **Simpler lifecycle** — No nuke/respawn cycle between assignments
* **Done means idle** — Session dies, sandbox lives, polecat awaits next assignment

## The Three Layers

### The Problem: Three Concepts Were Conflated

Early designs treated polecats as monolithic. This caused recurring issues:

| Concept | Lifecycle | Old behavior |
| --- | --- | --- |
| **Identity** | Long-lived (name, CV, ledger) | Destroyed on nuke |
| **Sandbox** | Per-assignment (worktree, branch) | Destroyed on nuke |
| **Session** | Ephemeral (Claude context window) | = polecat lifetime |

### Layer Summary

| Layer | Component | Lifecycle | Persistence |
| --- | --- | --- | --- |
| **Identity** | Agent bead, CV chain, work history | Permanent | Never dies |
| **Sandbox** | Git worktree, branch | Persistent across assignments | Created on first sling, reused thereafter |
| **Session** | Claude (tmux pane), context window | Ephemeral per step | Cycles per step/handoff |

### Identity Layer

The polecat's **identity is permanent**. It includes:
* Agent bead (created once, never deleted)
* CV chain (work history accumulates across all assignments)
* Mailbox and attribution record

Identity survives all session cycles and sandbox resets.

### Session Layer

The Claude session is **ephemeral**. It cycles frequently:
* After each molecule step (via `gt handoff`)
* On context compaction
* On crash/timeout
* After extended work periods

**Key insight:** Session cycling is **normal operation**, not failure. The polecat continues working—only the Claude context refreshes.

```
Session 1: Steps 1-2 → handoff
Session 2: Steps 3-4 → handoff
Session 3: Step 5 → gt done
```

All three sessions are the **same polecat**. The sandbox persists throughout.

### Sandbox Layer

The sandbox is the **git worktree**—the polecat's working directory:

```
~/gt/gastown/polecats/Toast/
```

This worktree:
* Exists from first `gt sling` and persists across assignments
* Survives all session cycles
* Is repaired (reset to fresh branch from main) when reused by `gt sling`
* Contains uncommitted work, staged changes, branch state during active work

The Witness never destroys sandboxes. Only explicit `gt polecat nuke` removes them.

### Slot Layer

The slot is the **name allocation** from the polecat pool:

```
# Pool: [Toast, Shadow, Copper, Ash, Storm...]
# Toast is allocated to work gt-abc
```

The slot:
* Determines the sandbox path (`polecats/Toast/`)
* Maps to a tmux session (`gt-gastown-Toast`)
* Appears in attribution (`gastown/polecats/Toast`)
* Persists until explicit nuke

## Correct Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│ gt sling                                                    │
│ → Find idle polecat OR allocate slot from pool (Toast)      │
│ → Create/repair sandbox (worktree on new branch)            │
│ → Start session (Claude in tmux)                            │
│ → Hook molecule to polecat                                  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Work Happens                                                │
│                                                             │
│ Session cycles happen here:                                 │
│ - gt handoff between steps                                  │
│ - Compaction triggers respawn                               │
│ - Crash → Witness respawns                                  │
│                                                             │
│ Sandbox persists through ALL session cycles                 │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ gt done (persistent model)                                  │
│ → Push branch to origin                                    │
│ → Submit work to merge queue (MR bead)                     │
│ → Set agent state to "idle"                                │
│ → Kill session                                              │
│                                                             │
│ Work now lives in MQ. Polecat is IDLE, not gone.           │
│ Sandbox preserved for reuse by next gt sling.               │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Refinery: merge queue                                       │
│ → Rebase and merge to target branch                        │
│   (main or integration branch — see below)                 │
│ → Close the issue                                          │
│ → If conflict: create task for available polecat           │
│                                                             │
│ Integration branch path:                                    │
│ → MRs from epic children merge to integration/              │
│ → When all children closed: land to main as one commit     │
└─────────────────────────────────────────────────────────────┘
```

## Polecat Identity

**Key insight:** Polecat _identity_ is permanent; sessions are ephemeral, sandboxes are persistent.

```
POLECAT IDENTITY (permanent)    SESSION (ephemeral)      SANDBOX (persistent)
├── CV chain                    ├── Claude instance      ├── Git worktree
├── Work history                ├── Context window       ├── Branch
├── Skills demonstrated         └── Dies on handoff      └── Repaired on reuse
└── Credit for work             or gt done              by gt sling
```

---

# ==============================================================================
# SECTION 9: Agent Provider Integration
# URL: https://gastown.dev/docs/agent-provider-integration
# ==============================================================================

How to integrate your agent CLI with Gas Town (and the upcoming Gas City).

## Integration Tiers

| Tier | Effort | What You Get | What You Provide |
| --- | --- | --- | --- |
| **0: Zero** | Nothing | Basic tmux orchestration | A CLI that runs in a terminal |
| **1: Preset** | JSON config file | Full lifecycle, resume, process detection | Preset entry in `agents.json` |
| **2: Hooks** | Settings file or plugin | Context injection, tool guards, mail delivery | Hook installer function |
| **3: Deep** | Code + scripts | Non-interactive mode, session forking, wrapper | Native API integration |

Most agent teams should target **Tier 1** first (15 minutes of work), then **Tier 2** if their CLI supports a hooks/plugin system.

## Tier 0: Zero Integration

**Any CLI that runs in a terminal works in Gas Town with zero changes.**

Gas Town launches agents in tmux sessions and communicates via `send-keys`. If your agent has a REPL or accepts text input, Gas Town can:
* Start it in a tmux pane
* Send work instructions via keystroke injection
* Detect liveness via `pane_current_command`
* Read output via `capture-pane`

## Tier 1: Preset Registration

**JSON config only. No code changes to Gas Town or your agent.**

A preset tells Gas Town everything it needs to launch, detect, resume, and communicate with your agent.

### Where to put the config

| Level | Path | Scope |
| --- | --- | --- |
| Town | `~/gt/settings/agents.json` | All rigs in the town |
| Rig | `~/gt//settings/agents.json` | Single rig only |
| Built-in | Compiled into `gt` binary | Ships with Gas Town |

### AgentPresetInfo field reference

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | Yes | Preset identifier (e.g., `"kiro"`) |
| `command` | string | Yes | CLI binary name or path (e.g., `"kiro"`) |
| `args` | string[] | Yes | Default args for autonomous mode (e.g., `["--yolo"]`) |
| `env` | map[string]string | No | Extra env vars to set (merged with GT_* vars) |
| `process_names` | string[] | No | Process names for tmux liveness detection |
| `session_id_env` | string | No | Env var the agent sets for session ID tracking |
| `resume_flag` | string | No | Flag or subcommand for resuming sessions |
| `resume_style` | string | No | `"flag"` or `"subcommand"` |
| `supports_hooks` | bool | No | Whether the agent has a hooks/plugin system |
| `supports_fork_session` | bool | No | Whether `--fork-session` is available |
| `non_interactive` | object | No | Settings for headless execution |
| `prompt_mode` | string | No | `"arg"` or `"none"`. Default: `"arg"` |
| `config_dir_env` | string | No | Env var for agent's config directory |
| `config_dir` | string | No | Top-level config dir name (e.g., `".kiro"`) |
| `hooks_provider` | string | No | Hooks framework identifier (for Tier 2) |
| `hooks_dir` | string | No | Directory for hooks/settings files |
| `hooks_settings_file` | string | No | Settings/plugin filename |
| `hooks_informational` | bool | No | `true` if hooks are instructions-only (not executable) |
| `ready_prompt_prefix` | string | No | Prompt string for readiness detection (e.g., `"❯ "`) |
| `ready_delay_ms` | int | No | Fallback delay for readiness (milliseconds) |
| `instructions_file` | string | No | Instruction file name (default: `"AGENTS.md"`) |

## Capability Matrix

Current agent capabilities at a glance:

| Agent | Hooks | Resume | Non-Interactive | Fork | Prompt Mode | Process Names |
| --- | --- | --- | --- | --- | --- | --- |
| Claude | Yes (settings.json) | `--resume` (flag) | Native | Yes | arg | node, claude |
| Gemini | Yes | `--resume` (flag) | `-p` | No | arg | gemini |
| Codex | No | `resume` (subcmd) | `exec` subcmd | No | none | codex |
| Cursor | No | `--resume` (flag) | `-p` | No | arg | cursor-agent |
| Auggie | No | `--resume` (flag) | No | No | arg | auggie |
| AMP | No | `threads continue` (subcmd) | No | No | arg | amp |
| OpenCode | Yes (plugin JS) | No | `run` subcmd | No | none | opencode, node, bun |

## Gas City Provider Contract (Forward-Looking)

Gas Town is being succeeded by Gas City, which formalizes the implicit provider interface into an explicit contract.

```typescript
interface AgentProvider {
  // --- Tier 1: Required ---
  // Lifecycle
  Start(workDir string, env map[string]string) -> Process
  IsReady() -> bool
  IsAlive() -> bool
  // Communication
  SendMessage(text string) -> error
  GetStatus() -> AgentStatus
  // Identity
  Name() -> string
  Version() -> string

  // --- Tier 2: Preferred ---
  // Context injection
  InjectContext(context string) -> error
  OnSessionStart(callback) -> void
  // Session management
  Resume(sessionID string) -> Process
  SessionID() -> string
  // Tool guards
  OnToolCall(callback) -> void

  // --- Tier 3: Advanced ---
  // Session forking
  ForkSession(sessionID string) -> Process
  // Non-interactive execution
  Exec(prompt string) -> Result
  // Cost tracking
  GetUsage() -> UsageReport
}
```

---

# ==============================================================================
# SECTION 10: Why These Features
# URL: https://gastown.dev/docs/why-these-features
# (Supplementary - not in original request but closely related to overview)
# ==============================================================================

Gas Town's architecture explained through enterprise AI challenges.

## The Problem

You have AI agents. Maybe a lot of them. They're writing code, reviewing PRs, fixing bugs, adding features. But you can't answer basic questions:
* **Who did what?** Which agent wrote this buggy code?
* **Who's reliable?** Which agents consistently deliver quality?
* **Who can do this?** Which agent should handle this Go refactor?
* **What's connected?** Does this frontend change depend on a backend PR?
* **What's the full picture?** How's the project doing across 12 repos?

## The Solution: A Work Ledger

Gas Town treats work as structured data. Every action is recorded. Every agent has a track record. Every piece of work has provenance.

### Feature: Entity Tracking and Attribution

Every Gas Town agent has a distinct identity. Every action is attributed:
```
Git commits: gastown/polecats/toast
Beads records: created_by: gastown/crew/joe
Event logs: actor: gastown/polecats/nux
```

### Feature: Work History (Agent CVs)

Every agent accumulates a work history:
```
# What has this agent done?
cd audit --actor=gastown/polecats/toast

# Success rate on Go projects
cd stats --actor=gastown/polecats/toast --tag=go
```

### Feature: Capability-Based Routing (Planned)

Work carries skill requirements. Agents have demonstrated capabilities (derived from their work history). Matching is automatic.

### Feature: Recursive Work Decomposition

Work decomposes naturally into epics, features, and tasks with automatic roll-ups.

### Feature: Cross-Project References

Explicit cross-project dependencies:
```
depends_on:
  beads://github/acme/backend/be-456  # Backend API
  beads://github/acme/shared/sh-789   # Shared types
```

### Feature: Federation (Planned)

Federated workspaces that reference each other across organizations.

### Feature: Validation and Quality Gates

Structured validation with attribution for quality control and audit trails.

### Feature: Real-Time Activity Feed

Work state as a real-time stream for debugging and status awareness.

## Design Philosophy

1. **Attribution is not optional.** Every action has an actor.
2. **Work is data.** Not just tickets - structured, queryable data.
3. **History matters.** Track records determine trust.
4. **Scale is assumed.** Multi-repo, multi-agent, multi-org from day one.
5. **Verification over trust.** Quality gates are first-class primitives.


---

# ==============================================================================
# SECTION 11: Propulsion Principle
# URL: https://gastown.dev/docs/concepts/propulsion-principle
# (Supplementary - covers the hooks/work-queue concept from original request)
# ==============================================================================

> **If you find something on your hook, YOU RUN IT.**

Gas Town is a steam engine. Agents are pistons. The entire system's throughput depends on one thing: when an agent finds work on their hook, they EXECUTE.

Why This Matters:

* There is no supervisor polling asking "did you start yet?"
* The hook IS your assignment - it was placed there deliberately
* Every moment you wait is a moment the engine stalls
* Other agents may be blocked waiting on YOUR output

## The Handoff Contract

When you were spawned, work was hooked for you. The system trusts that:
1. You will find it on your hook
2. You will understand what it is (`bd show` / `gt hook`)
3. You will BEGIN IMMEDIATELY

This isn't about being a good worker. This is physics. Steam engines don't run on politeness - they run on pistons firing. You are the piston.

## Molecule Navigation: Key Enabler

Molecules enable propulsion by providing clear waypoints. You don't need to memorize steps or wait for instructions - discover them:

### Orientation Commands

```
gt hook              # What's on my hook?
bd mol current       # Where am I in the molecule?
bd ready             # What step is next?
bd show              # What does this step require?
```

### The Propulsion Loop

```
1. gt hook                       # What's hooked?
2. bd mol current                # Where am I?
3. Execute step
4. bd close  --continue  # Close and advance
5. GOTO 2
```

## Startup Behavior

1. Check hook (`gt hook`)
2. Work hooked -> EXECUTE immediately
3. Hook empty -> Check mail for attached work
4. Nothing anywhere -> ERROR: escalate to Witness

**Note:** "Hooked" means work assigned to you. This triggers autonomous mode even if no molecule is attached.

## The Capability Ledger

Every completion is recorded. Every handoff is logged. Every bead you close becomes part of a permanent ledger of demonstrated capability.

* Your work is visible
* Redemption is real (consistent good work builds over time)
* Every completion is evidence that autonomous execution works
* Your CV grows with every completion

---

# ==============================================================================
# SECTION 12: Identity
# URL: https://gastown.dev/docs/concepts/identity
# (Supplementary - covers identity concepts referenced in original request)
# ==============================================================================

Canonical format for agent identity in Gas Town.

## Why Identity Matters

When you deploy AI agents at scale, anonymous work creates real problems:
* **Debugging:** "The AI broke it" isn't actionable. _Which_ AI?
* **Quality tracking:** You can't improve what you can't measure.
* **Compliance:** Auditors ask "who approved this code?" - you need an answer.
* **Performance management:** Some agents are better than others at certain tasks.

## BD_ACTOR Format Convention

The `BD_ACTOR` environment variable identifies agents in slash-separated path format. This is set automatically when agents are spawned and used for all attribution.

### Format by Role Type

| Role Type | Format | Example |
| --- | --- | --- |
| **Mayor** | `mayor` | `mayor` |
| **Deacon** | `deacon` | `deacon` |
| **Witness** | `{rig}/witness` | `gastown/witness` |
| **Refinery** | `{rig}/refinery` | `gastown/refinery` |
| **Crew** | `{rig}/crew/{name}` | `gastown/crew/joe` |
| **Polecat** | `{rig}/polecats/{name}` | `gastown/polecats/toast` |

## Attribution Model

Gas Town uses three fields for complete provenance:

### Git Commits

```
GIT_AUTHOR_NAME="gastown/crew/joe"      # Who did the work (agent)
GIT_AUTHOR_EMAIL="steve@example.com"     # Who owns the work (overseer)
```

### Beads Records

```json
{
  "id": "gt-xyz",
  "created_by": "gastown/crew/joe",
  "updated_by": "gastown/witness"
}
```

### Event Logging

```json
{
  "ts": "2025-01-15T10:30:00Z",
  "type": "sling",
  "actor": "gastown/crew/joe",
  "payload": {
    "bead": "gt-xyz",
    "target": "gastown/polecats/toast"
  }
}
```

## Audit Queries

Attribution enables powerful audit queries:

```
# All work by an agent
cd audit --actor=gastown/crew/joe

# All work in a rig
cd audit --actor=gastown/*

# All polecat work
cd audit --actor=*/polecats/*

# Git history by agent
git log --author="gastown/crew/joe"
```

## CV and Skill Accumulation

### Human Identity is Global

The global identifier is your **email** - it's already in every git commit.

```
steve@example.com              <- global identity (from git author)
├── Town A (home)              <- workspace
│   ├── gastown/crew/joe       <- agent executor
│   └── gastown/polecats/toast <- agent executor
└── Town B (work)              <- workspace
    └── acme/polecats/nux      <- agent executor
```

### Agent vs Owner

| Field | Scope | Purpose |
| --- | --- | --- |
| `BD_ACTOR` | Local (town) | Agent attribution for debugging |
| `GIT_AUTHOR_EMAIL` | Global | Human identity for CV |
| `created_by` | Local | Who created the bead |
| `owner` | Global | Who owns the work |

**Agents execute. Humans own.** The polecat name in `completed-by: gastown/polecats/toast` is executor attribution. The CV credits the human owner (`steve@example.com`).

---

# ==============================================================================
# SECTION 13: Integration Branches
# URL: https://gastown.dev/docs/concepts/integration-branches
# (Supplementary - covers formula/epic workflow concepts)
# ==============================================================================

Group epic work on a shared branch, land to main as a unit.

Integration branches provide end-to-end support for epic-scoped work across the Gas Town pipeline.

## Workflow

1. **Create the epic and its children.** Structure your work as an epic with child tasks.
2. **Create the integration branch.**
   ```
   gt mq integration create gt-auth-epic
   ```
3. **Create a convoy to track the work.**
   ```
   gt convoy create "Auth overhaul" gt-auth-tokens gt-auth-sessions gt-auth-middleware
   ```
4. **Sling the first wave.**
   ```
   gt sling gt-auth-tokens gastown --no-convoy
   gt sling gt-auth-sessions gastown --no-convoy
   ```
5. **Polecats process the work.** Each polecat spawns its worktree from the integration branch.
6. **Refinery merges to the integration branch.** Instead of merging to main.
7. **Track progress via the convoy.**
   ```
   gt convoy status hq-cv-abc
   ```
8. **Sling the next wave.** When dependencies unblock.
9. **Land when complete.**
   ```
   gt mq integration land gt-auth-epic
   ```

## Concept

### The Problem

Without integration branches, epic work lands piecemeal:
```
Child A ──► MR ──► main (lands Tuesday)
Child B ──► MR ──► main (lands Wednesday, breaks A's work)
Child C ──► MR ──► main (lands Thursday, depends on A+B together)
```

### The Solution

Integration branches batch epic work on a shared branch, then land atomically:
```
Epic: gt-auth-epic
         │
    ┌─────┼─────┐
    │     │     │
 Child A Child B Child C
    │     │     │
    ▼     ▼     ▼
  MR A   MR B   MR C
    │     │     │
    └─────┼─────┘
          ▼
   integration/gt-auth-epic (shared branch)
          │
          ▼
   gt mq integration land
   base branch (main)
   (single merge commit)
```

### With vs Without

| Aspect | Without | With Integration Branch |
| --- | --- | --- |
| MR target | main | `integration/{epic}` |
| Land timing | Each MR lands independently | All MRs land together |
| Cross-child deps | Risky—depends on merge order | Safe—children share a branch |
| Rollback | Revert individual commits | Revert one merge commit |
| CI on main | Runs per-MR | Runs once on combined work |

## Commands

### `gt mq integration create `

Create an integration branch for an epic.

Flags: `--branch` (override branch name template), `--base-branch` (create from this branch instead of default).

### `gt mq integration status `

Display integration branch status for an epic. Flags: `--json`.

### `gt mq integration land `

Merge an epic's integration branch back to its base branch.

Flags: `--force` (land even if some MRs still open), `--skip-tests`, `--dry-run`.

## Safety Guardrails

Integration branch landing is protected by a three-layer defense:
1. **Formula and Role Instructions** — explicitly forbid landing via raw git commands
2. **Pre-Push Hook** — detects integration branch content pushed to default branch
3. **Code path** — `gt mq integration land` sets bypass env var `GT_INTEGRATION_LAND=1`

---

# ==============================================================================
# SECTION 14: Reference (CLI Reference)
# URL: https://gastown.dev/docs/reference
# (Supplementary - covers formulas, hooks, dashboard commands)
# ==============================================================================

Technical reference for Gas Town internals.

## Beads Routing

Gas Town routes beads commands based on issue ID prefix:

```
bd show gp-xyz    # Routes to greenplace rig's beads
bd show hq-abc    # Routes to town-level beads
bd show wyv-123   # Routes to wyvern rig's beads
```

Debug routing: `BD_DEBUG_ROUTING=1 bd show `

## Configuration

### Rig Config (`config.json`)

```json
{
  "type": "rig",
  "name": "myproject",
  "git_url": "https://github.com/...",
  "default_branch": "main",
  "beads": {
    "prefix": "mp"
  }
}
```

### Settings (`settings/config.json`)

```json
{
  "theme": "desert",
  "merge_queue": {
    "enabled": true,
    "run_tests": true,
    "test_command": "go test ./...",
    "on_conflict": "assign_back",
    "delete_merged_branches": true,
    "integration_branch_polecat_enabled": true,
    "integration_branch_refinery_enabled": true,
    "integration_branch_template": "integration/{title}",
    "integration_branch_auto_land": false
  }
}
```

## Formula Format

```toml
formula = "name"
type = "workflow"     # workflow | expansion | aspect
version = 1
description = "..."

[vars.feature]
description = "..."
required = true

[[steps]]
id = "step-id"
title = "{{feature}}"
description = "..."
needs = ["other-step"]   # Dependencies
```

**Composition:**

```toml
extends = ["base-formula"]

[compose]
aspects = ["cross-cutting"]

[[compose.expand]]
target = "step-id"
with = "macro-formula"
```

## Molecule Lifecycle

**Summary**: Formula (TOML) --`bd cook`--> Protomolecule --`bd mol pour`--> Mol (persistent) or Wisp (ephemeral) --`bd squash`--> Digest.

| Operation | bd (data) | gt (agent) |
| --- | --- | --- |
| Cook/pour/wisp | `bd cook`, `bd mol pour/wisp` | — |
| Squash/burn | `bd mol squash/burn ` | `gt mol squash/burn` (attached) |
| Navigate | `bd mol current`, `bd mol show` | `gt hook`, `gt mol current` |
| Attach | — | `gt mol attach/detach` |

## Environment Variables

### Core Variables (All Agents)

| Variable | Purpose | Example |
| --- | --- | --- |
| `GT_ROLE` | Agent role type | `mayor`, `witness`, `polecat`, `crew` |
| `GT_ROOT` | Town root directory | `/home/user/gt` |
| `BD_ACTOR` | Agent identity for attribution | `gastown/polecats/toast` |
| `GIT_AUTHOR_NAME` | Commit attribution (same as BD_ACTOR) | `gastown/polecats/toast` |
| `BEADS_DIR` | Beads database location | `/home/user/gt/gastown/.beads` |

### Environment by Role

| Role | Key Variables |
| --- | --- |
| **Mayor** | `GT_ROLE=mayor`, `BD_ACTOR=mayor` |
| **Deacon** | `GT_ROLE=deacon`, `BD_ACTOR=deacon` |
| **Boot** | `GT_ROLE=deacon/boot`, `BD_ACTOR=deacon-boot` |
| **Witness** | `GT_ROLE=witness`, `GT_RIG=`, `BD_ACTOR=/witness` |
| **Refinery** | `GT_ROLE=refinery`, `GT_RIG=`, `BD_ACTOR=/refinery` |
| **Polecat** | `GT_ROLE=polecat`, `GT_RIG=`, `GT_POLECAT=`, `BD_ACTOR=/polecats/` |
| **Crew** | `GT_ROLE=crew`, `GT_RIG=`, `GT_CREW=`, `BD_ACTOR=/crew/` |

## CLI Reference

### Town Management

```
gt install [path]          # Create town
gt install --git           # With git init
gt doctor                  # Health check
gt doctor --fix            # Auto-repair
```

### Configuration

```
gt config agent list [--json]          # List all agents
gt config agent get               # Show agent configuration
gt config agent set    # Create or update custom agent
gt config agent remove          # Remove custom agent
gt config default-agent [name]          # Get or set town default agent
```

**Built-in agents**: `claude`, `gemini`, `codex`, `cursor`, `auggie`, `amp`

### Rig Management

```
gt rig add 
gt rig list
gt rig remove 
```

### Convoy Management (Primary Dashboard)

```
gt convoy list                          # Dashboard of active convoys
gt convoy status [convoy-id]            # Show progress
gt convoy create "name" [issues...]     # Create convoy tracking issues
gt convoy list --all                    # Include landed convoys
gt convoy list --status=closed          # Only landed convoys
```

### Work Assignment

```
gt sling    # Assign to polecat
gt sling   --on  # With workflow template
gt sling    --agent codex  # Override runtime
```

### Communication

```
gt mail inbox
gt mail read 
gt mail send  -s "Subject" -m "Body"
gt mail send --human -s "..."        # To overseer
```

### Escalation

```
gt escalate "topic"                    # Default: MEDIUM severity
gt escalate -s CRITICAL "msg"          # Urgent, immediate attention
gt escalate -s HIGH "msg"              # Important blocker
```

### Sessions

```
gt handoff                             # Request cycle (context-aware)
gt handoff --shutdown                  # Terminate (polecats)
gt session stop /
gt peek                         # Check health
gt nudge                  "message"    # Send message to agent
gt seance                              # List discoverable predecessor sessions
gt seance --talk                     # Talk to predecessor (full context)
```

### Emergency

```
gt stop --all                          # Kill all sessions
gt stop --rig                    # Kill rig sessions
```

### Merge Queue (MQ)

```
gt mq list [rig]                       # Show the merge queue
gt mq next [rig]                       # Show highest-priority merge request
gt mq submit                           # Submit current branch to merge queue
gt mq status                    # Show detailed merge request status
gt mq retry                     # Retry a failed merge request
gt mq reject                    # Reject a merge request
```

## Beads Commands (bd)

```
bd ready                               # Work with no blockers
bd list --status=open
bd list --status=in_progress
bd show 
bd create --title="..." --type=task
bd update  --status=in_progress
bd close 
bd dep add     # child depends on parent
```

## Patrol Agents

Deacon, Witness, and Refinery run continuous patrol loops using wisps:

| Agent | Patrol Molecule | Responsibility |
| --- | --- | --- |
| **Deacon** | `mol-deacon-patrol` | Agent lifecycle, plugin execution, health checks |
| **Witness** | `mol-witness-patrol` | Monitor polecats, nudge stuck workers |
| **Refinery** | `mol-refinery-patrol` | Process merge queue, review MRs, check integration branches |

```
1. gt patrol new           # Create root-only wisp
2. gt prime                # Shows patrol checklist inline
3. Work through each step
4. gt patrol report --summary "..."  # Close + start next cycle
```

## Formula Invocation Patterns

**CRITICAL**: Different formula types require different invocation methods.

### Workflow Formulas (sequential steps, single polecat)

Examples: `shiny`, `shiny-enterprise`, `mol-polecat-work`

```
gt sling  --on 
gt sling shiny-enterprise --on gt-abc123 gastown
```

### Convoy Formulas (parallel legs, multiple polecats)

Examples: `code-review`

**DO NOT use `gt sling` for convoy formulas!** It fails with "convoy type not supported".

```
# Correct invocation - use gt formula run:
gt formula run code-review --pr=123
gt formula run code-review --files="src/*.go"
# Dry run to preview:
gt formula run code-review --pr=123 --dry-run
```

---

# ==============================================================================
# NOTES ON MISSING PAGES
# ==============================================================================
#
# The following requested URLs do not exist on gastown.dev:
#
# 2. /docs/installing - NOT FOUND. No installing/quickstart page exists on the site.
# 3. /docs/quickstart - NOT FOUND.
# 9. /docs/concepts/wasteland - NOT FOUND. No such concept page exists.
# 10. /docs/concepts/hooks - NOT FOUND. Hooks are documented in the reference page.
# 14. /docs/concepts/formulas - NOT FOUND. Formulas are covered in molecules and reference pages.
# 15. /docs/concepts/dashboard - NOT FOUND. The "dashboard" is `gt convoy list`.
#
# The sitemap (https://gastown.dev/sitemap.xml) contains these additional pages
# that were not in the original request but may be of interest:
# - /docs/CLEANUP - Comprehensive cleanup command catalog
# - /docs/proxy-server - Sandboxed execution proxy for containers
# - /docs/why-these-features - Architecture rationale
# - /docs/reference - Full CLI reference
# - /docs/concepts/identity - Agent identity format
# - /docs/concepts/propulsion-principle - The "if hooked, run it" principle
# - /docs/concepts/integration-branches - Epic-scoped branch management
# - /docs/agent-provider-integration - Agent CLI integration guide
# - Multiple /docs/design/* pages (dolt-storage, mail-protocol, federation, etc.)

