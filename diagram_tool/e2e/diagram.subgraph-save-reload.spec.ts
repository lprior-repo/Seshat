import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  freshStart,
  nodeCount,
  nodeFrameByLabel,
  runEffectsSequential,
  runEffect,
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
  await freshStart(page);
  await clearCanvasOverlays(page);
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

test.describe("subgraph save-reload stability", () => {
  test("subgraph with nodes survives page reload", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithNodes(page);

    // Get initial positions
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const nodeBefore = await nodeFrameByLabel(page, "Text", 0);

    expect(subBefore.width).toBeGreaterThan(0);
    expect(subBefore.height).toBeGreaterThan(0);
    expect(nodeBefore.x).toBeGreaterThan(subBefore.x);
    expect(nodeBefore.y).toBeGreaterThan(subBefore.y);

    // Reload the page (simulates save-reload cycle via localStorage auto-save)
    await runEffectsSequential([
      () => page.reload({ waitUntil: "domcontentloaded" }),
      () => waitForUiReady(page),
    ]);

    // Verify subgraph and nodes are restored
    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const nodeAfter = await nodeFrameByLabel(page, "Text", 0);

    // Check dimensions preserved (within tolerance)
    expect(Math.abs(subAfter.width - subBefore.width)).toBeLessThanOrEqual(2);
    expect(Math.abs(subAfter.height - subBefore.height)).toBeLessThanOrEqual(2);

    // Check relative position preserved
    const relXBefore = (nodeBefore.x - subBefore.x) / subBefore.width;
    const relYBefore = (nodeBefore.y - subBefore.y) / subBefore.height;
    const relXAfter = (nodeAfter.x - subAfter.x) / subAfter.width;
    const relYAfter = (nodeAfter.y - subAfter.y) / subAfter.height;

    expect(Math.abs(relXAfter - relXBefore)).toBeLessThanOrEqual(0.1);
    expect(Math.abs(relYAfter - relYBefore)).toBeLessThanOrEqual(0.1);
    expect(pageErrors).toHaveLength(0);
  });

  test("subgraph resize proportions preserved after reload", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithNodes(page);

    // Select all and resize
    await runEffect(() => page.keyboard.press("Control+a"));
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const nodeBefore = await nodeFrameByLabel(page, "Text", 0);

    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await runEffectsSequential([
      () => page.mouse.move(seCenter.x, seCenter.y),
      () => page.mouse.down(),
      () => page.mouse.move(seCenter.x + 140, seCenter.y + 110, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    // Get positions after resize
    const subResized = await nodeFrameByLabel(page, "Subgraph");
    const nodeResized = await nodeFrameByLabel(page, "Text", 0);

    expect(subResized.width).toBeGreaterThan(subBefore.width);
    expect(subResized.height).toBeGreaterThan(subBefore.height);

    // Reload page
    await runEffectsSequential([
      () => page.reload({ waitUntil: "domcontentloaded" }),
      () => waitForUiReady(page),
    ]);

    // Verify proportions preserved after reload
    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const nodeAfter = await nodeFrameByLabel(page, "Text", 0);

    // Calculate relative positions before and after resize (pre-reload)
    const relXBeforeResize = (nodeBefore.x - subBefore.x) / subBefore.width;
    const relYBeforeResize = (nodeBefore.y - subBefore.y) / subBefore.height;
    const relXAfterResize = (nodeResized.x - subResized.x) / subResized.width;
    const relYAfterResize = (nodeResized.y - subResized.y) / subResized.height;

    // Calculate relative positions after reload
    const relXAfterReload = (nodeAfter.x - subAfter.x) / subAfter.width;
    const relYAfterReload = (nodeAfter.y - subAfter.y) / subAfter.height;

    // Relative positions should be consistent across reload
    expect(Math.abs(relXAfterReload - relXAfterResize)).toBeLessThanOrEqual(0.1);
    expect(Math.abs(relYAfterReload - relYAfterResize)).toBeLessThanOrEqual(0.1);
    expect(pageErrors).toHaveLength(0);
  });

  test("nested subgraphs survive page reload", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);
    const canvas = page.getByTestId("canvas-root");

    // Create first text node
    await runEffectsSequential([
      () => createTextNode(page, canvas, 620, 250),
      () => createTextNode(page, canvas, 760, 330),
    ]);
    expect(await nodeCount(page)).toBe(2);

    // Create outer subgraph
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    const canvasBox = await requireBox(canvas);
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 560, canvasBox.y + 210),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 860, canvasBox.y + 420, { steps: 8 }),
      () => page.mouse.up(),
    ]);
    expect(await nodeCount(page)).toBe(3);

    // Create inner subgraph
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 620, canvasBox.y + 260),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 780, canvasBox.y + 380, { steps: 8 }),
      () => page.mouse.up(),
    ]);
    expect(await nodeCount(page)).toBe(4);

    // Get positions before reload
    const outerBefore = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerBefore = await nodeFrameByLabel(page, "Subgraph", 1);

    // Reload page
    await runEffectsSequential([
      () => page.reload({ waitUntil: "domcontentloaded" }),
      () => waitForUiReady(page),
    ]);

    // Verify nested structure preserved
    const outerAfter = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerAfter = await nodeFrameByLabel(page, "Subgraph", 1);

    // Outer should contain inner
    expect(innerAfter.x).toBeGreaterThan(outerAfter.x);
    expect(innerAfter.y).toBeGreaterThan(outerAfter.y);
    expect(innerAfter.x + innerAfter.width).toBeLessThan(outerAfter.x + outerAfter.width);
    expect(innerAfter.y + innerAfter.height).toBeLessThan(outerAfter.y + outerAfter.height);

    // Proportions should be preserved
    const ratioWBefore = innerBefore.width / outerBefore.width;
    const ratioHBefore = innerBefore.height / outerBefore.height;
    const ratioWAfter = innerAfter.width / outerAfter.width;
    const ratioHAfter = innerAfter.height / outerAfter.height;

    expect(Math.abs(ratioWAfter - ratioWBefore)).toBeLessThanOrEqual(0.1);
    expect(Math.abs(ratioHAfter - ratioHBefore)).toBeLessThanOrEqual(0.1);
    expect(pageErrors).toHaveLength(0);
  });

  test("nested subgraph resize proportions preserved after reload", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);
    const canvas = page.getByTestId("canvas-root");

    // Create text nodes
    await runEffectsSequential([
      () => createTextNode(page, canvas, 620, 250),
      () => createTextNode(page, canvas, 760, 330),
    ]);

    // Create outer subgraph
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    const canvasBox = await requireBox(canvas);
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 560, canvasBox.y + 210),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 860, canvasBox.y + 420, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    // Create inner subgraph
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 620, canvasBox.y + 260),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 780, canvasBox.y + 380, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    // Select all and resize
    await runEffect(() => page.keyboard.press("Control+a"));

    const outerBefore = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerBefore = await nodeFrameByLabel(page, "Subgraph", 1);

    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await runEffectsSequential([
      () => page.mouse.move(seCenter.x, seCenter.y),
      () => page.mouse.down(),
      () => page.mouse.move(seCenter.x + 160, seCenter.y + 120, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    // Get positions after resize
    const outerResized = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerResized = await nodeFrameByLabel(page, "Subgraph", 1);

    // Reload page
    await runEffectsSequential([
      () => page.reload({ waitUntil: "domcontentloaded" }),
      () => waitForUiReady(page),
    ]);

    // Verify inner/outer proportions preserved after reload
    const outerAfter = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerAfter = await nodeFrameByLabel(page, "Subgraph", 1);

    // Calculate proportions before and after resize
    const ratioWBeforeResize = innerBefore.width / outerBefore.width;
    const ratioHBeforeResize = innerBefore.height / outerBefore.height;
    const ratioWAfterResize = innerResized.width / outerResized.width;
    const ratioHAfterResize = innerResized.height / outerResized.height;

    // After reload
    const ratioWAfterReload = innerAfter.width / outerAfter.width;
    const ratioHAfterReload = innerAfter.height / outerAfter.height;

    // Ratios should be consistent across reload
    expect(Math.abs(ratioWAfterReload - ratioWAfterResize)).toBeLessThanOrEqual(0.1);
    expect(Math.abs(ratioHAfterReload - ratioHAfterResize)).toBeLessThanOrEqual(0.1);
    expect(pageErrors).toHaveLength(0);
  });
});
