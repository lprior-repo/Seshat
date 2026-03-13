# Martin Fowler Test Plan: seshat-1hg

bead_id: seshat-1hg
bead_title: ui-arch: Extract clipboard logic from thread_local
phase: test-plan
updated_at: 2026-03-12T23:04:00Z

## Overview
Validates clipboard refactoring from thread_local to Dioxus Signal/Context.

## Happy Path Tests
- test_clipboard_data_new_creates_empty_state
- test_copy_selection_returns_some_when_nodes_selected  
- test_paste_contents_returns_some_with_new_nodes

## Error Path Tests
- test_copy_selection_returns_none_when_no_selection
- test_paste_returns_none_when_clipboard_empty

## Contract Verification (Behavioral)
- test_contract_clipboard_operations_work_without_thread_local
- test_contract_signal_type_in_signatures

## Integration Tests
- test_integration_copy_paste_round_trip

## Notes
Tests exist in diagram_tool/src/ui/commands.rs. Test compilation has pre-existing failures unrelated to clipboard.
