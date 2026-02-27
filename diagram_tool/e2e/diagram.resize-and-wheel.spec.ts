import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeCount,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForUiReady,
  zoomPercent,
} from "./helpers";

type BoundingBox = { x: number; y: number; width: number; height: number };

async function firstNodeBox(page: Page): Promise<BoundingBox> {
  const node = page.getByTestId("canvas-root").getByText("Text", { exact: true }).first();
  const box = await runEffect(() => node.boundingBox());
  if (!box) {
    throw new Error("node bounds unavailable");
  }
  return box;
}

test.describe("diagram resize and wheel behavior", () => {
  test("wheel on canvas zooms editor and does not scroll page @p0-smoke", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    expect(await zoomPercent(page)).toBe(100);

    const canvas = page.getByTestId("canvas-root");
    const box = await runEffect(() => canvas.boundingBox());
    if (!box) {
      throw new Error("canvas bounds unavailable");
    }

    const beforeScroll = await runEffect(() => page.evaluate(() => window.scrollY));
    await runEffectsSequential([
      () => page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.4),
      () => page.mouse.wheel(0, -180),
    ]);
    expect(await zoomPercent(page)).not.toBe(100);

    const afterScroll = await runEffect(() => page.evaluate(() => window.scrollY));
    expect(afterScroll).toBe(beforeScroll);
    expect(pageErrors).toHaveLength(0);
  });

  test("resize interaction updates dimensions progressively and stays finite", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 620, 280));
    expect(await nodeCount(page)).toBe(1);

    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

    const before = await firstNodeBox(page);
    const east = await runEffect(() =>
      canvas.getByTestId("resize-handle-e").first().boundingBox(),
    );
    if (!east) {
      throw new Error("resize handle bounds unavailable");
    }

    await runEffectsSequential([
      () => page.mouse.move(east.x + east.width / 2, east.y + east.height / 2),
      () => page.mouse.down(),
      () => page.mouse.move(east.x - 60, east.y, { steps: 4 }),
    ]);
    const mid = await firstNodeBox(page);
    await runEffectsSequential([
      () => page.mouse.move(east.x - 140, east.y, { steps: 4 }),
      () => page.mouse.up(),
    ]);
    const after = await firstNodeBox(page);

    expect(Number.isFinite(mid.width)).toBe(true);
    expect(Number.isFinite(after.width)).toBe(true);
    expect(mid.width).toBeLessThanOrEqual(before.width);
    expect(after.width).toBeLessThanOrEqual(mid.width);
    expect(after.width).toBeGreaterThanOrEqual(24);
    expect(pageErrors).toHaveLength(0);
  });

  test("small handle drag does not jump from viewport-offset math", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 700, 320));
    expect(await nodeCount(page)).toBe(1);

    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());

    const before = await firstNodeBox(page);
    const east = await runEffect(() =>
      canvas.getByTestId("resize-handle-e").first().boundingBox(),
    );
    if (!east) {
      throw new Error("east resize handle unavailable");
    }

    const cx = east.x + east.width / 2;
    const cy = east.y + east.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(cx, cy),
      () => page.mouse.down(),
      () => page.mouse.move(cx - 12, cy, { steps: 3 }),
      () => page.mouse.up(),
    ]);

    const after = await firstNodeBox(page);
    const delta = Math.abs(after.width - before.width);

    expect(delta).toBeGreaterThanOrEqual(0);
    expect(delta).toBeLessThanOrEqual(40);
    expect(pageErrors).toHaveLength(0);
  });
});
