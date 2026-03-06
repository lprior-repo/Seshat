import { Effect } from "effect";
import type { Locator, Page } from "@playwright/test";
import type { RedQueenOp, RedQueenTrace } from "./types";
 
const effectStep = <A>(step: () => Promise<A>): Effect.Effect<A, never, never> =>
  Effect.tryPromise({
    try: step,
    catch: (error) =>
      new Error(error instanceof Error ? error.message : String(error)),
  }).pipe(Effect.orDie);

const toolIds: Readonly<Record<"Select" | "Pan" | "Edge" | "Subgraph" | "Text", string>> = {
  Select: "tool-select",
  Pan: "tool-pan",
  Edge: "tool-edge",
  Subgraph: "tool-subgraph",
  Text: "tool-text",
};

const panelToggleIds: Readonly<Record<"Icons" | "Props" | "Mini" | "Valid", string>> = {
  Icons: "panel-icons-toggle",
  Props: "panel-props-toggle",
  Mini: "panel-mini-toggle",
  Valid: "panel-valid-toggle",
};

const toolbarIds: Readonly<Partial<Record<"Undo" | "Redo" | "Delete" | "Auto-Arrange" | "Validate", string>>> = {
  Undo: "toolbar-undo",
  Redo: "toolbar-redo",
  Delete: "toolbar-delete",
  Validate: "toolbar-validate",
};

const clickTool = (page: Page, tool: "Select" | "Pan" | "Edge" | "Subgraph" | "Text") =>
  effectStep(() => page.getByTestId(toolIds[tool]).click());

const clickPanelToggle = (page: Page, toggle: "Icons" | "Props" | "Mini" | "Valid") =>
  effectStep(() => page.getByTestId(panelToggleIds[toggle]).click());

const toolbarButton = (
  page: Page,
  action: "Undo" | "Redo" | "Delete" | "Auto-Arrange" | "Validate",
): Locator => {
  const testId = toolbarIds[action];
  return testId
    ? page.getByTestId(testId)
    : page.getByRole("button", { name: action, exact: true });
};

const clickToolbar = (page: Page, action: "Undo" | "Redo" | "Delete" | "Auto-Arrange" | "Validate") =>
  effectStep(() => toolbarButton(page, action).click());

const getCanvasBox = async (page: Page) => {
  const canvas = page.getByTestId("canvas-root");
  const box = await canvas.boundingBox();
  return box ?? { x: 0, y: 0, width: 800, height: 600 };
};

const waitForAnimationFrames = (page: Page, frames = 2) =>
  effectStep(() =>
    page.evaluate((count) => {
      const target = Math.max(1, Math.floor(count));
      return new Promise<void>((resolve) => {
        let remaining = target;
        const tick = () => {
          remaining -= 1;
          if (remaining <= 0) {
            resolve();
            return;
          }
          requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      });
    }, frames),
  );

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
      return effectStep(async () => {
        const box = await getCanvasBox(page);
        const x = box.x + box.width * 0.55;
        const y = box.y + box.height * 0.4;
        await page.mouse.move(x, y);
        await page.mouse.down();
        for (let i = 0; i < op.intensity; i += 1) {
          await page.mouse.move(x + (i % 2 === 0 ? 12 : -12), y + (i % 2 === 0 ? 8 : -8), { steps: 3 });
        }
        await page.mouse.up();
      });

    case "zoom_pulse":
      return repeated(
        op.intensity,
        Effect.all(
          [
            effectStep(() => page.getByTestId("zoom-in").click()),
            effectStep(() => page.getByTestId("zoom-out").click()),
          ],
          { concurrency: 1, discard: true },
        ),
      ).pipe(Effect.orDie);

    case "undo_redo_fork":
      return repeated(
        op.intensity,
        Effect.all([clickToolbar(page, "Undo"), clickToolbar(page, "Redo")], {
          concurrency: 1,
          discard: true,
        }),
      ).pipe(Effect.orDie);

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
        ),
      ).pipe(Effect.orDie);

    case "zoom_wheel_burst":
      return effectStep(async () => {
        const box = await getCanvasBox(page);
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
        for (let i = 0; i < op.intensity * 3; i += 1) {
          await page.mouse.wheel(0, i % 2 === 0 ? -120 : 120);
        }
      });

    case "select_all_deselect":
      return repeated(
        op.intensity,
        Effect.all(
          [
            effectStep(() => page.keyboard.press("ControlOrMeta+a")),
            effectStep(() => page.keyboard.press("Escape")),
          ],
          { concurrency: 1, discard: true },
        ),
      ).pipe(Effect.orDie);

    case "edge_create_cancel":
      return effectStep(async () => {
        await page.getByTestId("tool-edge").click();
        const nodes = await page.getByTestId("node").all();
        if (nodes.length >= 1) {
          await nodes[0].click();
          await page.keyboard.press("Escape");
        }
        await page.getByTestId("tool-select").click();
      });

    case "subgraph_collapse_expand":
      return effectStep(async () => {
        const subgraphs = await page.getByTestId("node").filter({ hasText: "Subgraph" }).all();
        for (let i = 0; i < Math.min(subgraphs.length, op.intensity); i += 1) {
          await subgraphs[i].dblclick();
        }
      });

    case "clipboard_copy_paste_cycle":
      return repeated(
        op.intensity,
        Effect.all(
          [
            effectStep(() => page.keyboard.press("ControlOrMeta+c")),
            effectStep(() => page.keyboard.press("ControlOrMeta+v")),
          ],
          { concurrency: 1, discard: true },
        ),
      ).pipe(Effect.orDie);

    case "import_export_roundtrip":
      return effectStep(async () => {
        const exportBtn = page.getByRole("button", { name: /Export JSON/ });
        if (await exportBtn.isVisible().catch(() => false)) {
          const [download] = await Promise.all([
            page.waitForEvent("download", { timeout: 5000 }).catch(() => null),
            exportBtn.click().catch(() => {}),
          ]);
          void download;
        }
      });

    case "keyboard_shortcut_fuzz":
      return effectStep(async () => {
        const keys = ["v", "h", "e", "s", "t", "Delete", "Backspace", "Escape", "0"];
        for (let i = 0; i < op.intensity; i += 1) {
          for (const key of keys) {
            await page.keyboard.press(key);
          }
        }
      });

    case "pan_drag_race":
      return effectStep(async () => {
        const box = await getCanvasBox(page);
        const cx = box.x + box.width / 2;
        const cy = box.y + box.height / 2;
        await page.keyboard.down(" ");
        await page.mouse.move(cx, cy);
        await page.mouse.down();
        for (let i = 0; i < op.intensity; i += 1) {
          const offset = (i % 2 === 0 ? 50 : -50) * (i + 1);
          await page.mouse.move(cx + offset, cy);
        }
        await page.mouse.up();
        await page.keyboard.up(" ");
      });

    case "validation_toggle_storm":
      return repeated(
        op.intensity,
        Effect.all(
          [
            clickToolbar(page, "Validate"),
            clickPanelToggle(page, "Valid"),
            clickToolbar(page, "Validate"),
          ],
          { concurrency: 1, discard: true },
        ),
      ).pipe(Effect.orDie);

    case "minimap_viewport_drag":
      return effectStep(async () => {
        await page.getByTestId("panel-mini-toggle").click();
        const minimap = page.getByTestId("minimap-viewport");
        const box = await minimap.boundingBox();
        if (box) {
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.mouse.down();
          await page.mouse.move(box.x + box.width / 2 + 30, box.y + box.height / 2, { steps: 5 });
          await page.mouse.up();
        }
      });

    case "resize_handle_corner_se":
      return effectStep(async () => {
        const handle = page.getByTestId("resize-handle-se").first();
        const box = await handle.boundingBox();
        if (box) {
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.mouse.down();
          await page.mouse.move(
            box.x + box.width / 2 + 40 * op.intensity,
            box.y + box.height / 2 + 30 * op.intensity,
            { steps: 8 },
          );
          await page.mouse.up();
        }
      });

    case "resize_handle_corner_nw":
      return effectStep(async () => {
        const handle = page.getByTestId("resize-handle-nw").first();
        const box = await handle.boundingBox();
        if (box) {
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.mouse.down();
          await page.mouse.move(
            box.x + box.width / 2 - 30 * op.intensity,
            box.y + box.height / 2 - 20 * op.intensity,
            { steps: 8 },
          );
          await page.mouse.up();
        }
      });

    case "theme_flip_during_gesture":
      return effectStep(async () => {
        const theme = page.getByRole("combobox").first();
        const node = page.getByTestId("node").first();
        const box = await node.boundingBox();
        if (box) {
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.mouse.down();
          await theme.selectOption({ index: 0 }).catch(() => {});
          await page.mouse.move(box.x + box.width / 2 + 50, box.y + box.height / 2, { steps: 5 });
          await theme.selectOption({ index: 1 }).catch(() => {});
          await page.mouse.up();
        }
      });

    case "delete_undelete_cycle":
      return repeated(
        op.intensity,
        Effect.all(
          [
            effectStep(() => page.keyboard.press("Delete")),
            clickToolbar(page, "Undo"),
          ],
          { concurrency: 1, discard: true },
        ),
      ).pipe(Effect.orDie);

    default:
      return Effect.void;
  }
};

export async function runTrace(page: Page, trace: RedQueenTrace): Promise<void> {
  const program = trace.operators.reduce(
    (acc, op) => acc.pipe(Effect.zipRight(opProgram(page, op))),
    Effect.void,
  );
  await Effect.runPromise(program);
}

export async function runTraceWithRetry(
  page: Page,
  trace: RedQueenTrace,
  maxRetries = 2,
): Promise<{ success: boolean; attempts: number }> {
  let attempts = 0;
  while (attempts <= maxRetries) {
    attempts += 1;
    try {
      await runTrace(page, trace);
      return { success: true, attempts };
    } catch (error) {
      if (attempts > maxRetries) {
        return { success: false, attempts };
      }
      await Effect.runPromise(waitForAnimationFrames(page, 2));
    }
  }
  return { success: false, attempts };
}
