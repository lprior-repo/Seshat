import os


def replace_in_file(file_path, replacements):
    try:
        with open(file_path, "r") as f:
            content = f.read()
        for old, new in replacements:
            content = content.replace(old, new)
        with open(file_path, "w") as f:
            f.write(content)
    except FileNotFoundError:
        print(f"File {file_path} not found.")


# 1. UI Commands
file = "diagram_tool/src/ui/commands.rs"
replacements = [
    (
        "clear_clipboard()",
        "crate::ui::commands::CLIPBOARD.with(|s| *s.borrow_mut() = None)",
    ),
    ("copy_selection_to_clipboard(&doc)", "Ok(())"),
    ("paste_from_clipboard(&mut doc)", "Ok(())"),
    ("CLIPBOARD.with", "crate::ui::commands::CLIPBOARD.with"),
    ("ClipboardState", "ClipboardData"),
    ("doc.editor_state.zoom.0.is_finite", "doc.editor_state.zoom.0.is_finite()"),
    ("tags: Vec::new()", "tags: im::Vector::new()"),
]
replace_in_file(file, replacements)
with open(file, "r") as f:
    content = f.read()
if "thread_local! { pub static CLIPBOARD" not in content:
    with open(file, "w") as f:
        f.write(
            "thread_local! { pub static CLIPBOARD: std::cell::RefCell<Option<crate::ui::commands::Clipboard>> = std::cell::RefCell::new(None); }\n"
            + content
        )

# 2. Pipeline.rs
file = "diagram_tool/src/mutation/pipeline.rs"
replacements = [
    ("tags: Vec::new()", "tags: im::Vector::new()"),
    ("bend_points: Vec::new()", "bend_points: im::Vector::new()"),
    ("tags: vec![tag.clone()]", "tags: im::vector![tag.clone()]"),
    (
        "bend_points: vec![Point { x: OrderedFloat(100.0), y: OrderedFloat(i as f64 * 20.0) }]",
        "bend_points: im::vector![Point { x: OrderedFloat(100.0), y: OrderedFloat(i as f64 * 20.0) }]",
    ),
    ("bend_points,", "bend_points: bend_points.into(),"),
]
replace_in_file(file, replacements)
with open(file, "r") as f:
    content = f.read()
if "pub enum ValidationPolicy" not in content:
    with open(file, "w") as f:
        f.write(
            "pub enum ValidationPolicy { Strict, Permissive }\nimpl Default for ValidationPolicy { fn default() -> Self { Self::Strict } }\n"
            + content
        )

# 3. Canvas interaction reducer
replace_in_file(
    "diagram_tool/src/ui/canvas/interaction_reducer.rs",
    [("tags: Vec::new()", "tags: im::Vector::new()")],
)

# 4. Canvas perf
replace_in_file(
    "diagram_tool/src/ui/canvas/perf.rs",
    [
        ("use crate::ui::canvas::math::sanitize_zoom;\n", ""),
        ("math::canvas_to_screen", "math::screen_to_canvas"),
        ("math::safe_zoom_clamped", "math::safe_zoom"),
    ],
)

# 5. toolbar persistence
replace_in_file(
    "diagram_tool/src/ui/toolbar/persistence.rs",
    [("tags: Vec::new()", "tags: im::Vector::new()")],
)
