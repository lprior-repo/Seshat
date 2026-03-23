# Code Review Defects

STATUS: REJECTED

The implementation was rigorously reviewed. While the previous defects were addressed, new critical flaws were introduced in the fixes:

## 1. TOCTOU (Time-Of-Check to Time-Of-Use) / Stale Existence Check
The existence of the node and edge is still checked *outside* of the atomic mutation closure (`mutate_doc_with_history`).
- In `commit_node_edit`, `ensure_node_exists(doc_signal, node_id)?` reads the signal state before the transaction.
- In `commit_edge_edit`, `doc_signal.read().document.edges.contains_key(...)` performs a stale read.
- Inside the `mutate_doc_with_history` closure, if the entity was concurrently deleted (or if the outer check was stale), `current_doc.document.nodes.get(&node_id_clone).map_or_else(String::new, ...)` incorrectly falls back to an empty string.
- If the `new_label` is also an empty string, the condition `old_label == new_label_clone` evaluates to `true`, and the transaction completes successfully (`Ok(current_doc.clone())`), failing to return the required `CommitError::TargetNotFound`.
- **Fix:** The existence check must occur *strictly inside* the `mutate_doc_with_history` closure using the `current_doc` state, and the outer checks (`ensure_node_exists` and `edge_exists`) must be removed.

## 2. Flawed Input Validation (Overzealous Control Character Ban)
The `is_valid_label` function rejects *all* control characters by using `c.is_control()`.
- The `char::is_control` method returns `true` for ASCII control characters, which includes common formatting characters like newline (`\n`), carriage return (`\r`), and tab (`\t`).
- By completely banning `is_control()`, the implementation breaks legitimate multi-line text and text with tabs.
- **Fix:** The validation should explicitly allow safe whitespace control characters. For example: `c.is_control() && c != '\n' && c != '\r' && c != '\t'`.