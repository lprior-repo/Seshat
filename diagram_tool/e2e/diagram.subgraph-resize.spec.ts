import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeCount,
  nodeFrameByLabel,
  runEffectsSequential,
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

async function pickSouthEastHandle(canvas: Locator): Promise<Box> {
  const handles = canvas.getByTestId("resize-handle-se");
  const count = await runEffect(() => handles.count());
  if (count < 1) {
    throw new Error("no resize handles found");
  }

  const boxes: Box[] = [];
  for (let i = 0; i < count; i += 1) {
    const next = await runEffect(() => handles.nth(i).boundingBox());
    if (next) {
      boxes.push(next);
    }
  }

  const best = boxes[0];
  if (!best) {
    throw new Error("no visible resize handle boxes");
  }
  return best;
}

async function center(box: Box) {
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function setupSubgraphWithNodes(page: Page) {
  await runEffectsSequential([
    () => page.goto("/"),
    () => waitForUiReady(page),
    () => clearCanvasOverlays(page),
  ]);
  const canvas = page.getByTestId("canvas-root");

  await runEffectsSequential([
    () => createTextNode(page, canvas, 620, 250),
    () => createTextNode(page, canvas, 760, 330),
  ]);
  expect(await nodeCount(page)).toBe(2);

  await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
  const canvasBox = await requireBox(canvas);
  const sx = canvasBox.x + 560;
  const sy = canvasBox.y + 210;
  const ex = canvasBox.x + 860;
  const ey = canvasBox.y + 420;
  await runEffectsSequential([
    () => page.mouse.move(sx, sy),
    () => page.mouse.down(),
    () => page.mouse.move(ex, ey, { steps: 8 }),
    () => page.mouse.up(),
  ]);

  expect(await nodeCount(page)).toBe(3);
  return canvas;
}

test.describe("subgraph proportional resize", () => {
  test("resizing subgraph and contained nodes keeps proportions coherent", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithNodes(page);

    await runEffect(() => page.keyboard.press("Control+a"));
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(3);

    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const nodeBefore = await nodeFrameByLabel(page, "Text", 0);

    const relXBefore = (nodeBefore.x - subBefore.x) / subBefore.width;
    const relYBefore = (nodeBefore.y - subBefore.y) / subBefore.height;

    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await runEffectsSequential([
      () => page.mouse.move(seCenter.x, seCenter.y),
      () => page.mouse.down(),
      () => page.mouse.move(seCenter.x + 140, seCenter.y + 110, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const nodeAfter = await nodeFrameByLabel(page, "Text", 0);

    expect(subAfter.width).toBeGreaterThan(subBefore.width);
    expect(subAfter.height).toBeGreaterThan(subBefore.height);

    const relXAfter = (nodeAfter.x - subAfter.x) / subAfter.width;
    const relYAfter = (nodeAfter.y - subAfter.y) / subAfter.height;

    expect(Math.abs(relXAfter - relXBefore)).toBeLessThanOrEqual(0.2);
    expect(Math.abs(relYAfter - relYBefore)).toBeLessThanOrEqual(0.2);
    expect(pageErrors).toHaveLength(0);
  });

  test("nested subgraph resize preserves inner/outer proportionality", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithNodes(page);

    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    const canvasBox = await requireBox(canvas);
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 640, canvasBox.y + 250),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 800, canvasBox.y + 360, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    expect(await nodeCount(page)).toBe(4);
    await runEffect(() => page.keyboard.press("Control+a"));
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(4);

    const outerBefore = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerBefore = await nodeFrameByLabel(page, "Subgraph", 1);
    const ratioWBefore = innerBefore.width / outerBefore.width;
    const ratioHBefore = innerBefore.height / outerBefore.height;

    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await runEffectsSequential([
      () => page.mouse.move(seCenter.x, seCenter.y),
      () => page.mouse.down(),
      () => page.mouse.move(seCenter.x + 160, seCenter.y + 120, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    const outerAfter = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerAfter = await nodeFrameByLabel(page, "Subgraph", 1);
    const ratioWAfter = innerAfter.width / outerAfter.width;
    const ratioHAfter = innerAfter.height / outerAfter.height;

    expect(Math.abs(ratioWAfter - ratioWBefore)).toBeLessThanOrEqual(0.2);
    expect(Math.abs(ratioHAfter - ratioHBefore)).toBeLessThanOrEqual(0.2);
    expect(pageErrors).toHaveLength(0);
  });
});
