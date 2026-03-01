import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  freshStart,
  nodeCount,
  nodeFrameByLabel,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  zoomPercent,
} from "./helpers";

type Box = { x: number; y: number; width: number; height: number };

async function requireBox(target: Locator): Promise<Box> {
  const box = await runEffect(() => target.boundingBox());
  if (!box) {
    throw new Error("expected bounding box");
  }
  return box;
}

async function zoomToAtLeast(page: Page, target: number) {
  for (let i = 0; i < 12; i += 1) {
    const current = await zoomPercent(page);
    if (current >= target) {
      return;
    }
    await runEffect(() => page.getByRole("button", { name: "+", exact: true }).first().click());
  }
}

async function southEastHandle(canvas: Locator): Promise<Box> {
  const selected = await runEffect(() =>
    canvas.getByTestId("resize-handle-se").first().boundingBox(),
  );
  if (!selected) {
    throw new Error("missing south-east handle");
  }
  return selected;
}

async function worldWidth(box: Box, zoomPct: number): Promise<number> {
  return box.width / (zoomPct / 100);
}

async function runResizeScenario(
  page: Page,
  zoomTarget: number,
  dragPixels: number,
): Promise<{ beforeWorld: number; afterWorld: number; deltaWorld: number }> {
  await freshStart(page);
  await clearCanvasOverlays(page);

  const canvas = page.getByTestId("canvas-root");
  await runEffect(() => createTextNode(page, canvas, 680, 300));
  const node = canvas.getByTestId("node").first();
  await runEffect(() => node.click());
  expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

  await zoomToAtLeast(page, zoomTarget);
  const before = await nodeFrameByLabel(page, "Text", 0);
  const zoomBefore = await zoomPercent(page);
  const beforeWorld = (await worldWidth(before, zoomBefore)) + (before.height / (zoomBefore / 100));

  const handle = await southEastHandle(canvas);
  const hx = handle.x + handle.width / 2;
  const hy = handle.y + handle.height / 2;
  await runEffectsSequential([
    () => page.mouse.move(hx, hy),
    () => page.mouse.down(),
    () => page.mouse.move(hx + dragPixels, hy + dragPixels * 0.7, { steps: 6 }),
    () => page.mouse.up(),
  ]);

  const after = await nodeFrameByLabel(page, "Text", 0);
  const zoomAfter = await zoomPercent(page);
  const afterWorld = (await worldWidth(after, zoomAfter)) + (after.height / (zoomAfter / 100));

  return {
    beforeWorld,
    afterWorld,
    deltaWorld: afterWorld - beforeWorld,
  };
}

test.describe("zoom/scale consistency", () => {
  test("equivalent world resize delta at 100% and 400% remains close", async ({ page }) => {
    const pageErrors = trapPageErrors(page);

    const at100 = await runResizeScenario(page, 100, 40);
    const at400 = await runResizeScenario(page, 380, 160);

    expect(at100.deltaWorld).toBeGreaterThan(1);
    expect(at400.deltaWorld).toBeGreaterThan(1);
    expect(Math.abs(at100.deltaWorld - at400.deltaWorld)).toBeLessThanOrEqual(12);
    expect(pageErrors).toHaveLength(0);
  });

  test("after resize at high zoom node remains selectable by center click", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 700, 320));
    const node = canvas.getByTestId("node").first();
    await runEffect(() => node.click());
    expect(await nodeCount(page)).toBe(1);
    await zoomToAtLeast(page, 300);

    const handle = await southEastHandle(canvas);
    const hx = handle.x + handle.width / 2;
    const hy = handle.y + handle.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(hx, hy),
      () => page.mouse.down(),
      () => page.mouse.move(hx - 120, hy, { steps: 6 }),
      () => page.mouse.up(),
    ]);

    const nodeAfter = await nodeFrameByLabel(page, "Text", 0);
    await runEffect(() =>
      page.mouse.click(nodeAfter.x + nodeAfter.width / 2, nodeAfter.y + nodeAfter.height / 2),
    );
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(pageErrors).toHaveLength(0);
  });
});
