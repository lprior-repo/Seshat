# Contract Specification

## Context
- Feature: Transform Persistence (MUL-026 to MUL-030)
- Domain terms: Multi-selection, Transforms (translation, scale, rotation), Persistence (Committing transforms to the document, generating undo records), Document Model, Bounding Box.
- Assumptions: Transformations are applied interactively and then committed to persistence. The commit step must be atomic across all selected items.
- Open questions: Does the transform persistence create a single composite Undo record or one per item? (Assumption: single composite Undo record for the whole selection).

## Preconditions
- [P1] The selection must not be empty when a transform commit is requested.
- [P2] The transform parameters (delta x, y, scale factors, angle) must be finite, valid numbers (no NaN or Infinity).
- [P3] All selected item IDs must exist in the document.
- [P4] The document must not be locked or read-only.

## Postconditions
- [G1] All selected items have their local geometries (positions, sizes) updated according to the transformation.
- [G2] A single composite undo action is pushed to the history for the multi-item transform.
- [G3] The document's unsaved changes flag (or version/last_modified) is incremented.
- [G4] If any item update fails, no items are updated (atomic transaction).

## Invariants
- [I1] The relative spatial relationships (proportions or distances) between selected items are preserved.
- [I2] A transform operation never leaves the document in a partially updated state.

## Error Taxonomy
- `Error::EmptySelection` - when trying to persist a transform for an empty selection.
- `Error::InvalidTransform` - when the transform contains NaN, Infinity, or scaling by zero.
- `Error::ItemNotFound(ItemId)` - when an item ID in the selection doesn't exist in the document.
- `Error::DocumentLocked` - when trying to persist changes to a read-only document.
- `Error::PersistenceFailed` - when the underlying storage or commit log fails to record the change.

## Contract Signatures
- `fn commit_transform(selection: &NonEmptySelection, transform: &ValidTransform, doc: &mut Document) -> Result<(), Error>`

## Type Encoding
For each precondition, specify the strongest possible type enforcement:
| Precondition | Enforcement Level | Type / Pattern |
|---|---|---|
| Non-empty selection | Compile-time (strongest) | `NonEmptySelection<'a>` or `NonEmptyVec<ItemId>` |
| Valid transform | Compile-time | `ValidTransform` (checked builder, e.g., `ValidTransform::new(...) -> Result<Self, Error>`) |
| Items exist | Result error variant | `Result<(), Error::ItemNotFound>` |
| Document writable | Result error variant | `Result<(), Error::DocumentLocked>` |

## Violation Examples (REQUIRED -- one per precondition and postcondition)
- VIOLATES P1: `commit_transform(&Selection::empty(), &valid_transform, &mut doc)` -- should produce a compile-time error if `NonEmptySelection` is used, otherwise `Err(Error::EmptySelection)`.
- VIOLATES P2: `commit_transform(&valid_selection, &Transform::scale(f64::NAN), &mut doc)` -- should produce a compile-time error if `ValidTransform` builder is used, or `Err(Error::InvalidTransform)`.
- VIOLATES P3: `commit_transform(&valid_selection_with_missing_id, &valid_transform, &mut doc)` -- should produce `Err(Error::ItemNotFound(missing_id))`
- VIOLATES P4: `commit_transform(&valid_selection, &valid_transform, &mut read_only_doc)` -- should produce `Err(Error::DocumentLocked)`
- VIOLATES G1: After commit, `doc.get_item(id).position == old_position` -- test should fail if the document geometry wasn't updated.
- VIOLATES G4: A partial failure updates item A but leaves item B unchanged -- should produce `Err(Error::PersistenceFailed)` and roll back.

## Ownership Contracts (Rust-specific)
- Shared borrow: `fn commit_transform(selection: &NonEmptySelection, transform: &ValidTransform, ...)` -- read-only, no mutation, borrows the selection and transform specifications.
- Exclusive borrow: `fn commit_transform(..., doc: &mut Document)` -- mutation contract: modifies `doc.items` (updating geometries), `doc.history` (pushing a composite undo command), and `doc.metadata` (incrementing version).
- Clone policy: The transformation is cloned or applied by value to the individual items. The items themselves are updated in place, not cloned unless necessary for the undo log.