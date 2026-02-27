import type { RedQueenOp } from "./types";

const baseOps: ReadonlyArray<RedQueenOp> = [
  { operator: "tool_churn", intensity: 2 },
  { operator: "drag_jitter", intensity: 3 },
  { operator: "zoom_pulse", intensity: 2 },
  { operator: "undo_redo_fork", intensity: 2 },
  { operator: "panel_reflow", intensity: 1 },
];

export function traceForSeed(seed: number, wave: 1 | 3): ReadonlyArray<RedQueenOp> {
  const multiplier = wave === 3 ? 2 : 1;
  const count = Math.min(baseOps.length, 3 + (seed % 3));
  return baseOps.slice(0, count).map((op, index) => ({
    operator: op.operator,
    intensity: op.intensity * multiplier + (index % 2),
  }));
}
