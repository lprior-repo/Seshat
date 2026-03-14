# Contract: seshat-x68 - MUL-016 to MUL-020 Multi-select rotation

## Description
Rotate a group of nodes around a center point without deformation.

## MUL-016: Rotate asymmetric selection
Test rotation of nodes with irregular/non-uniform distribution around the selection center.

## MUL-017: Rotate preserves relative distances
Test that distances between all pairs of nodes are preserved after rotation.

## MUL-018: Rotate snaps to 90-degree increments
Test that rotation snaps to cardinal directions (0, 90, 180, 270 degrees).

## MUL-019: Rotate with subpixel precision
Test rotation with non-integer coordinates.

## MUL-020: Rotate edge cases
Test rotation with edge cases: single node, two nodes, collinear nodes, empty selection.
