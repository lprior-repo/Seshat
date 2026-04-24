# REDQUEEN Findings: se-qzu

## Target
`diagram_models/src/clipboard_contract_tests.rs` — coevolutionary quality testing against `clipboard_contract.rs`

## Starting State
- 17 existing tests covering happy paths and basic error paths
- Implementation: 3 public functions (`copy`, `cut`, `calculate_paste`), 8 error variants

## New Tests Added (15 adversarial tests)

### Mutation Killers Added

| Test | Mutation Target | Line(s) | Killing Strategy |
|------|----------------|---------|------------------|
| `given_self_loop_edge_when_copy_then_edge_included_in_clipboard` | Edge filter `&&` on self-referencing edge | L97 | Self-loop where source==target must still pass `&&` check |
| `given_node_with_parent_in_doc_not_clipboard_when_paste_then_parent_kept` | Parent remap `doc.document.nodes.contains_key(parent)` removed | L213 | Parent in doc but not clipboard must be kept (not remapped) |
| `given_copy_preserves_all_node_fields_then_paste_preserves_them` | Field stripping in paste (removing `..node.clone()`) | L169-174 | All 14 Node fields verified after round-trip |
| `given_cut_then_paste_then_new_nodes_exist_and_old_removed` | Cut not removing nodes / paste not creating new IDs | L117-126, L162 | Full cut→paste round-trip with edge remapping verification |
| `given_multiple_edges_same_pair_when_copy_then_all_edges_copied` | Edge deduplication (collecting only first edge per pair) | L92-100 | Two edges between same nodes must both survive |
| `given_deep_parent_chain_when_paste_then_all_parents_remapped` | Cycle detection false positive on long chains | L189-204 | 8-level deep chain must not trigger false cycle detection |
| `given_duplicate_edge_ids_in_clipboard_when_paste_then_corrupt_clipboard` | Edge ID dedup check removed | L149-154 | Duplicate edge IDs must be caught |
| `given_paste_result_selection_contains_all_new_node_ids` | `new_selection` not populated or incomplete | L273-276 | Every pasted node must appear in new_selection |
| `given_edge_with_one_end_in_clipboard_other_in_doc_when_paste_then_invalid_edge_reference` | Edge validation `!is_remapped && !doc.contains` changed to `\|\|` | L250-254 | Edge to nowhere must fail |
| `given_node_self_parent_when_paste_then_cyclic_detected` | Cycle detection skip for self-parent | L198 | `n.parent = Some(n)` must be caught as cycle |
| `given_copy_with_nonexistent_node_in_selection_then_postcondition_violated` | Missing node lookup | L80-90 | Ghost node in selection must return PostconditionViolated |
| `given_two_node_cycle_when_paste_then_cyclic_detected` | Mutual cycle detection | L189-204 | A→B→A cycle must be caught |
| `given_paste_serial_zero_then_offset_is_20` | `paste_serial + 1` → `paste_serial` | L156 | Serial=0 must give offset=20, not 0 |
| `given_cut_removes_edges_connected_to_cut_nodes` | Cut not removing nodes | L117-126 | After cut, only unselected nodes remain in doc |
| `given_edge_preserves_label_and_style_through_copy_paste` | Edge field stripping in paste | L264-268 | Edge label, color, thickness, directed all survive round-trip |

## Coverage Analysis

### Error Variant Coverage (8/8)
- `EmptySelection` — test_copy_returns_error_when_selection_is_empty
- `EmptyClipboard` — test_paste_returns_error_when_clipboard_is_empty
- `CorruptClipboard` — test_corrupt_clipboard_with_duplicate_node_ids, given_duplicate_edge_ids_in_clipboard_when_paste_then_corrupt_clipboard
- `DuplicateIdCreated` — defense-in-depth (UUID collision), not directly testable
- `InvalidEdgeReference` — test_q6_violation, given_edge_to_external_node, given_edge_from_external_to_pasted, given_edge_with_one_end_in_clipboard_other_in_doc
- `InvalidParentReference` — test_q7_violation, given_node_with_parent_in_doc_not_clipboard (inverse)
- `CyclicParentReference` — test_cyclic_parent_reference, given_node_self_parent, given_two_node_cycle, given_three_node_chain (inverse)
- `PostconditionViolated` — given_copy_with_nonexistent_node_in_selection

### Mutation-Susceptible Paths Covered
1. `&&` vs `||` in edge filtering (L97) — 4 tests
2. `+1` in offset calculation (L156) — 2 tests
3. Cycle detection loop (L189-204) — 4 tests
4. Parent remapping: clipboard vs doc (L209-217) — 3 tests
5. Edge source/target remapping (L235-248) — 5 tests
6. Edge validation check (L250-254) — 4 tests
7. Field preservation through clone (L169-174, L264-268) — 2 tests
8. Duplicate detection: nodes vs edges (L143-154) — 2 tests

## Results
- All 32 tests pass (17 original + 15 new)
- Zero compilation warnings
- No bugs found in implementation — code is well-structured

## Files Changed
- `diagram_models/src/clipboard_contract_tests.rs` — 15 new tests added (408→817 lines)
