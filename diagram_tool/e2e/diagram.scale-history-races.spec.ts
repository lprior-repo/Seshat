import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeCount,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

type Box = { x: number; y: number; width: number; height: number };

async function requireBox(target: Locator): Promise<Box> {
  const box = await runEffect(() => target.boundingBox());
  if (!box) {
    throw new Error("expected bounding box");
  }
  return box;
}

async function eastHandle(canvas: Locator): Promise<Box> {
  return requireBox(canvas.locator('[data-testid="selection-handle"][data-handle="e"]').first());
}

async function setupSingleNode(page: Page) {
  await runEffect(() => page.goto("/"));
  await runEffect(() => waitForUiReady(page));
  await runEffect(() => clearCanvasOverlays(page));

  const canvas = page.getByTestId("canvas-container");
  await runEffect(() => createTextNode(page, canvas, 680, 300));
  expect(await nodeCount(page)).toBe(1);

  const node = canvas.getByText("Text", { exact: true }).first();
  await runEffect(() => node.click());
  expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

  return { canvas, node };
}

test.describe("scale history and race safety", () => {
  test("one long resize gesture maps to one undo and one redo", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const { canvas, node } = await setupSingleNode(page);

    const before = await requireBox(node);
    const handle = await eastHandle(canvas);
    const hx = handle.x + handle.width / 2;
    const hy = handle.y + handle.height / 2;

    await runEffect(() => page.mouse.move(hx, hy));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(hx - 30, hy, { steps: 3 }));
    await runEffect(() => page.mouse.move(hx - 70, hy, { steps: 4 }));
    await runEffect(() => page.mouse.move(hx - 110, hy, { steps: 5 }));
    await runEffect(() => page.mouse.up());

    const after = await requireBox(node);
    expect(after.width).toBeLessThanOrEqual(before.width);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    const undone = await requireBox(node);
    expect(Math.abs(undone.width - before.width)).toBeLessThanOrEqual(2);

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    const redone = await requireBox(node);
    expect(Math.abs(redone.width - after.width)).toBeLessThanOrEqual(2);
    expect(pageErrors).toHaveLength(0);
  });

  test("release pointer outside canvas finalizes resize cleanly", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const { canvas, node } = await setupSingleNode(page);

    const before = await requireBox(node);
    const handle = await eastHandle(canvas);
    const hx = handle.x + handle.width / 2;
    const hy = handle.y + handle.height / 2;
    const canvasBox = await requireBox(canvas);

    await runEffect(() => page.mouse.move(hx, hy));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(canvasBox.x - 60, canvasBox.y - 40, { steps: 6 }));
    await runEffect(() => page.mouse.up());

    const after = await requireBox(node);
    expect(Number.isFinite(after.width)).toBe(true);
    expect(after.width).toBeGreaterThanOrEqual(24);
    expect(Math.abs(after.width - before.width)).toBeLessThanOrEqual(220);

    await runEffect(() =>
      page.mouse.click(after.x + after.width / 2, after.y + after.height / 2),
    );
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(pageErrors).toHaveLength(0);
  });
});
