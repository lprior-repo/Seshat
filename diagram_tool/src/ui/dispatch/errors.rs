//! Error types for dispatch operations

/// Error types for dispatch operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    /// WAL disconnected - `db_tx` is None
    WalDisconnected,
    /// Channel missing - `db_tx` is None (seshat-088)
    ChannelMissing,
    /// No transaction - `db_tx` is None (seshat-5zs)
    NoTx,
    /// Failed to send to `db_tx` channel
    SendFailed,
    /// Invalid coordinates (NaN or Infinity)
    InvalidCoordinates,
    /// No selection for delete operation
    NoSelection,
    /// Dispatch incomplete - sent count doesn't match expected
    DispatchIncomplete,
    /// Edge not found in document
    EdgeNotFound,
    /// Edge not in selection
    NotSelected,
    /// Edge would create a cycle in DAG
    CycleDetected,
    /// Self-loop: source equals target
    SelfLoop,
}

/// Result of a dispatch operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchResult {
    /// Number of nodes deleted/dispatched
    pub nodes_affected: usize,
    /// Number of envelopes sent to `db_tx`
    pub dispatches_sent: usize,
}
