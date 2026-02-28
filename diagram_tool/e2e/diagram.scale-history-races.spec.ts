import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  freshStart,
  nodeCount,
  runEffect,
  runEffectsSequential,
  selectedCount,
  trapPageErrors,
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
  return requireBox(canvas.getByTestId("resize-handle-se").first());
}

async function setupSingleNode(page: Page) {
  await freshStart(page);
  await clearCanvasOverlays(page);

  const canvas = page.getByTestId("canvas-root");
  const bounds = await requireBox(canvas);
  await runEffect(() => page.mouse.dblclick(bounds.x + 680, bounds.y + 300));
  expect(await nodeCount(page)).toBe(1);

  await runEffect(() => page.getByRole("button", { name: "Select", exact: true }).click());
  const node = canvas.getByTestId("node").first();
  await runEffect(() => node.click());
  expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

  return { canvas, node };
}

test.describe("scale history and race safety", () => {
  test("drag gesture stays single-history under duplicate release events @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const { node } = await setupSingleNode(page);

    const before = await requireBox(node);
    const startX = before.x + before.width / 2;
    const startY = before.y + before.height / 2;
    const endX = startX + 140;
    const endY = startY + 12;

    await runEffectsSequential([
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(endX, endY, { steps: 6 }),
      () => page.mouse.up(),
      () =>
        page.evaluate(
          ({ x, y }) => {
            window.dispatchEvent(
              new PointerEvent("pointerup", {
                clientX: x,
                clientY: y,
                bubbles: true,
              }),
            );
            window.dispatchEvent(
              new MouseEvent("mouseup", {
                clientX: x,
                clientY: y,
                bubbles: true,
              }),
            );
          },
          { x: endX, y: endY },
        ),
    ]);

    const after = await requireBox(node);
    expect(after.x).toBeGreaterThan(before.x + 20);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    const undone = await requireBox(node);
    expect(Math.abs(undone.x - before.x)).toBeLessThanOrEqual(2);

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    const redone = await requireBox(node);
    expect(Math.abs(redone.x - after.x)).toBeLessThanOrEqual(2);
    expect(pageErrors).toHaveLength(0);
  });

  test("one long resize gesture maps to one undo and one redo @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const { canvas, node } = await setupSingleNode(page);

    const before = await requireBox(node);
    const handle = await eastHandle(canvas);
    const hx = handle.x + handle.width / 2;
    const hy = handle.y + handle.height / 2;

    await runEffectsSequential([
      () => page.mouse.move(hx, hy),
      () => page.mouse.down(),
      () => page.mouse.move(hx - 30, hy, { steps: 3 }),
      () => page.mouse.move(hx - 70, hy, { steps: 4 }),
      () => page.mouse.move(hx - 110, hy, { steps: 5 }),
      () => page.mouse.up(),
      () =>
        page.evaluate(
          ({ x, y }) => {
            window.dispatchEvent(
              new PointerEvent("pointerup", {
                clientX: x,
                clientY: y,
                bubbles: true,
              }),
            );
            window.dispatchEvent(
              new MouseEvent("mouseup", {
                clientX: x,
                clientY: y,
                bubbles: true,
              }),
            );
          },
          { x: hx - 110, y: hy },
        ),
    ]);

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

  test("release pointer outside canvas finalizes resize cleanly @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const { canvas, node } = await setupSingleNode(page);

    const before = await requireBox(node);
    const handle = await eastHandle(canvas);
    const hx = handle.x + handle.width / 2;
    const hy = handle.y + handle.height / 2;
    const canvasBox = await requireBox(canvas);

    await runEffectsSequential([
      () => page.mouse.move(hx, hy),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x - 60, canvasBox.y - 40, { steps: 6 }),
      () => page.mouse.up(),
    ]);

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
