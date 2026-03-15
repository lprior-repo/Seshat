# Implementation Summary
- Modified `diagram_tool/src/ui/canvas.rs` `pointerdown` and `onmousedown` handlers to check for node hits using `find_node_at(&doc, pos.0, pos.1)`.
- If a node is hit, it selects the node (or adds to selection if additive) and sets interaction mode to `DraggingSelection`.
- If nothing is hit, it clears the selection (unless additive) and starts a `RubberBand` selection.