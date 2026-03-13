# Test Plan: seshat-nwc

## Unit Tests

### test_dispatch_ungroup_sends_envelope_when_db_tx_available
- **Given**: A valid db_tx channel and a group_id
- **When**: dispatch_ungroup is called
- **Then**: Returns Ok with dispatches_sent = 1

### test_dispatch_ungroup_returns_error_when_db_tx_none
- **Given**: db_tx is None
- **When**: dispatch_ungroup is called
- **Then**: Returns Err(DispatchError::WalDisconnected)

### test_apply_ungroup_selection_dispatches_to_db_tx
- **Given**: Document with selected subgraph, db_tx available
- **When**: apply_ungroup_selection is called
- **Then**: dispatch_ungroup is called with correct group_id

### test_apply_ungroup_selection_returns_false_when_no_subgraphs
- **Given**: Document with no subgraphs selected
- **When**: apply_ungroup_selection is called
- **Then**: Returns false, no dispatch occurs

## Integration Tests

### test_integration_ungroup_dispatch_to_wal
- **Given**: Real mpsc channel connected to test receiver
- **When**: dispatch_ungroup is called
- **Then**: WAL receives EventEnvelope with DomainOp::Ungroup

### test_integration_keyboard_triggers_ungroup_dispatch
- **Given**: Keyboard hook registered, db_tx available
- **When**: Ctrl+Shift+G pressed with group selected
- **Then**: dispatch_ungroup called with correct group_id

## E2E Tests

### test_e2e_user_presses_ctrl_shift_g_with_group_selected
- **Given**: User has diagram with subgraph node selected
- **When**: User presses Ctrl+Shift+G
- **Then**: Subgraph is ungrouped, children become top-level nodes

### test_e2e_user_presses_ctrl_shift_g_without_selection
- **Given**: User has diagram with no selection
- **When**: User presses Ctrl+Shift+G
- **Then**: No state change, no error displayed
