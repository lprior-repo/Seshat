import { Effect } from "effect";
import type { Page } from "@playwright/test";
import type { RedQueenOp, RedQueenTrace } from "./types";
import { runEffect } from "../helpers";

const clickTool = (page: Page, tool: "Select" | "Pan" | "Edge" | "Subgraph" | "Text") =>
  Effect.tryPromise(() => page.getByRole("button", { name: tool, exact: true }).click());

const clickPanelToggle = (page: Page, toggle: "Icons" | "Props" | "Mini" | "Valid") =>
  Effect.tryPromise(() => page.getByRole("button", { name: toggle, exact: true }).click());

const opProgram = (page: Page, op: RedQueenOp): Effect.Effect<void, never, never> => {
  const repeated = (n: number, program: Effect.Effect<void, never, never>) =>
    Effect.all(Array.from({ length: Math.max(1, n) }, () => program), {
      concurrency: 1,
      discard: true,
    });

  switch (op.operator) {
    case "tool_churn":
      return repeated(
        op.intensity,
        Effect.all(
          [
            clickTool(page, "Text"),
            clickTool(page, "Edge"),
            clickTool(page, "Select"),
            clickTool(page, "Pan"),
            clickTool(page, "Select"),
          ],
          { concurrency: 1, discard: true },
        ),
      ).pipe(Effect.orDie);
    case "drag_jitter":
      return Effect.tryPromise(async () => {
        const canvas = page.getByTestId("canvas-container");
        const box = await canvas.boundingBox();
        if (!box) {
          return;
        }
        const x = box.x + box.width * 0.55;
        const y = box.y + box.height * 0.4;
        await page.mouse.move(x, y);
        await page.mouse.down();
        await page.mouse.move(x + 12 * op.intensity, y + 8 * op.intensity, { steps: 5 });
        await page.mouse.up();
      }).pipe(Effect.orDie);
    case "zoom_pulse":
      return repeated(
        op.intensity,
        Effect.all([clickTool(page, "Select"), clickTool(page, "Select")], {
          concurrency: 1,
          discard: true,
        })
          .pipe(Effect.zipRight(Effect.tryPromise(() => page.getByTestId("zoom-in").click())))
          .pipe(Effect.zipRight(Effect.tryPromise(() => page.getByTestId("zoom-out").click())))
          .pipe(Effect.orDie),
      );
    case "undo_redo_fork":
      return repeated(
        op.intensity,
        Effect.all(
          [
            Effect.tryPromise(() => page.getByRole("button", { name: "Undo", exact: true }).click()),
            Effect.tryPromise(() => page.getByRole("button", { name: "Redo", exact: true }).click()),
          ],
          { concurrency: 1, discard: true },
        ).pipe(Effect.orDie),
      );
    case "panel_reflow":
      return repeated(
        op.intensity,
        Effect.all(
          [
            clickPanelToggle(page, "Icons"),
            clickPanelToggle(page, "Props"),
            clickPanelToggle(page, "Mini"),
            clickPanelToggle(page, "Valid"),
          ],
          { concurrency: 1, discard: true },
        ).pipe(Effect.orDie),
      );
    default:
      return Effect.void;
  }
};

export async function runTrace(page: Page, trace: RedQueenTrace): Promise<void> {
  const program = trace.operators.reduce(
    (acc, op) => acc.pipe(Effect.zipRight(opProgram(page, op))),
    Effect.void,
  );
  await runEffect(() => Effect.runPromise(program));
}
