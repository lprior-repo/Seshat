import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
} from "./helpers";

type Box = {
  x: number;
  y: number;
  width: number;
  height: number;
};

async function getSelectionBounds(canvas: Locator): Promise<Box> {
  const bounds = canvas.getByTestId("selection-bounds").first();
  const box = await runEffect(() => bounds.boundingBox());
  if (!box) {
    throw new Error("selection bounds not available");
  }
  return box;
}

async function getResizeHandle(canvas: Locator, corner: "nw" | "ne" | "se" | "sw"): Promise<Box> {
  const handle = canvas.getByTestId(`resize-handle-${corner}`).first();
  const box = await runEffect(() => handle.boundingBox());
  if (!box) {
    throw new Error(`resize handle ${corner} not available`);
  }
  return box;
}

async function dragHandle(
  page: Page,
  handleBox: Box,
  dx: number,
  dy: number,
  steps = 6,
) {
  const cx = handleBox.x + handleBox.width / 2;
  const cy = handleBox.y + handleBox.height / 2;
  await runEffectsSequential([
    () => page.mouse.move(cx, cy),
    () => page.mouse.down(),
    () => page.mouse.move(cx + dx, cy + dy, { steps }),
    () => page.mouse.up(),
  ]);
}

async function selectMultipleNodes(page: Page, canvas: Locator, count: number) {
  const nodes = canvas.getByTestId("node");
  await runEffect(() => nodes.first().click());
  for (let i = 1; i < count; i++) {
    await runEffectsSequential([
      () => page.keyboard.down("Shift"),
      () => nodes.nth(i).click(),
      () => page.keyboard.up("Shift"),
    ]);
  }
}

test.describe("diagram multi-select resize", () => {
  // MUL-006: Resize from NW/NE/SE/SW corners
  test("MUL-006: resize from NW corner handle resizes selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 250),
      () => createTextNode(page, canvas, 650, 300),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const beforeBounds = await getSelectionBounds(canvas);
    const nwHandle = await getResizeHandle(canvas, "nw");

    // Drag NW handle inward (toward center) to shrink selection
    await dragHandle(page, nwHandle, 30, 20);

    const afterBounds = await getSelectionBounds(canvas);

    // Selection should have shrunk from the NW corner
    expect(afterBounds.x).toBeGreaterThan(beforeBounds.x);
    expect(afterBounds.y).toBeGreaterThan(beforeBounds.y);
    expect(afterBounds.width).toBeLessThan(beforeBounds.width);
    expect(afterBounds.height).toBeLessThan(beforeBounds.height);

    // Selection should still have 2 nodes selected
    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  test("MUL-006: resize from NE corner handle resizes selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 250),
      () => createTextNode(page, canvas, 650, 300),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const beforeBounds = await getSelectionBounds(canvas);
    const neHandle = await getResizeHandle(canvas, "ne");

    // Drag NE handle inward (left and down) to shrink selection
    await dragHandle(page, neHandle, -30, 20);

    const afterBounds = await getSelectionBounds(canvas);

    // Selection should have shrunk from the NE corner
    expect(afterBounds.y).toBeGreaterThan(beforeBounds.y);
    expect(afterBounds.width).toBeLessThan(beforeBounds.width);
    expect(afterBounds.height).toBeLessThan(beforeBounds.height);

    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  test("MUL-006: resize from SE corner handle resizes selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 250),
      () => createTextNode(page, canvas, 650, 300),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const beforeBounds = await getSelectionBounds(canvas);
    const seHandle = await getResizeHandle(canvas, "se");

    // Drag SE handle outward (right and down) to expand selection
    await dragHandle(page, seHandle, 50, 40);

    const afterBounds = await getSelectionBounds(canvas);

    // Selection should have expanded from the SE corner
    expect(afterBounds.width).toBeGreaterThan(beforeBounds.width);
    expect(afterBounds.height).toBeGreaterThan(beforeBounds.height);

    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  test("MUL-006: resize from SW corner handle resizes selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 250),
      () => createTextNode(page, canvas, 650, 300),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const beforeBounds = await getSelectionBounds(canvas);
    const swHandle = await getResizeHandle(canvas, "sw");

    // Drag SW handle inward (right and up) to shrink selection
    await dragHandle(page, swHandle, 30, -20);

    const afterBounds = await getSelectionBounds(canvas);

    // Selection should have shrunk from the SW corner
    expect(afterBounds.x).toBeGreaterThan(beforeBounds.x);
    expect(afterBounds.width).toBeLessThan(beforeBounds.width);
    expect(afterBounds.height).toBeLessThan(beforeBounds.height);

    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  // MUL-007: Multi-select resize maintains relative positions
  test("MUL-007: resize maintains node positions within selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 400, 200),
      () => createTextNode(page, canvas, 600, 300),
    ]);
    await expectNodeCount(page, 2);

    // Get initial node positions
    const nodes = canvas.getByTestId("node");
    const node1Before = await runEffect(() => nodes.first().boundingBox());
    const node2Before = await runEffect(() => nodes.nth(1).boundingBox());
    if (!node1Before || !node2Before) {
      throw new Error("node bounds not available");
    }

    // Calculate relative distance between nodes
    const dxBefore = node2Before.x - node1Before.x;
    const dyBefore = node2Before.y - node1Before.y;

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const seHandle = await getResizeHandle(canvas, "se");
    await dragHandle(page, seHandle, 60, 50);

    // Get updated node positions
    const node1After = await runEffect(() => nodes.first().boundingBox());
    const node2After = await runEffect(() => nodes.nth(1).boundingBox());
    if (!node1After || !node2After) {
      throw new Error("node bounds not available after resize");
    }

    // Calculate relative distance after resize
    const dxAfter = node2After.x - node1After.x;
    const dyAfter = node2After.y - node1After.y;

    // Relative positions should be preserved (within tolerance for resize scaling)
    expect(Math.abs(dxAfter - dxBefore)).toBeLessThan(50);
    expect(Math.abs(dyAfter - dyBefore)).toBeLessThan(50);

    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  // MUL-008: Resize clamps to minimum size
  test("MUL-008: resize clamps to minimum size without errors @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 250),
      () => createTextNode(page, canvas, 600, 300),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const seHandle = await getResizeHandle(canvas, "se");

    // Try to shrink selection dramatically (beyond minimum)
    await dragHandle(page, seHandle, -300, -250, 10);

    // Selection should still exist with valid dimensions
    const afterBounds = await getSelectionBounds(canvas);
    expect(afterBounds.width).toBeGreaterThan(0);
    expect(afterBounds.height).toBeGreaterThan(0);
    expect(Number.isFinite(afterBounds.width)).toBe(true);
    expect(Number.isFinite(afterBounds.height)).toBe(true);

    // Nodes should still be selected
    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  // MUL-009: Resize expands selection bounds
  test("MUL-009: resize expands selection bounds correctly @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 450, 220),
      () => createTextNode(page, canvas, 550, 280),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const beforeBounds = await getSelectionBounds(canvas);
    const seHandle = await getResizeHandle(canvas, "se");

    // Expand selection significantly
    await dragHandle(page, seHandle, 100, 80);

    const afterBounds = await getSelectionBounds(canvas);

    // Selection should have expanded
    expect(afterBounds.width).toBeGreaterThan(beforeBounds.width + 50);
    expect(afterBounds.height).toBeGreaterThan(beforeBounds.height + 40);

    // Both nodes should still be selected
    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  // MUL-010: Resize with text nodes
  test("MUL-010: resize with text nodes works without errors @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");

    // Create text nodes
    await runEffectsSequential([
      () => createTextNode(page, canvas, 400, 200),
      () => createTextNode(page, canvas, 550, 280),
    ]);
    await expectNodeCount(page, 2);

    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    const beforeBounds = await getSelectionBounds(canvas);

    // Resize using NW handle
    const nwHandle = await getResizeHandle(canvas, "nw");
    await dragHandle(page, nwHandle, -40, -30);

    const afterBounds = await getSelectionBounds(canvas);

    // Selection should have expanded from NW
    expect(afterBounds.x).toBeLessThan(beforeBounds.x);
    expect(afterBounds.y).toBeLessThan(beforeBounds.y);
    expect(afterBounds.width).toBeGreaterThan(beforeBounds.width);
    expect(afterBounds.height).toBeGreaterThan(beforeBounds.height);

    // Verify no errors occurred
    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });
});
