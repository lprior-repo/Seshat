export type OperatorName =
  | "tool_churn"
  | "drag_jitter"
  | "zoom_pulse"
  | "undo_redo_fork"
  | "panel_reflow"
  | "zoom_wheel_burst"
  | "select_all_deselect"
  | "edge_create_cancel"
  | "subgraph_collapse_expand"
  | "clipboard_copy_paste_cycle"
  | "import_export_roundtrip"
  | "keyboard_shortcut_fuzz"
  | "pan_drag_race"
  | "validation_toggle_storm"
  | "minimap_viewport_drag"
  | "resize_handle_corner_se"
  | "resize_handle_corner_nw"
  | "theme_flip_during_gesture"
  | "delete_undelete_cycle";

export type OperatorCategory =
  | "shell_boot"
  | "tool_modes"
  | "selection"
  | "transform"
  | "viewport"
  | "edges"
  | "subgraph"
  | "history"
  | "panels"
  | "keyboard"
  | "numeric"
  | "race_chaos"
  | "performance"
  | "import_export";

export type RedQueenOp = Readonly<{
  operator: OperatorName;
  intensity: number;
  category: OperatorCategory;
}>;

export type RedQueenTrace = Readonly<{
  sceneId: string;
  seed: number;
  wave: 1 | 2 | 3;
  operators: ReadonlyArray<RedQueenOp>;
  timestamp?: string;
}>;

export type SeedCorpusEntry = Readonly<{
  id: string;
  trace: RedQueenTrace;
  promotedAt: string;
  failureReason: string;
  fixedAt: string | null;
}>;

export type InvariantCheck = Readonly<{
  name: string;
  passed: boolean;
  message?: string;
}>;

export type WaveResult = Readonly<{
  wave: 1 | 2 | 3;
  passed: number;
  failed: number;
  seedsRun: ReadonlyArray<number>;
  invariantBreaches: ReadonlyArray<string>;
}>;
