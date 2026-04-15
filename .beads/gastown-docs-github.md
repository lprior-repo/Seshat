# Gas Town Documentation (Scraped from GitHub)

Source: https://github.com/gastownhall/gastown (main branch)
Scraped: 2026-04-05

---

# Section 1: README.md
Source: https://github.com/gastownhall/gastown/blob/main/README.md

# Gas Town

**Multi-agent orchestration system for Claude Code, GitHub Copilot, and other AI agents with persistent work tracking**

## Overview

Gas Town is a workspace manager that lets you coordinate multiple AI coding agents (Claude Code, GitHub Copilot, Codex, Gemini, and others) working on different tasks. Instead of losing context when agents restart, Gas Town persists work state in git-backed hooks, enabling reliable multi-agent workflows.

### What Problem Does This Solve?

| Challenge | Gas Town Solution |
| --- | --- |
| Agents lose context on restart | Work persists in git-backed hooks |
| Manual agent coordination | Built-in mailboxes, identities, and handoffs |
| 4-10 agents become chaotic | Scale comfortably to 20-30 agents |
| Work state lost in agent memory | Work state stored in Beads ledger |

## Architecture

```
graph TB
    Mayor[The Mayor<br/>AI Coordinator]
    Town[Town Workspace<br/>~/gt/]

    Town --> Mayor
    Town --> Rig1[Rig: Project A]
    Town --> Rig2[Rig: Project B]

    Rig1 --> Crew1[Crew Member<br/>Your workspace]
    Rig1 --> Hooks1[Hooks<br/>Persistent storage]
    Rig1 --> Polecats1[Polecats<br/>Worker agents]

    Rig2 --> Crew2[Crew Member]
    Rig2 --> Hooks2[Hooks]
    Rig2 --> Polecats2[Polecats]

    Hooks1 -.git worktree.-> GitRepo1[Git Repository]
    Hooks2 -.git worktree.-> GitRepo2[Git Repository]
```

## Core Concepts

### The Mayor

Your primary AI coordinator. The Mayor is a Claude Code instance with full context about your workspace, projects, and agents. **Start here** - just tell the Mayor what you want to accomplish.

### Town

Your workspace directory (e.g., `~/gt/`). Contains all projects, agents, and configuration.

### Rigs

Project containers. Each rig wraps a git repository and manages its associated agents.

### Crew Members

Your personal workspace within a rig. Where you do hands-on work.

### Polecats

Worker agents with persistent identity but ephemeral sessions. Spawned for tasks, sessions end on completion, but identity and work history persist.

### Hooks

Git worktree-based persistent storage for agent work. Survives crashes and restarts.

### Convoys

Work tracking units. Bundle multiple beads that get assigned to agents. Convoys labeled `mountain` get autonomous stall detection and smart skip logic for epic-scale execution.

### Beads Integration

Git-backed issue tracking system that stores work state as structured data.

**Bead IDs** (also called **issue IDs**) use a prefix + 5-character alphanumeric format (e.g., `gt-abc12`, `hq-x7k2m`). The prefix indicates the item's origin or rig.

### Molecules

Workflow templates that coordinate multi-step work. Formulas (TOML definitions) are instantiated as molecules with tracked steps. Two modes: root-only wisps (steps materialized at runtime, lightweight) and poured wisps (steps materialized as sub-wisps with checkpoint recovery).

### Monitoring: Witness, Deacon, Dogs

A three-tier watchdog system keeps agents healthy:

- **Witness** - Per-rig lifecycle manager. Monitors polecats, detects stuck agents, triggers recovery, manages session cleanup.
- **Deacon** - Background supervisor running continuous patrol cycles across all rigs.
- **Dogs** - Infrastructure workers dispatched by the Deacon for maintenance tasks (e.g., Boot for triage).

### Refinery

Per-rig merge queue processor. When polecats complete work via `gt done`, the Refinery batches merge requests, runs verification gates, and merges to main using a Bors-style bisecting queue.

### Escalation

Severity-routed issue escalation. Agents that hit blockers escalate via `gt escalate`, which creates tracked beads routed through the Deacon, Mayor, and (if needed) Overseer. Severity levels: CRITICAL (P0), HIGH (P1), MEDIUM (P2).

### Scheduler

Config-driven capacity governor for polecat dispatch. Prevents API rate limit exhaustion by batching dispatch under configurable concurrency limits.

### Seance

Session discovery and continuation. Discovers previous agent sessions via `.events.jsonl` logs, enabling agents to query their predecessors for context and decisions from earlier work.

### Wasteland

Federated work coordination network linking Gas Towns through DoltHub. Rigs post wanted items, claim work from other towns, submit completion evidence, and earn portable reputation via multi-dimensional stamps.

## Installation

### Prerequisites

- **Go 1.25+**
- **Git 2.25+**
- **Dolt 1.82.4+**
- **beads (bd) 0.55.4+**
- **sqlite3**
- **tmux 3.0+**
- **Claude Code CLI** (default runtime)
- **Codex CLI** (optional runtime)
- **GitHub Copilot CLI** (optional runtime)

### Setup

```
brew install gastown
# or: npm install -g @gastown/gt
# or: go install github.com/steveyegge/gastown/cmd/gt@latest

gt install ~/gt --git
cd ~/gt
gt rig add myproject https://github.com/you/repo.git
gt crew add yourname --rig myproject
cd myproject/crew/yourname
gt mayor attach
```

## Key Commands

### Workspace Management
```
gt install <path>           # Initialize workspace
gt rig add <name> <repo>    # Add project
gt rig list                 # List projects
gt crew add <name> --rig <rig>  # Create crew workspace
```

### Agent Operations
```
gt agents                   # List active agents
gt sling <bead-id> <rig>    # Assign work to agent
gt mayor attach             # Start Mayor session
gt prime                    # Context recovery
gt feed                     # Real-time activity feed (TUI)
```

Built-in agent presets: `claude`, `gemini`, `codex`, `cursor`, `auggie`, `amp`, `opencode`, `copilot`, `pi`, `omp`

### Convoy (Work Tracking)
```
gt convoy create <name> [issues...]
gt convoy list
gt convoy show [id]
gt convoy add <convoy-id> <issue-id...>
```

### Monitoring & Health
```
gt escalate -s HIGH "description"
gt escalate list
gt scheduler status
gt seance
```

---

# Section 2: docs/INSTALLING.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/INSTALLING.md

## Installing Gas Town

Complete setup guide for Gas Town multi-agent orchestrator.

## Prerequisites

### Required

| Tool | Version | Check | Install |
| --- | --- | --- | --- |
| **Go** | 1.24+ | `go version` | golang.org |
| **Git** | 2.20+ | `git --version` | See below |
| **Dolt** | >= 1.82.4 | `dolt version` | dolthub/dolt |
| **Beads** | >= 0.55.4 | `bd version` | `go install github.com/steveyegge/beads/cmd/bd@latest` |

### Optional (for Full Stack Mode)

| Tool | Version | Check | Install |
| --- | --- | --- | --- |
| **tmux** | 3.0+ | `tmux -V` | See below |
| **Claude Code** (default) | latest | `claude --version` | claude.ai/claude-code |
| **Codex CLI** (optional) | latest | `codex --version` | developers.openai.com/codex/cli |
| **OpenCode CLI** (optional) | latest | `opencode --version` | opencode.ai |
| **GitHub Copilot CLI** (optional) | latest | `copilot --version` | cli.github.com |

## Installing Gas Town

### Step 1: Install the Binaries

```
go install github.com/steveyegge/gastown/cmd/gt@latest
go install github.com/steveyegge/beads/cmd/bd@latest
gt version
bd version
```

### Step 2: Create Your Workspace

```
gt install ~/gt --shell
```

### Step 3: Add a Project (Rig)

```
gt rig add myproject https://github.com/you/repo.git
```

### Step 4: Verify Installation

```
cd ~/gt
gt enable
gt git-init
gt up
gt doctor
gt status
```

## Minimal Mode vs Full Stack Mode

### Minimal Mode (No Daemon)
Run individual runtime instances manually. Gas Town only tracks state.

### Full Stack Mode (With Daemon)
Agents run in tmux sessions. Daemon manages lifecycle automatically.

### Choosing Roles

| Configuration | Roles | Use Case |
| --- | --- | --- |
| **Polecats only** | Workers | Manual spawning, no monitoring |
| **+ Witness** | + Monitor | Automatic lifecycle, stuck detection |
| **+ Refinery** | + Merge queue | MR review, code integration |
| **+ Mayor** | + Coordinator | Cross-project coordination |

---

# Section 3: docs/glossary.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/glossary.md

## Gas Town Glossary

Gas Town is an agentic development environment for managing multiple Claude Code instances simultaneously using the `gt` and `bd` (Beads) binaries, coordinated with tmux in git-managed directories.

## Core Principles

### MEOW (Molecular Expression of Work)
Breaking large goals into detailed instructions for agents. Supported by Beads, Epics, Formulas, and Molecules.

### GUPP (Gas Town Universal Propulsion Principle)
"If there is work on your Hook, YOU MUST RUN IT."

### NDI (Nondeterministic Idempotence)
The overarching goal ensuring useful outcomes through orchestration of potentially unreliable processes.

## Environments

### Town
The management headquarters (e.g., `~/gt/`).

### Rig
A project-specific Git repository under Gas Town management.

## Town-Level Roles

### Mayor
Chief-of-staff agent responsible for initiating Convoys, coordinating work distribution, and notifying users of important events.

### Deacon
Daemon beacon running continuous Patrol cycles. The system's watchdog.

### Dogs
The Deacon's crew of maintenance agents.

### Boot (the Dog)
A special Dog that checks the Deacon every 5 minutes.

## Rig-Level Roles

### Polecat
Worker agents with persistent identity but ephemeral sessions.

### Refinery
Manages the Merge Queue for a Rig.

### Witness
Patrol agent that oversees Polecats and the Refinery within a Rig.

### Crew
Long-lived, named agents for persistent collaboration.

## Work Units

### Bead
Git-backed atomic work unit stored in Dolt.

### Formula
TOML-based workflow source template.

### Protomolecule
A template class for instantiating Molecules.

### Molecule
Durable chained Bead workflows.

### Wisp
Ephemeral Beads destroyed after runs.

### Hook
A special pinned Bead for each agent. The agent's primary work queue.

## Workflow Commands

### Convoy
Primary work-order wrapping related Beads.

### Slinging
Assigning work to agents via `gt sling`.

### Nudging
Real-time messaging between agents with `gt nudge`.

### Handoff
Agent session refresh via `/handoff`.

### Seance
Communicating with previous sessions via `gt seance`.

### Patrol
Ephemeral loop maintaining system heartbeat.

---

# Section 4: docs/design/architecture.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/design/architecture.md

# Gas Town Architecture

Technical architecture for Gas Town multi-agent workspace management.

## Two-Level Beads Architecture

| Level | Location | Prefix | Purpose |
| --- | --- | --- | --- |
| **Town** | `~/gt/.beads/` | `hq-*` | Cross-rig coordination, Mayor mail, agent identity |
| **Rig** | `<rig>/mayor/rig/.beads/` | project prefix | Implementation work, MRs, project issues |

## Agent Taxonomy

### Town-Level Agents (Cross-Rig)

| Agent | Role | Persistence |
| --- | --- | --- |
| **Mayor** | Global coordinator | Persistent |
| **Deacon** | Daemon beacon | Persistent |
| **Boot** | Deacon watchdog | Ephemeral |
| **Dogs** | Long-running workers | Variable |

### Rig-Level Agents (Per-Project)

| Agent | Role | Persistence |
| --- | --- | --- |
| **Witness** | Monitors polecat health | Persistent |
| **Refinery** | Processes merge queue | Persistent |
| **Polecats** | Workers with persistent identity | Persistent identity, ephemeral sessions |
| **Crew** | Human workspaces | Persistent |

## Directory Structure

```
~/gt/                           Town root
├── .beads/                     Town-level beads (hq-* prefix)
├── .dolt-data/                 Centralized Dolt data directory
├── daemon/                     Daemon runtime state
├── deacon/                     Deacon workspace
├── mayor/                      Mayor agent home
├── settings/                   Town-level settings
├── directives/                 Town-level role directives
├── formula-overlays/           Town-level formula overlays
└── <rig>/                      Project container
    ├── mayor/rig/              Canonical clone (beads live here)
    ├── refinery/               Refinery agent home
    ├── witness/                Witness agent home
    ├── crew/                   Crew parent
    └── polecats/               Polecats parent
```

## Storage Layer: Dolt SQL Server

All beads data is stored in a single Dolt SQL Server process per town. No embedded Dolt fallback.

## Merge Queue: Batch-then-Bisect

The refinery processes MRs through a batch-then-bisect merge queue (Bors-style).

## Polecat Lifecycle: Self-Managed Completion

Polecats manage their own lifecycle end-to-end. The Witness observes but does NOT gate completion.

## Data Plane Lifecycle

All beads data flows through a six-stage lifecycle: CREATE -> LIVE -> CLOSE -> DECAY -> COMPACT -> FLATTEN

## Role Directives and Formula Overlays

Operators can customize agent behavior at the town or rig level without modifying the Go binary. Per-role Markdown files and TOML formula overlays.

---

# Section 5: docs/design/escalation.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/design/escalation.md

## Gas Town Escalation Protocol

## Severity Levels

| Level | Priority | Description | Default Route |
| --- | --- | --- | --- |
| **CRITICAL** | P0 (urgent) | System-threatening | bead + mail + email + SMS |
| **HIGH** | P1 (high) | Important blocker | bead + mail + email |
| **MEDIUM** | P2 (normal) | Standard escalation | bead + mail mayor |

## Tiered Escalation Flow

```
Agent -> gt escalate -s <SEVERITY> "description"
           |
           v
     [Deacon receives]
           |
           +-- resolves --> updates issue, re-slings work
           +-- cannot  --> forwards to Mayor
                              +-- resolves --> updates issue, re-slings
                              +-- cannot  --> forwards to Overseer --> resolves
```

## Commands

```
gt escalate -s <MEDIUM|HIGH|CRITICAL> "Short description"
gt escalate ack <bead-id>
gt escalate list [--severity=...] [--stale] [--unacked] [--all]
gt escalate stale [--dry-run]
gt escalate close <bead-id>
```

---

# Section 6: docs/design/scheduler.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/design/scheduler.md

# Scheduler Architecture

Config-driven capacity-controlled polecat dispatch.

## Quick Start

```
gt config set scheduler.max_polecats 5
gt sling gt-abc gastown
gt scheduler status
gt scheduler run
```

## Dispatch Modes

| Value | Mode | Behavior |
| --- | --- | --- |
| `-1` (default) | Direct dispatch | `gt sling` dispatches immediately |
| `0` | Direct dispatch | Same as `-1` |
| `N > 0` | Deferred dispatch | `gt sling` creates sling context bead, daemon dispatches |

## Sling Context Beads

Scheduling state is stored on separate ephemeral beads called sling contexts. The work bead is never modified by the scheduler.

## Circuit Breaker

Threshold: 3 consecutive failures. Break action: close sling context.

---

# Section 7: docs/design/polecat-lifecycle-patrol.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/design/polecat-lifecycle-patrol.md

## Polecat Lifecycle and Patrol Coordination

Core insight: Polecats do NOT complete complex molecules end-to-end. Instead, each molecule step gets one polecat session. The sandbox (branch, worktree) persists across sessions. Sessions are the pistons; sandboxes are the cylinders.

## Two Cleanup Stages

1. **Step Cleanup** (Session Dies, Sandbox Lives) - Step completes but more steps remain
2. **Molecule Cleanup** (Polecat Goes Idle) - Final step completes, work submitted

## GUPP + Pinned Work = Completion Guarantee

As long as three conditions hold, a molecule WILL eventually complete:
1. Work is pinned (hook_bead set)
2. Sandbox persists (branch + worktree exist)
3. Someone keeps spawning sessions

## The Four Patrol Agents

| Agent | Scope | Frequency | Key Checks |
| --- | --- | --- | --- |
| **Daemon** | Town-wide | 3-minute heartbeat | Session liveness, GUPP violations |
| **Boot/Deacon** | Town-wide | Per daemon tick | Deacon health, witness health |
| **Witness** | Per-rig | Continuous | Polecat health, zombie detection |
| **Refinery** | Per-rig | On demand | Merge queue processing |

---

# Section 8: docs/design/plugin-system.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/design/plugin-system.md

## Plugin System Design

> Status: Design proposal -- not yet implemented

## Problem Statement
Gas Town needs extensible, project-specific automation that runs during Deacon patrol cycles.

## Architecture

- Plugin locations: `~/gt/plugins/` (town-level) and `<rig>/plugins/` (rig-level)
- Execution model: Dog Dispatch (plugin execution dispatched to dogs, non-blocking)
- State tracking: Wisps on the ledger
- Plugin format: `plugin.md` with TOML frontmatter

## Gate Types

| Type | Config | Behavior |
| --- | --- | --- |
| `cooldown` | `duration = "1h"` | Query wisps, run if none in window |
| `cron` | `schedule = "0 9 * * *"` | Run on cron schedule |
| `condition` | `check = "cmd"` | Run check command, run if exit 0 |
| `event` | `on = "startup"` | Run on Deacon startup |
| `manual` | (no gate section) | Never auto-run |

---

# Section 9: docs/design/witness-at-team-lead.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/design/witness-at-team-lead.md

# Witness AT Team Lead: Implementation Spec

> Status: Future architecture -- NOT YET IMPLEMENTED

This document specifies how the Witness becomes an AT team lead, replacing the current tmux-based polecat session management with Claude Code Agent Teams.

## Key Concepts

- Witness enters **delegate mode** (structurally enforced coordination-only)
- Spawns polecat teammates for assigned work
- Monitors via AT's native lifecycle hooks
- Syncs completions to beads at task boundaries

## AT Spike Findings

Recommendation: CONDITIONAL GO for Phase 1 experiment.
5/8 criteria clear GO. 2 require workarounds. 1 conditional on cost validation.

## Critical Blockers

1. No per-teammate working directory - workaround: PreToolUse hook
2. No session resumption for teammates - workaround: PreCompact handoff + beads state recovery
3. Token cost ~7x per teammate - mitigated by using Sonnet for polecat teammates

---

# Section 10: docs/HOOKS.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/HOOKS.md

## Gas Town Hooks Management

Centralized hook management for Gas Town workspaces.

## Architecture

```
~/.gt/hooks-base.json              <- Shared base config (all agents)
~/.gt/hooks-overrides/
  ├── crew.json                    <- Override for all crew workers
  ├── witness.json                 <- Override for all witnesses
  ├── gastown__crew.json           <- Override for gastown crew specifically
  └── ...
```

Merge strategy: `base -> role -> rig+role` (more specific wins)

## Commands

```
gt hooks sync             # Write all settings files
gt hooks diff             # Show differences
gt hooks base             # Edit the shared base config
gt hooks override <target> # Edit overrides
gt hooks list             # Show all targets
gt hooks scan             # Scan existing hooks
gt hooks init             # Bootstrap base config
gt hooks registry         # Browse available hooks
gt hooks install <hook-id> # Install a hook
```

## Current Registry Hooks

| Hook | Event | Enabled | Roles |
| --- | --- | --- | --- |
| pr-workflow-guard | PreToolUse | Yes | crew, polecat |
| session-prime | SessionStart | Yes | all |
| pre-compact-prime | PreCompact | Yes | all |
| mail-check | UserPromptSubmit | Yes | all |
| costs-record | Stop | Yes | crew, polecat, witness, refinery |
| clone-guard | PreToolUse | No | crew, polecat |
| dangerous-command-guard | PreToolUse | Yes | crew, polecat |

---

# Section 11: docs/WASTELAND.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/WASTELAND.md

## Getting Started with the Wasteland

The Wasteland is a federated work coordination network linking Gas Towns through DoltHub.

> Status: Phase 1 (wild-west mode)

## Quick Reference

| Command | Purpose |
| --- | --- |
| `gt wl join <upstream>` | Join a wasteland |
| `gt wl browse` | View the wanted board |
| `gt wl claim <id>` | Claim a wanted item |
| `gt wl done <id> --evidence <url>` | Submit completion evidence |
| `gt wl post --title "..."` | Post a new wanted item |
| `gt wl sync` | Pull upstream changes |

## Core Concepts

- **Wanted Board** - shared list of open work
- **Rigs** - participant identity
- **Stamps and Reputation** - multi-dimensional attestation
- **Trust Levels** (Planned) - 0-3 progression

## Database Schema

Seven tables: `_meta`, `rigs`, `wanted`, `completions`, `stamps`, `badges`, `chain_meta`

---

# Section 12: docs/otel-data-model.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/otel-data-model.md

All Gastown telemetry events are OTel log records exported via OTLP.

## 1. Identity hierarchy

- **Instance**: `hostname:basename(town_root)`
- **Run**: UUID per agent spawn. All OTel records carry `run.id`.

## 2. Events

| Event | Key Attributes |
| --- | --- |
| `agent.instantiate` | run.id, role, agent_type, issue_id, git_branch |
| `session.start/stop` | run.id, session_id, status |
| `prime` | run.id, role, hook_mode, formula |
| `prompt.send` | run.id, session, keys, debounce_ms |
| `agent.event` | run.id, event_type, content |
| `agent.usage` | run.id, input_tokens, output_tokens |
| `bd.call` | run.id, subcommand, args, duration_ms |
| `mail` | run.id, operation, msg.from, msg.to, msg.subject |
| `agent.state_change` | run.id, agent_id, new_state |
| `mol.cook/wisp/squash/burn` | run.id, formula_name |
| `bead.create` | run.id, bead_id, parent_id |
| `sling/nudge/done/polecat.spawn/convoy.create` | run.id + specific fields |

## 4. Environment Variables

| Variable | Description |
| --- | --- |
| `GT_RUN` | Run UUID |
| `GT_OTEL_LOGS_URL` | OTLP logs endpoint |
| `GT_OTEL_METRICS_URL` | OTLP metrics endpoint |
| `GT_LOG_AGENT_OUTPUT` | Opt-in: stream agent events |
| `GT_LOG_BD_OUTPUT` | Opt-in: include bd stdout/stderr |
| `GT_LOG_MAIL_BODY` | Opt-in: include mail body |
| `GT_LOG_PROMPT_KEYS` | Opt-in: include prompt text |

---

# Section 13: docs/agent-provider-integration.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/agent-provider-integration.md

# Agent Provider Integration Guide

How to integrate your agent CLI with Gas Town (and the upcoming Gas City).

## Integration Tiers

| Tier | Effort | What You Get | What You Provide |
| --- | --- | --- | --- |
| **0: Zero** | Nothing | Basic tmux orchestration | A CLI that runs in a terminal |
| **1: Preset** | JSON config file | Full lifecycle, resume, process detection | Preset entry in `agents.json` |
| **2: Hooks** | Settings file or plugin | Context injection, tool guards, mail delivery | Hook installer function |
| **3: Deep** | Code + scripts | Non-interactive mode, session forking, wrapper | Native API integration |

## Preset Registration (Tier 1)

JSON config at `~/gt/settings/agents.json` with fields: name, command, args, process_names, session_id_env, resume_flag, resume_style, prompt_mode, ready_delay_ms, hooks_provider, etc.

## Hooks Integration (Tier 2)

Three patterns:
- Pattern A: Claude-compatible settings.json
- Pattern B: Plugin/script hooks (like OpenCode's JS plugins)
- Pattern C: Informational hooks (instructions file)

## Gas City Provider Contract (Forward-Looking)

```
interface AgentProvider {
    Start(workDir, env) -> Process
    IsReady() -> bool
    IsAlive() -> bool
    SendMessage(text) -> error
    GetStatus() -> AgentStatus
    InjectContext(context) -> error
    Resume(sessionID) -> Process
    ForkSession(sessionID) -> Process
    Exec(prompt) -> Result
    GetUsage() -> UsageReport
}
```

## Capability Matrix

| Agent | Hooks | Resume | Non-Interactive | Prompt Mode |
| --- | --- | --- | --- | --- |
| Claude | Yes | `--resume` | Native | arg |
| Gemini | Yes | `--resume` | `-p` | arg |
| Codex | No | `resume` subcmd | `exec` subcmd | none |
| Cursor | No | `--resume` | `-p` | arg |
| OpenCode | Yes (plugin) | No | `run` subcmd | none |
| Copilot | Yes | `--resume` | No | arg |

---

# Section 14: docs/concepts/molecules.md
Source: https://github.com/gastownhall/gastown/blob/main/docs/concepts/molecules.md

## Molecules

Molecules are workflow templates that coordinate multi-step work in Gas Town.

## Molecule Lifecycle

```
Formula (source TOML) --- "Ice-9"
    |
    v bd cook
Protomolecule (frozen template) --- Solid
    |
    +-> bd mol pour -> Mol (persistent) --- Liquid
    +-> bd mol wisp --root-only -> Root Wisp (ephemeral) --- Vapor
```

**Root-only wisps** (default): Formula steps NOT materialized as database rows. Only single root wisp created. Prevents wisp accumulation.

**Poured wisps** (`pour = true`): Steps ARE materialized as sub-wisps with checkpoint recovery.

## How Agents See Steps

Agents see formula steps rendered inline when running `gt prime`. They work through the checklist and run `gt done` or `gt patrol report`.

## Commands

```
bd formula list / show / cook
bd mol list / show / wisp / bond
gt hook / prime / mol attach / mol detach
gt patrol new / patrol report
```

## Heuristic

If you would curse losing the progress after a crash, set `pour = true`. High frequency + cheap steps = inline (default). Low frequency + expensive steps = pour.

---

# 404 - File Not Found

The following URLs returned 404 (files do not exist in the repository):

- docs/design/convoy/README.md
- docs/concepts/escalation.md
- docs/concepts/scheduler.md
- docs/concepts/wasteland.md

These files may have been moved, renamed, or not yet created.
