import type { OperatorCategory, OperatorName, RedQueenOp } from "./types";

const op = (operator: OperatorName, intensity: number, category: OperatorCategory): RedQueenOp => ({
  operator,
  intensity,
  category,
});

const allOps: ReadonlyArray<RedQueenOp> = [
  op("tool_churn", 2, "tool_modes"),
  op("drag_jitter", 3, "transform"),
  op("zoom_pulse", 2, "viewport"),
  op("undo_redo_fork", 2, "history"),
  op("panel_reflow", 1, "panels"),
  op("zoom_wheel_burst", 3, "viewport"),
  op("select_all_deselect", 2, "selection"),
  op("edge_create_cancel", 2, "edges"),
  op("subgraph_collapse_expand", 2, "subgraph"),
  op("clipboard_copy_paste_cycle", 2, "history"),
  op("import_export_roundtrip", 1, "import_export"),
  op("keyboard_shortcut_fuzz", 3, "keyboard"),
  op("pan_drag_race", 3, "race_chaos"),
  op("validation_toggle_storm", 2, "panels"),
  op("minimap_viewport_drag", 2, "viewport"),
  op("resize_handle_corner_se", 3, "transform"),
  op("resize_handle_corner_nw", 3, "transform"),
  op("theme_flip_during_gesture", 2, "race_chaos"),
  op("delete_undelete_cycle", 2, "history"),
];

const categoryPriority: ReadonlyArray<OperatorCategory> = [
  "shell_boot",
  "tool_modes",
  "selection",
  "transform",
  "viewport",
  "edges",
  "subgraph",
  "history",
  "numeric",
  "race_chaos",
  "panels",
  "keyboard",
  "performance",
  "import_export",
];

function shuffleWithSeed<T>(arr: ReadonlyArray<T>, seed: number): ReadonlyArray<T> {
  const result = [...arr];
  let state = seed >>> 0;
  for (let i = result.length - 1; i > 0; i -= 1) {
    state = (state * 1_664_525 + 1_013_904_223) >>> 0;
    const j = state % (i + 1);
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

export function traceForSeed(seed: number, wave: 1 | 2 | 3): ReadonlyArray<RedQueenOp> {
  const multiplier = wave === 3 ? 2 : wave === 2 ? 1.5 : 1;
  const baseCount = wave === 3 ? 6 : wave === 2 ? 4 : 3;
  const opCount = Math.min(allOps.length, baseCount + (seed % 4));

  const shuffled = shuffleWithSeed(allOps, seed);

  return shuffled.slice(0, opCount).map((baseOp, index) => ({
    operator: baseOp.operator,
    intensity: Math.max(1, Math.floor(baseOp.intensity * multiplier + (index % 3))),
    category: baseOp.category,
  }));
}

export function opsByCategory(category: OperatorCategory): ReadonlyArray<RedQueenOp> {
  return allOps.filter((op) => op.category === category);
}

export function traceForCategory(
  category: OperatorCategory,
  seed: number,
  wave: 1 | 2 | 3,
): ReadonlyArray<RedQueenOp> {
  const categoryOps = opsByCategory(category);
  const multiplier = wave === 3 ? 2 : wave === 2 ? 1.5 : 1;
  const shuffled = shuffleWithSeed(categoryOps, seed);

  return shuffled.map((baseOp, index) => ({
    operator: baseOp.operator,
    intensity: Math.max(1, Math.floor(baseOp.intensity * multiplier + (index % 2))),
    category: baseOp.category,
  }));
}

export function allCategories(): ReadonlyArray<OperatorCategory> {
  return categoryPriority;
}

export { allOps };
