import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeFrameByLabel,
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

async function zoomPercent(page: Page): Promise<number> {
  const text = await runEffect(() =>
    page.getByRole("button", { name: /\d+%/ }).first().innerText(),
  );
  const parsed = Number.parseInt(text, 10);
  return Number.isNaN(parsed) ? 100 : parsed;
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
  const handles = canvas.locator('button[style*="nwse-resize"], button[style*="nesw-resize"]');
  const count = await runEffect(() => handles.count());
  const boxes: Box[] = [];
  for (let i = 0; i < count; i += 1) {
    const next = await runEffect(() => handles.nth(i).boundingBox());
    if (next) {
      boxes.push(next);
    }
  }
  const selected = boxes.sort((a, b) => (a.x + a.y) - (b.x + b.y)).pop();
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
  await runEffect(() => page.goto("/"));
  await runEffect(() => waitForUiReady(page));
  await runEffect(() => clearCanvasOverlays(page));

  const canvas = page.locator(".canvas-container");
  await runEffect(() => createTextNode(page, canvas, 680, 300));
  const node = canvas.getByText("Text", { exact: true }).first();
  await runEffect(() => node.click());
  expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

  await zoomToAtLeast(page, zoomTarget);
  const before = await nodeFrameByLabel(page, "Text", 0);
  const zoomBefore = await zoomPercent(page);
  const beforeWorld = (await worldWidth(before, zoomBefore)) + (before.height / (zoomBefore / 100));

  const handle = await southEastHandle(canvas);
  const hx = handle.x + handle.width / 2;
  const hy = handle.y + handle.height / 2;
  await runEffect(() => page.mouse.move(hx, hy));
  await runEffect(() => page.mouse.down());
  await runEffect(() => page.mouse.move(hx + dragPixels, hy + dragPixels * 0.7, { steps: 6 }));
  await runEffect(() => page.mouse.up());

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
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 700, 320));
    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());
    await zoomToAtLeast(page, 300);

    const handle = await southEastHandle(canvas);
    const hx = handle.x + handle.width / 2;
    const hy = handle.y + handle.height / 2;
    await runEffect(() => page.mouse.move(hx, hy));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(hx - 120, hy, { steps: 6 }));
    await runEffect(() => page.mouse.up());

    const nodeAfter = await nodeFrameByLabel(page, "Text", 0);
    await runEffect(() =>
      page.mouse.click(nodeAfter.x + nodeAfter.width / 2, nodeAfter.y + nodeAfter.height / 2),
    );
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(pageErrors).toHaveLength(0);
  });
});
