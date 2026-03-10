# Bead: seshat-k40

**Title**: Refactor store.rs using Scott Wlaschin DDD

**Phase**: STATE 1 - Contract Synthesis

**Status**: In Progress

## Summary
Refactor `store_async.rs` to use Scott Wlaschin DDD types (`ValidEvent`, `BoundedBatch`, `Revision`) instead of primitive inputs, and update downstream call sites to parse at boundaries.

## Context
- The project has DDD types already defined in `diagram_tool/src/store/types.rs`
- Need to update function signatures in `store_async.rs` to accept validated types
- Update 92+ call sites in downstream modules

## Current Focus
Synthesizing contract specification - COMPLETED

## Phase History
- STATE 1 (Contract Synthesis): COMPLETED - Contract and tests created
- STATE 2 (Test Review): APPROVED - Test plan accepted

## Current Focus
Implementing the contract - launching functional-rust agent
