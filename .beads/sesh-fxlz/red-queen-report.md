# Red Queen Report: Edge Label Editing Contract Violations

## Summary
Adversarial testing of edge label editing was conducted by generating inputs that violate potential application contracts, limits, and safe rendering boundaries. Testing discovered that the system performs **zero validation** on edge labels across multiple core domain boundaries.

## Findings
The `apply_update_edge_label` (in `diagram_models::projection::ops::edge_ops`) and `calculate_edge_label_edit` (in `canvas_domain::interaction_reducer::commit`) functions blindly accept edge label strings of any composition and length.

### Missing Validation Scenarios Verified:
1. **Massive Payload Lengths**: Labels of 100,000+ characters are immediately accepted. This can lead to denial-of-service (OOM, database storage bloat, or UI lock-up during rendering/measurement in the canvas).
2. **Null Bytes (`\0`)**: Accepted seamlessly. Can break string handling where null-terminated strings are expected (e.g., C-FFI, serialization boundaries).
3. **Control Characters (`\x01`, `\n`, `\r`)**: Accepted without escaping. Newlines in edge labels might break line-rendering bounds calculations or layout logic.
4. **Zero-width Spaces (`\u{200B}`) & Bi-directional Overrides (`\u{202E}`)**: Accepted without filtering, potentially leading to visual spoofing of diagrams or invisible labels that interfere with hit-testing and edge selection boundaries.

### Validation Gap in Schema
The `diagram_models::schema::validation::validate_document` function verifies structural consistency (e.g. `label_offset_t.0.is_finite()`), but completely fails to validate length bounds or disallowed character classes for edge or node labels.

## Generated Artifacts
- **Test File**: `diagram_models/tests/adversarial_edge_label.rs`
- **Bead Created**: `sesh-i38v` ("Edge label editing lacks validation for massive payloads and malicious characters" - P0 Priority)
