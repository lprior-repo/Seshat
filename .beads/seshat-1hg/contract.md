# Contract Specification: seshat-1hg

bead_id: seshat-1hg
bead_title: ui-arch: Extract clipboard logic from thread_local
phase: contract
updated_at: 2026-03-12T23:04:00Z

## Overview
Extract clipboard logic from thread_local RefCell-based state to idiomatic Dioxus primitives (Signal, use_context, use_context_provider).

## Preconditions (P)
- P1: No authentication required
- P2: System state: Clipboard currently uses thread_local (pre-refactor)
- P3: Clipboard state must be accessible via Dioxus Signal

## Postconditions (Q)
- Q1: Clipboard uses use_store/use_context (Signal<Option<ClipboardData>>)
- Q2: No thread_local RefCell usage remains in clipboard modules
- Q3: Operations return bool (graceful failure handling)

## Invariants (I)
- I1: Clipboard state updates trigger reactivity correctly
- I2: Clipboard operations are atomic (immutable data)
- I3: Clipboard signal initialization semantics

## Error Taxonomy
- ClipboardError::EmptyClipboard
- ClipboardError::NoSelection
- ClipboardError::PasteFailed
- ClipboardError::ContextNotFound
