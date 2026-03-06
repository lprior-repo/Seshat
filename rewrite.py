import sys
import re

with open('/home/lewis/src/seshat/diagram_tool/src/ui/canvas/interaction_reducer.rs', 'r') as f:
    content = f.read()

# Replace safe_zoom and within definitions
content = re.sub(
    r'fn safe_zoom\(zoom: f64\) -> Option<f64> \{\s+\(zoom\.is_finite\(\) && zoom > f64::EPSILON\)\.then_some\(zoom\)\s+\}',
    '',
    content
)

content = re.sub(
    r'fn within\(subgraph: \(f64, f64, f64, f64\), node: \(f64, f64, f64, f64\)\) -> bool \{\s+let \(sx, sy, sw, sh\) = subgraph;\s+let \(nx, ny, nw, nh\) = node;\s+nx >= sx && ny >= sy && nx \+ nw <= sx \+ sw && ny \+ nh <= sy \+ sh\s+\}',
    '',
    content
)

# Update start_resize_interaction pointer math
old_start_resize = """        let Some(zoom) = safe_zoom(doc.editor_state.zoom.0) else {
            return;
        };
        let cx = (client_x / zoom) + doc.editor_state.camera_x.0;
        let cy = (client_y / zoom) + doc.editor_state.camera_y.0;"""

new_start_resize = """        let Some((cx, cy)) = crate::ui::canvas::math::screen_to_canvas(
            client_x,
            client_y,
            doc.editor_state.camera_x.0,
            doc.editor_state.camera_y.0,
            doc.editor_state.zoom.0,
        ) else {
            return;
        };"""

content = content.replace(old_start_resize, new_start_resize)

# Remove the imported safe_zoom, within from proptests module
content = content.replace(
    '        finalize_motion_release, resize_target_ids, safe_zoom, within, InteractionMode,',
    '        finalize_motion_release, resize_target_ids, InteractionMode,'
)

content = content.replace(
    '        finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle,',
    '        finalize_motion_release, resize_target_ids, InteractionMode, ResizeHandle,'
)
content = content.replace(
    '        resize_target_ids, within, InteractionMode',
    '        resize_target_ids, InteractionMode'
)

# Replace 'within(' with 'crate::ui::canvas::math::within('
content = content.replace('within(', 'crate::ui::canvas::math::within(')

# But wait, we shouldn't replace it in tests or we can let it be in tests because we moved some tests.
# Wait, some tests in interaction_reducer.rs might still use `within`. 
# It's better to just add an import if we need it, but `crate::ui::canvas::math::within` is safe.

with open('/home/lewis/src/seshat/diagram_tool/src/ui/canvas/interaction_reducer.rs', 'w') as f:
    f.write(content)
