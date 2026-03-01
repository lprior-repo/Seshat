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

async function setupSubgraphWithChildNode(page: Page): Promise<Locator> {
  await freshStart(page);
  await clearCanvasOverlays(page);
  const canvas = page.getByTestId("canvas-root");

  // Create a text node that will be inside the subgraph
  await runEffectsSequential([
    () => createTextNode(page, canvas, 620, 280),
  ]);
  expect(await nodeCount(page)).toBe(1);

  // Create subgraph around the node
  await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
  const canvasBox = await requireBox(canvas);
  const sx = canvasBox.x + 560;
  const sy = canvasBox.y + 220;
  const ex = canvasBox.x + 760;
  const ey = canvasBox.y + 380;
  await runEffectsSequential([
    () => page.mouse.move(sx, sy),
    () => page.mouse.down(),
    () => page.mouse.move(ex, ey, { steps: 8 }),
    () => page.mouse.up(),
  ]);

  expect(await nodeCount(page)).toBe(2); // 1 text node + 1 subgraph
  await runEffect(() => page.getByRole("button", { name: "Select", exact: true }).click());
  return canvas;
}

async function dragMouse(page: Page, from: { x: number; y: number }, to: { x: number; y: number }) {
  await runEffectsSequential([
    () => page.mouse.move(from.x, from.y),
    () => page.mouse.down(),
    () => page.mouse.move(to.x, to.y, { steps: 8 }),
    () => page.mouse.up(),
  ]);
}

test.describe("SUB subgraph container behavior", () => {
  // SUB-011: Container auto-expand when child crosses boundary
  test("container handles child crossing boundary gracefully @behavior", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithChildNode(page);

    // Get initial positions
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const childBefore = await nodeFrameByLabel(page, "Text", 0);

    // Verify child is inside container initially
    expect(childBefore.x).toBeGreaterThan(subBefore.x);
    expect(childBefore.y).toBeGreaterThan(subBefore.y);
    expect(childBefore.x + childBefore.width).toBeLessThan(subBefore.x + subBefore.width);
    expect(childBefore.y + childBefore.height).toBeLessThan(subBefore.y + subBefore.height);

    // Select the child node
    const nodes = canvas.getByTestId("node");
    // First node should be the subgraph, second should be the text
    const textNode = nodes.filter({ hasText: "Text" }).first();
    await runEffect(() => textNode.click());
    expect(await selectedCount(page)).toBe(1);

    // Calculate drag to move child toward the right edge of container
    const childCenter = await center(childBefore);
    const rightEdgeOfContainer = subBefore.x + subBefore.width;
    const dragDistance = rightEdgeOfContainer - childCenter.x - 20; // Move close to edge

    const dragEnd = {
      x: childCenter.x + dragDistance,
      y: childCenter.y,
    };

    await dragMouse(page, childCenter, dragEnd);

    // Get final positions
    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const childAfter = await nodeFrameByLabel(page, "Text", 0);

    // Verify the system handled the boundary crossing gracefully:
    // Either container expanded OR child is constrained within bounds
    const childIsContained =
      childAfter.x >= subAfter.x &&
      childAfter.y >= subAfter.y &&
      childAfter.x + childAfter.width <= subAfter.x + subAfter.width + 10 && // tolerance
      childAfter.y + childAfter.height <= subAfter.y + subAfter.height + 10;

    const containerExpanded =
      subAfter.width > subBefore.width || subAfter.height > subBefore.height;

    // One of these should be true for graceful handling
    expect(childIsContained || containerExpanded).toBe(true);

    // Verify no rendering artifacts (dimensions are valid)
    expect(Number.isFinite(subAfter.width)).toBe(true);
    expect(Number.isFinite(subAfter.height)).toBe(true);
    expect(Number.isFinite(childAfter.width)).toBe(true);
    expect(Number.isFinite(childAfter.height)).toBe(true);
    expect(subAfter.width).toBeGreaterThan(0);
    expect(subAfter.height).toBeGreaterThan(0);

    expect(pageErrors).toHaveLength(0);
  });

  // SUB-012: Container resize behavior (children keep size vs scale)
  test("children maintain size when container is resized independently @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithChildNode(page);

    // Get initial dimensions
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const childBefore = await nodeFrameByLabel(page, "Text", 0);
    const childWidthBefore = childBefore.width;
    const childHeightBefore = childBefore.height;

    // Select only the subgraph (not the child)
    const subgraphNode = canvas.getByTestId("node").filter({ hasText: "Subgraph" }).first();
    await runEffect(() => subgraphNode.click());
    expect(await selectedCount(page)).toBe(1);

    // Get the SE resize handle (should be for the subgraph only)
    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);

    // Resize the subgraph larger
    await dragMouse(page, seCenter, { x: seCenter.x + 100, y: seCenter.y + 80 });

    // Get final dimensions
    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const childAfter = await nodeFrameByLabel(page, "Text", 0);

    // Container should have grown
    expect(subAfter.width).toBeGreaterThan(subBefore.width);
    expect(subAfter.height).toBeGreaterThan(subBefore.height);

    // Child should maintain its size (not scale with container)
    // Allow small tolerance for rounding
    expect(Math.abs(childAfter.width - childWidthBefore)).toBeLessThan(5);
    expect(Math.abs(childAfter.height - childHeightBefore)).toBeLessThan(5);

    // Child position relative to container may shift, but size stays constant
    expect(Number.isFinite(childAfter.x)).toBe(true);
    expect(Number.isFinite(childAfter.y)).toBe(true);

    expect(pageErrors).toHaveLength(0);
  });

  // SUB-013: Container overflow handling
  test("container handles overflow when shrunk smaller than children @behavior", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);
    const canvas = page.getByTestId("canvas-root");

    // Create two text nodes that will be inside the subgraph
    await runEffectsSequential([
      () => createTextNode(page, canvas, 600, 250),
      () => createTextNode(page, canvas, 720, 250),
    ]);
    expect(await nodeCount(page)).toBe(2);

    // Create a large subgraph around both nodes
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    const canvasBox = await requireBox(canvas);
    const sx = canvasBox.x + 540;
    const sy = canvasBox.y + 200;
    const ex = canvasBox.x + 860;
    const ey = canvasBox.y + 400;
    await runEffectsSequential([
      () => page.mouse.move(sx, sy),
      () => page.mouse.down(),
      () => page.mouse.move(ex, ey, { steps: 8 }),
      () => page.mouse.up(),
    ]);

    expect(await nodeCount(page)).toBe(3); // 2 text nodes + 1 subgraph
    await runEffect(() => page.getByRole("button", { name: "Select", exact: true }).click());

    // Get initial positions
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const child1Before = await nodeFrameByLabel(page, "Text", 0);
    const child2Before = await nodeFrameByLabel(page, "Text", 1);

    // Select only the subgraph
    const subgraphNode = canvas.getByTestId("node").filter({ hasText: "Subgraph" }).first();
    await runEffect(() => subgraphNode.click());
    expect(await selectedCount(page)).toBe(1);

    // Get the NW resize handle to shrink from the right/bottom
    // We'll use the SE handle and drag it up/left to shrink
    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);

    // Shrink the container significantly (may make it smaller than children)
    await dragMouse(page, seCenter, { x: seCenter.x - 150, y: seCenter.y - 100 });

    // Get final positions
    const subAfter = await nodeFrameByLabel(page, "Subgraph");

    // Verify the container dimensions are valid (no NaN, Infinity, etc.)
    expect(Number.isFinite(subAfter.width)).toBe(true);
    expect(Number.isFinite(subAfter.height)).toBe(true);
    expect(Number.isFinite(subAfter.x)).toBe(true);
    expect(Number.isFinite(subAfter.y)).toBe(true);
    expect(subAfter.width).toBeGreaterThan(0);
    expect(subAfter.height).toBeGreaterThan(0);

    // Verify children still exist and have valid dimensions
    const child1After = await nodeFrameByLabel(page, "Text", 0);
    const child2After = await nodeFrameByLabel(page, "Text", 1);

    expect(Number.isFinite(child1After.width)).toBe(true);
    expect(Number.isFinite(child1After.height)).toBe(true);
    expect(Number.isFinite(child2After.width)).toBe(true);
    expect(Number.isFinite(child2After.height)).toBe(true);

    // Children should still be visible (overflow visible) or consistently clipped
    // Either behavior is acceptable, but no rendering artifacts
    expect(pageErrors).toHaveLength(0);
  });

  // SUB-014: Container padding alignment
  test("container maintains padding alignment with children @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithChildNode(page);

    // Get initial positions
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const childBefore = await nodeFrameByLabel(page, "Text", 0);

    // Calculate initial padding from container edges
    const paddingLeftBefore = childBefore.x - subBefore.x;
    const paddingTopBefore = childBefore.y - subBefore.y;
    const paddingRightBefore = (subBefore.x + subBefore.width) - (childBefore.x + childBefore.width);
    const paddingBottomBefore = (subBefore.y + subBefore.height) - (childBefore.y + childBefore.height);

    // Select only the subgraph
    const subgraphNode = canvas.getByTestId("node").filter({ hasText: "Subgraph" }).first();
    await runEffect(() => subgraphNode.click());
    expect(await selectedCount(page)).toBe(1);

    // Resize the container larger
    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await dragMouse(page, seCenter, { x: seCenter.x + 80, y: seCenter.y + 60 });

    // Get final positions
    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const childAfter = await nodeFrameByLabel(page, "Text", 0);

    // Container should have grown
    expect(subAfter.width).toBeGreaterThan(subBefore.width);
    expect(subAfter.height).toBeGreaterThan(subBefore.height);

    // Calculate final padding
    const paddingLeftAfter = childAfter.x - subAfter.x;
    const paddingTopAfter = childAfter.y - subAfter.y;
    const paddingRightAfter = (subAfter.x + subAfter.width) - (childAfter.x + childAfter.width);
    const paddingBottomAfter = (subAfter.y + subAfter.height) - (childAfter.y + childAfter.height);

    // Child position should remain valid (positive padding)
    expect(paddingLeftAfter).toBeGreaterThan(0);
    expect(paddingTopAfter).toBeGreaterThan(0);

    // When container grows, padding should generally increase or stay similar
    // (child doesn't move when only container is resized)
    // Left and top padding should remain relatively constant (child doesn't move)
    expect(Math.abs(paddingLeftAfter - paddingLeftBefore)).toBeLessThan(10);
    expect(Math.abs(paddingTopAfter - paddingTopBefore)).toBeLessThan(10);

    // Right and bottom padding should increase (container grew)
    expect(paddingRightAfter).toBeGreaterThan(paddingRightBefore - 10);
    expect(paddingBottomAfter).toBeGreaterThan(paddingBottomBefore - 10);

    expect(pageErrors).toHaveLength(0);
  });

  // Additional test: Verify proportional scaling when selecting all vs just container
  test("proportional scaling applies when selecting all including children @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithChildNode(page);

    // Get initial positions
    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const childBefore = await nodeFrameByLabel(page, "Text", 0);

    // Calculate relative position of child within container
    const relXBefore = (childBefore.x - subBefore.x) / subBefore.width;
    const relYBefore = (childBefore.y - subBefore.y) / subBefore.height;

    // Select all (container + children)
    await runEffect(() => page.keyboard.press("Control+a"));
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(2);

    // Resize using SE handle
    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await dragMouse(page, seCenter, { x: seCenter.x + 100, y: seCenter.y + 80 });

    // Get final positions
    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const childAfter = await nodeFrameByLabel(page, "Text", 0);

    // Container should have grown
    expect(subAfter.width).toBeGreaterThan(subBefore.width);
    expect(subAfter.height).toBeGreaterThan(subBefore.height);

    // Calculate relative position after resize
    const relXAfter = (childAfter.x - subAfter.x) / subAfter.width;
    const relYAfter = (childAfter.y - subAfter.y) / subAfter.height;

    // When selecting all, children should scale proportionally
    // Relative position should be approximately preserved
    expect(Math.abs(relXAfter - relXBefore)).toBeLessThanOrEqual(0.25);
    expect(Math.abs(relYAfter - relYBefore)).toBeLessThanOrEqual(0.25);

    expect(pageErrors).toHaveLength(0);
  });
});
