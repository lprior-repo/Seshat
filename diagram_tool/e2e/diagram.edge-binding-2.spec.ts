import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  edgeCount,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  nodeCenters,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForNoRebuildOverlay,
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

async function edgeClick(page: Page, x: number, y: number) {
  await runEffectsSequential([
    () => page.mouse.move(x, y),
    () => page.mouse.down(),
    () => page.mouse.up(),
  ]);
}

async function clickCanvasWhitespace(page: Page, canvasRoot: Locator) {
  const box = await runEffect(() => canvasRoot.boundingBox());
  if (!box) {
    throw new Error("canvas bounds unavailable");
  }
  await edgeClick(page, box.x + 28, box.y + 28);
}

function extrema(points: Array<{ x: number; y: number }>) {
  const left = points.reduce((best, p) => (p.x < best.x ? p : best));
  const right = points.reduce((best, p) => (p.x > best.x ? p : best));
  const top = points.reduce((best, p) => (p.y < best.y ? p : best));
  const bottom = points.reduce((best, p) => (p.y > best.y ? p : best));
  return { left, right, top, bottom };
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

test.describe("diagram edge binding 2", () => {
  // EDG-011: Rotate node keeps binding
  // NOTE: Rotation feature is not currently exposed in the UI.
  // This test is skipped until rotation controls are implemented.
  test.skip("EDG-011: rotate node keeps binding @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 250),
      () => createTextNode(page, canvas, 700, 350),
    ]);
    await expectNodeCount(page, 2);

    // Create edge between nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes to connect");
    }
    await edgeClick(page, centers[0].x, centers[0].y);
    await edgeClick(page, centers[1].x, centers[1].y);
    await expectEdgeCount(page, 1);

    // TODO: Add rotation test when rotation controls are implemented
    // For now, verify edge exists and no errors
    expect(await edgeCount(page)).toBe(1);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-012: Rotate selection with edges
  // NOTE: Rotation feature is not currently exposed in the UI.
  // This test is skipped until rotation controls are implemented.
  test.skip("EDG-012: rotate selection with edges @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 400, 200),
      () => createTextNode(page, canvas, 600, 200),
      () => createTextNode(page, canvas, 500, 350),
    ]);
    await expectNodeCount(page, 3);

    // Create edges between nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 3) {
      throw new Error("expected at least three nodes to connect");
    }
    // Create a triangle of edges
    await edgeClick(page, centers[0].x, centers[0].y);
    await edgeClick(page, centers[1].x, centers[1].y);
    await edgeClick(page, centers[2].x, centers[2].y);
    await edgeClick(page, centers[0].x, centers[0].y);
    await expectEdgeCount(page, 3);

    // Select all nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );
    await selectMultipleNodes(page, canvas, 3);

    // TODO: Add rotation test when rotation controls are implemented
    // For now, verify edges exist and no errors
    expect(await edgeCount(page)).toBe(3);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-013: Resize selection with edges
  test("EDG-013: resize selection with edges maintains bindings @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 450, 220),
      () => createTextNode(page, canvas, 600, 320),
    ]);
    await expectNodeCount(page, 2);

    // Create edge between nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await waitForNoRebuildOverlay(page);
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes to connect");
    }
    await edgeClick(page, centers[0].x, centers[0].y);
    await edgeClick(page, centers[1].x, centers[1].y);
    await expectEdgeCount(page, 1);

    // Switch to select mode and select both nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );
    await waitForNoRebuildOverlay(page);
    await selectMultipleNodes(page, canvas, 2);
    await expectSelectedCount(page, 2);

    // Resize the selection
    const seHandle = await getResizeHandle(canvas, "se");
    await dragHandle(page, seHandle, 80, 60);

    // Edge should still exist (binding maintained)
    await expectEdgeCount(page, 1);

    // Selection should still have 2 nodes
    await expectSelectedCount(page, 2);

    expect(pageErrors).toHaveLength(0);
  });

  // EDG-014: Multi-select includes edge but not nodes
  test("EDG-014: clicking edge selects edge only not nodes @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 400, 280),
      () => createTextNode(page, canvas, 700, 280),
    ]);
    await expectNodeCount(page, 2);

    // Create edge between nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await waitForNoRebuildOverlay(page);
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes to connect");
    }
    const { left, right } = extrema(centers);
    await edgeClick(page, left.x, left.y);
    await edgeClick(page, right.x, right.y);
    await expectEdgeCount(page, 1);

    // Switch to select mode
    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );
    await waitForNoRebuildOverlay(page);

    // Click on the middle of the edge (not near endpoints)
    const edgeMidX = (left.x + right.x) / 2;
    const edgeMidY = (left.y + right.y) / 2;

    // First clear any selection
    await clickCanvasWhitespace(page, canvas);
    expect(await selectedCount(page)).toBe(0);

    // Click on the edge
    await edgeClick(page, edgeMidX, edgeMidY);

    // Should have exactly 1 selected item (the edge, not nodes)
    expect(await selectedCount(page)).toBe(1);

    // Verify edge still exists
    expect(await edgeCount(page)).toBe(1);

    expect(pageErrors).toHaveLength(0);
  });

  // EDG-015: Edge endpoint follows node during drag
  test("EDG-015: edge endpoint follows node during drag @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 400, 280),
      () => createTextNode(page, canvas, 700, 280),
    ]);
    await expectNodeCount(page, 2);

    // Create edge between nodes
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await waitForNoRebuildOverlay(page);
    const centersBefore = await runEffect(() => nodeCenters(canvas));
    if (centersBefore.length < 2) {
      throw new Error("expected at least two nodes to connect");
    }
    await edgeClick(page, centersBefore[0].x, centersBefore[0].y);
    await edgeClick(page, centersBefore[1].x, centersBefore[1].y);
    await expectEdgeCount(page, 1);

    // Switch to select mode
    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );
    await waitForNoRebuildOverlay(page);

    // Get the first node's position
    const nodes = canvas.getByTestId("node");
    const node1Before = await runEffect(() => nodes.first().boundingBox());
    if (!node1Before) {
      throw new Error("node bounds not available");
    }

    // Drag the first node to a new position
    const dragOffsetX = 100;
    const dragOffsetY = 80;
    await runEffectsSequential([
      () => page.mouse.move(node1Before.x + 20, node1Before.y + 15),
      () => page.mouse.down(),
      () => page.mouse.move(
        node1Before.x + 20 + dragOffsetX,
        node1Before.y + 15 + dragOffsetY,
        { steps: 10 },
      ),
      () => page.mouse.up(),
    ]);

    // Wait for UI to settle
    await waitForNoRebuildOverlay(page);

    // Verify edge still exists (binding maintained)
    await expectEdgeCount(page, 1);

    // Verify the node moved
    const node1After = await runEffect(() => nodes.first().boundingBox());
    if (!node1After) {
      throw new Error("node bounds not available after drag");
    }

    // Node should have moved by approximately the drag offset
    expect(Math.abs(node1After.x - node1Before.x - dragOffsetX)).toBeLessThan(30);
    expect(Math.abs(node1After.y - node1Before.y - dragOffsetY)).toBeLessThan(30);

    // Click on canvas to clear selection
    await clickCanvasWhitespace(page, canvas);

    // Click on the edge at its new midpoint to verify it's still selectable
    // (which indicates the edge visual updated correctly)
    const centersAfter = await runEffect(() => nodeCenters(canvas));
    if (centersAfter.length < 2) {
      throw new Error("expected two nodes after drag");
    }
    const { left, right } = extrema(centersAfter);
    const edgeMidX = (left.x + right.x) / 2;
    const edgeMidY = (left.y + right.y) / 2;

    await edgeClick(page, edgeMidX, edgeMidY);
    expect(await selectedCount(page)).toBe(1);

    expect(pageErrors).toHaveLength(0);
  });
});
