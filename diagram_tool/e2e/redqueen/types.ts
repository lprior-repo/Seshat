export type OperatorName =
  | "tool_churn"
  | "drag_jitter"
  | "zoom_pulse"
  | "undo_redo_fork"
  | "panel_reflow";

export type RedQueenOp = Readonly<{
  operator: OperatorName;
  intensity: number;
}>;

export type RedQueenTrace = Readonly<{
  sceneId: string;
  seed: number;
  wave: 1 | 3;
  operators: ReadonlyArray<RedQueenOp>;
}>;
