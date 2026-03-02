import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  createTextNode,
  edgeCount,
  expectEdgeCount,
  expectNodeCount,
  freshStart,
  nodeCenters,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForNoRebuildOverlay,
  waitForUiReady,
  zoomPercent,
} from "./helpers";

test.describe("diagram edges and routing", () => {
  async function edgeClick(page: Page, x: number, y: number) {
    await runEffectsSequential([
      () => page.mouse.move(x, y),
      () => page.mouse.down(),
      () => page.mouse.up(),
    ]);
  }

  async function resetZoom(page: Page) {
    await runEffect(() => page.getByTestId("zoom-reset").click());
    await expect.poll(() => zoomPercent(page)).toBe(100);
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

  async function zoomInToAtLeast(page: Page, targetPercent: number) {
    for (let i = 0; i < 16; i += 1) {
      const current = await zoomPercent(page);
      if (current >= targetPercent) {
        return;
      }
      await runEffect(() => page.getByTestId("zoom-in").click());
    }
    throw new Error(`failed to reach zoom >= ${targetPercent}%`);
  }

  test("connects nodes with edge tool @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");

    await runEffectsSequential([
      () => createTextNode(page, canvas, 560, 210),
      () => waitForUiReady(page),
      () => createTextNode(page, canvas, 820, 330),
    ]);
    await expectNodeCount(page, 2);

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
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(pageErrors).toHaveLength(0);
  });

  test("rejects cycle-forming edge in dag flow @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 210),
      () => createTextNode(page, canvas, 760, 230),
      () => createTextNode(page, canvas, 980, 260),
    ]);
    await expectNodeCount(page, 3);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 3) {
      throw new Error("expected three nodes for cycle rejection test");
    }

    await edgeClick(page, centers[0].x, centers[0].y);
    await edgeClick(page, centers[1].x, centers[1].y);
    await edgeClick(page, centers[2].x, centers[2].y);
    await expectEdgeCount(page, 2);

    await edgeClick(page, centers[2].x, centers[2].y);
    await edgeClick(page, centers[0].x, centers[0].y);
    await expectEdgeCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  test("edge overlap hit-selection stays deterministic across undo/redo cycle @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 360, 320),
      () => waitForUiReady(page),
      () => createTextNode(page, canvas, 780, 320),
      () => waitForUiReady(page),
      () => createTextNode(page, canvas, 620, 160),
      () => waitForUiReady(page),
      () => createTextNode(page, canvas, 620, 480),
    ]);
    await expectNodeCount(page, 4);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 4) {
      throw new Error("expected four nodes for overlap hit-selection test");
    }
    const { left, right, top, bottom } = extrema(centers);
    await edgeClick(page, left.x, left.y);
    await edgeClick(page, right.x, right.y);
    await edgeClick(page, top.x, top.y);
    await edgeClick(page, bottom.x, bottom.y);
    await expectEdgeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const centerX = (left.x + right.x) / 2;
    const centerY = (top.y + bottom.y) / 2;
    const horizontalProbeX = (left.x + centerX) / 2;
    const horizontalProbeY = centerY;
    const verticalProbeX = centerX;
    const verticalProbeY = (top.y + centerY) / 2;

    const detectRemainingOrientation = async (): Promise<"horizontal" | "vertical"> => {
      await edgeClick(page, centerX, centerY);
      expect(await selectedCount(page)).toBe(1);

      await runEffect(() => page.keyboard.press("Delete"));
      expect(await edgeCount(page)).toBe(1);

      await clickCanvasWhitespace(page, canvas);
      await edgeClick(page, horizontalProbeX, horizontalProbeY);
      const horizontalHit = (await selectedCount(page)) === 1;

      await clickCanvasWhitespace(page, canvas);
      await edgeClick(page, verticalProbeX, verticalProbeY);
      const verticalHit = (await selectedCount(page)) === 1;

      expect(horizontalHit).not.toBe(verticalHit);
      return horizontalHit ? "horizontal" : "vertical";
    };

    const firstRemaining = await detectRemainingOrientation();
    await runEffect(() =>
      page.getByRole("button", { name: "Undo", exact: true }).click(),
    );
    expect(await edgeCount(page)).toBe(2);

    const secondRemaining = await detectRemainingOrientation();
    expect(secondRemaining).toBe(firstRemaining);
    expect(pageErrors).toHaveLength(0);
  });

  test("overlapping edge hit-selection is deterministic across repeated clicks @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 360, 320),
      () => createTextNode(page, canvas, 780, 320),
      () => createTextNode(page, canvas, 620, 160),
      () => createTextNode(page, canvas, 620, 480),
    ]);
    await expectNodeCount(page, 4);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 4) {
      throw new Error("expected four nodes for repeated overlap test");
    }
    const { left, right, top, bottom } = extrema(centers);
    await edgeClick(page, left.x, left.y);
    await edgeClick(page, right.x, right.y);
    await edgeClick(page, top.x, top.y);
    await edgeClick(page, bottom.x, bottom.y);
    await expectEdgeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const centerX = (left.x + right.x) / 2;
    const centerY = (top.y + bottom.y) / 2;
    const horizontalProbeX = (left.x + centerX) / 2;
    const horizontalProbeY = centerY;
    const verticalProbeX = centerX;
    const verticalProbeY = (top.y + centerY) / 2;

    const detectRemovedOrientation = async (): Promise<"horizontal" | "vertical"> => {
      await edgeClick(page, centerX, centerY);
      expect(await selectedCount(page)).toBe(1);

      await runEffect(() => page.keyboard.press("Delete"));
      expect(await edgeCount(page)).toBe(1);

      await clickCanvasWhitespace(page, canvas);
      await edgeClick(page, horizontalProbeX, horizontalProbeY);
      const horizontalRemains = (await selectedCount(page)) === 1;

      await clickCanvasWhitespace(page, canvas);
      await edgeClick(page, verticalProbeX, verticalProbeY);
      const verticalRemains = (await selectedCount(page)) === 1;

      expect(horizontalRemains).not.toBe(verticalRemains);
      return horizontalRemains ? "vertical" : "horizontal";
    };

    const firstRemoved = await detectRemovedOrientation();
    for (let i = 0; i < 2; i += 1) {
      await runEffect(() =>
        page.getByRole("button", { name: "Undo", exact: true }).click(),
      );
      expect(await edgeCount(page)).toBe(2);
      expect(await detectRemovedOrientation()).toBe(firstRemoved);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("thin vertical edge remains selectable across zoom levels @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 680, 160),
      () => createTextNode(page, canvas, 680, 520),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes for thin-edge zoom test");
    }
    const byY = [...centers].sort((a, b) => a.y - b.y);
    await edgeClick(page, byY[0].x, byY[0].y);
    await edgeClick(page, byY[1].x, byY[1].y);
    await expectEdgeCount(page, 1);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const probeX = byY[0].x + 3;
    const probeY = (byY[0].y + byY[1].y) / 2;

    await resetZoom(page);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    await resetZoom(page);
    await runEffect(() => page.getByTestId("zoom-out").click());
    await clickCanvasWhitespace(page, canvas);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    await resetZoom(page);
    await zoomInToAtLeast(page, 200);
    await clickCanvasWhitespace(page, canvas);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    await resetZoom(page);
    await zoomInToAtLeast(page, 300);
    await clickCanvasWhitespace(page, canvas);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    expect(pageErrors).toHaveLength(0);
  });

  test("endpoint-near clicks keep selecting the same edge endpoint @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 260),
      () => createTextNode(page, canvas, 860, 260),
      () => createTextNode(page, canvas, 700, 460),
    ]);
    await expectNodeCount(page, 3);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 3) {
      throw new Error("expected three nodes for endpoint-near selection test");
    }
    const { left, right, bottom } = extrema(centers);
    await edgeClick(page, left.x, left.y);
    await edgeClick(page, right.x, right.y);
    await edgeClick(page, bottom.x, bottom.y);
    await edgeClick(page, right.x, right.y);
    await expectEdgeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const nearEndpointX = right.x - 8;
    const nearEndpointY = right.y + 7;
    const horizontalProbeX = (left.x + right.x) / 2;
    const horizontalProbeY = (left.y + right.y) / 2;
    const diagonalProbeX = (bottom.x + right.x) / 2;
    const diagonalProbeY = (bottom.y + right.y) / 2;

    const detectRemoved = async (): Promise<"horizontal" | "diagonal"> => {
      await edgeClick(page, nearEndpointX, nearEndpointY);
      expect(await selectedCount(page)).toBe(1);

      await runEffect(() => page.keyboard.press("Delete"));
      expect(await edgeCount(page)).toBe(1);

      await clickCanvasWhitespace(page, canvas);
      await edgeClick(page, horizontalProbeX, horizontalProbeY);
      const horizontalRemains = (await selectedCount(page)) === 1;

      await clickCanvasWhitespace(page, canvas);
      await edgeClick(page, diagonalProbeX, diagonalProbeY);
      const diagonalRemains = (await selectedCount(page)) === 1;

      expect(horizontalRemains).not.toBe(diagonalRemains);
      return horizontalRemains ? "diagonal" : "horizontal";
    };

    const firstRemoved = await detectRemoved();
    for (let i = 0; i < 2; i += 1) {
      await runEffect(() =>
        page.getByRole("button", { name: "Undo", exact: true }).click(),
      );
      expect(await edgeCount(page)).toBe(2);
      expect(await detectRemoved()).toBe(firstRemoved);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("selects thin edge reliably near target-side endpoint @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 260),
      () => createTextNode(page, canvas, 860, 260),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected two nodes for thin-edge endpoint test");
    }
    const { left, right } = extrema(centers);
    await edgeClick(page, left.x, left.y);
    await edgeClick(page, right.x, right.y);
    await expectEdgeCount(page, 1);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const nearTargetX = right.x - 8;
    const nearTargetY = right.y + 7;
    await edgeClick(page, nearTargetX, nearTargetY);
    expect(await selectedCount(page)).toBe(1);

    await clickCanvasWhitespace(page, canvas);
    await edgeClick(page, nearTargetX, nearTargetY);
    expect(await selectedCount(page)).toBe(1);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-011: Edge between nodes in same container
  test("edge between nodes in same container @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);

    // Create 2 text nodes
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 520, 240),
      () => createTextNode(page, diagramCanvas, 640, 300),
    ]);
    await expectNodeCount(page, 2);

    // Create a subgraph container around the nodes
    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounding box not available");
    }
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 480, canvasBox.y + 200),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 720, canvasBox.y + 360, { steps: 8 }),
      () => page.mouse.up(),
    ]);
    // Switch back to select tool
    await runEffect(() => page.getByRole("button", { name: "Select", exact: true }).click());
    await waitForNoRebuildOverlay(page);
    await waitForUiReady(page);
    await expectNodeCount(page, 3); // 2 text nodes + 1 subgraph container

    // Create edge between the two nodes inside the container
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(diagramCanvas));
    // Filter to only text nodes (skip subgraph container which is larger)
    const textNodeCenters = centers.slice(0, 2);
    if (textNodeCenters.length < 2) {
      throw new Error("expected at least two text nodes inside container");
    }

    await edgeClick(page, textNodeCenters[0].x, textNodeCenters[0].y);
    await edgeClick(page, textNodeCenters[1].x, textNodeCenters[1].y);

    await expectEdgeCount(page, 1);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-012: Edge crossing container boundary
  test("edge crossing container boundary @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);

    // Create a text node that will be inside the container
    await runEffect(() => createTextNode(page, diagramCanvas, 520, 240));

    // Create a text node that will stay outside the container
    await runEffect(() => createTextNode(page, diagramCanvas, 820, 240));
    await expectNodeCount(page, 2);

    // Create a subgraph container around only the first node
    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounding box not available");
    }
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 480, canvasBox.y + 200),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 640, canvasBox.y + 360, { steps: 8 }),
      () => page.mouse.up(),
    ]);
    // Switch back to select tool
    await runEffect(() => page.getByRole("button", { name: "Select", exact: true }).click());
    await waitForNoRebuildOverlay(page);
    await waitForUiReady(page);
    await expectNodeCount(page, 3); // 2 text nodes + 1 subgraph container

    // Create edge from inside node to outside node
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(diagramCanvas));
    // Filter to only text nodes (skip subgraph container)
    const textNodeCenters = centers.slice(0, 2);
    if (textNodeCenters.length < 2) {
      throw new Error("expected at least two text nodes");
    }

    // Create edge crossing container boundary
    await edgeClick(page, textNodeCenters[0].x, textNodeCenters[0].y);
    await edgeClick(page, textNodeCenters[1].x, textNodeCenters[1].y);

    await expectEdgeCount(page, 1);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-013: Reparent node with edges
  test("reparent node with connected edge produces valid state @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);

    // Create 2 text nodes
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 520, 240),
      () => createTextNode(page, diagramCanvas, 640, 300),
    ]);
    await expectNodeCount(page, 2);

    // Create a subgraph container around the nodes
    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounding box not available");
    }
    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    await runEffectsSequential([
      () => page.mouse.move(canvasBox.x + 480, canvasBox.y + 200),
      () => page.mouse.down(),
      () => page.mouse.move(canvasBox.x + 720, canvasBox.y + 360, { steps: 8 }),
      () => page.mouse.up(),
    ]);
    // Switch back to select tool
    await runEffect(() => page.getByRole("button", { name: "Select", exact: true }).click());
    await waitForNoRebuildOverlay(page);
    await waitForUiReady(page);
    await expectNodeCount(page, 3); // 2 text nodes + 1 subgraph container

    // Create edge between the two nodes inside the container
    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(diagramCanvas));
    const textNodeCenters = centers.slice(0, 2);
    if (textNodeCenters.length < 2) {
      throw new Error("expected at least two text nodes inside container");
    }

    await edgeClick(page, textNodeCenters[0].x, textNodeCenters[0].y);
    await edgeClick(page, textNodeCenters[1].x, textNodeCenters[1].y);
    await expectEdgeCount(page, 1);

    // Switch to select tool
    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    // Drag the first node outside the container (to the left)
    const dragFromX = textNodeCenters[0].x;
    const dragFromY = textNodeCenters[0].y;
    const dragToX = canvasBox.x + 200; // Well outside to the left
    const dragToY = dragFromY;

    await runEffectsSequential([
      () => page.mouse.move(dragFromX, dragFromY),
      () => page.mouse.down(),
      () => page.mouse.move(dragToX, dragToY, { steps: 8 }),
      () => page.mouse.up(),
    ]);
    await waitForNoRebuildOverlay(page);
    await waitForUiReady(page);

    // Node count should still be 3
    await expectNodeCount(page, 3);

    // Edge should still exist (not orphaned)
    await expectEdgeCount(page, 1);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-014: Edge routing stable on overlapping nodes (horizontal)
  test("horizontal edge overlap hit-selection is deterministic across repeated clicks @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvasEl = page.getByTestId("canvas-root");
    // Create 4 nodes for two horizontal edges that will overlap
    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 260, 220),
      () => createTextNode(page, canvasEl, 880, 220),
      () => createTextNode(page, canvasEl, 260, 380),
      () => createTextNode(page, canvasEl, 880, 380),
    ]);
    await expectNodeCount(page, 4);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvasEl));
    if (centers.length < 4) {
      throw new Error("expected four nodes for horizontal overlap test");
    }

    // Sort by y to get top row and bottom row
    const byY = [...centers].sort((a, b) => a.y - b.y);
    // Top row: byY[0], byY[1]; Bottom row: byY[2], byY[3]
    // Sort each row by x
    const topRow = [byY[0], byY[1]].sort((a, b) => a.x - b.x);
    const bottomRow = [byY[2], byY[3]].sort((a, b) => a.x - b.x);

    // Create top horizontal edge
    await edgeClick(page, topRow[0].x, topRow[0].y);
    await edgeClick(page, topRow[1].x, topRow[1].y);

    // Create bottom horizontal edge
    await edgeClick(page, bottomRow[0].x, bottomRow[0].y);
    await edgeClick(page, bottomRow[1].x, bottomRow[1].y);

    await expectEdgeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    // Probe at the center of the canvas where edges should overlap
    const probeX = (topRow[0].x + topRow[1].x) / 2;
    const probeY = (topRow[0].y + bottomRow[0].y) / 2;

    // Click at overlap point multiple times and verify same edge is selected
    const selectedEdgeIds: number[] = [];
    for (let i = 0; i < 3; i += 1) {
      await clickCanvasWhitespace(page, canvasEl);
      await edgeClick(page, probeX, probeY);
      expect(await selectedCount(page)).toBe(1);

      // Delete the selected edge to identify which one it was
      await runEffect(() => page.keyboard.press("Delete"));
      const remainingEdges = await edgeCount(page);

      // If we deleted the top edge, top edge is the one selected
      // If we deleted the bottom edge, bottom edge is the one selected
      selectedEdgeIds.push(remainingEdges);

      // Undo to restore the edge
      await runEffect(() =>
        page.getByRole("button", { name: "Undo", exact: true }).click(),
      );
      await expectEdgeCount(page, 2);
    }

    // All selections should have identified the same edge
    expect(selectedEdgeIds[0]).toBe(selectedEdgeIds[1]);
    expect(selectedEdgeIds[1]).toBe(selectedEdgeIds[2]);
    expect(pageErrors).toHaveLength(0);
  });

  // EDG-015: Edge routing stable on overlapping nodes (vertical)
  test("vertical edge overlap hit-selection is deterministic across repeated clicks @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvasEl = page.getByTestId("canvas-root");
    // Create 4 nodes for two vertical edges that will overlap
    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 420, 120),
      () => createTextNode(page, canvasEl, 420, 520),
      () => createTextNode(page, canvasEl, 720, 120),
      () => createTextNode(page, canvasEl, 720, 520),
    ]);
    await expectNodeCount(page, 4);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvasEl));
    if (centers.length < 4) {
      throw new Error("expected four nodes for vertical overlap test");
    }

    // Sort by x to get left column and right column
    const byX = [...centers].sort((a, b) => a.x - b.x);
    // Left column: byX[0], byX[1]; Right column: byX[2], byX[3]
    // Sort each column by y
    const leftCol = [byX[0], byX[1]].sort((a, b) => a.y - b.y);
    const rightCol = [byX[2], byX[3]].sort((a, b) => a.y - b.y);

    // Create left vertical edge
    await edgeClick(page, leftCol[0].x, leftCol[0].y);
    await edgeClick(page, leftCol[1].x, leftCol[1].y);

    // Create right vertical edge
    await edgeClick(page, rightCol[0].x, rightCol[0].y);
    await edgeClick(page, rightCol[1].x, rightCol[1].y);

    await expectEdgeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    // Probe at the center of the canvas where edges should overlap
    const probeX = (leftCol[0].x + rightCol[0].x) / 2;
    const probeY = (leftCol[0].y + leftCol[1].y) / 2;

    // Click at overlap point multiple times and verify same edge is selected
    const selectedEdgeIds: number[] = [];
    for (let i = 0; i < 3; i += 1) {
      await clickCanvasWhitespace(page, canvasEl);
      await edgeClick(page, probeX, probeY);
      expect(await selectedCount(page)).toBe(1);

      // Delete the selected edge to identify which one it was
      await runEffect(() => page.keyboard.press("Delete"));
      const remainingEdges = await edgeCount(page);

      selectedEdgeIds.push(remainingEdges);

      // Undo to restore the edge
      await runEffect(() =>
        page.getByRole("button", { name: "Undo", exact: true }).click(),
      );
      await expectEdgeCount(page, 2);
    }

    // All selections should have identified the same edge
    expect(selectedEdgeIds[0]).toBe(selectedEdgeIds[1]);
    expect(selectedEdgeIds[1]).toBe(selectedEdgeIds[2]);
    expect(pageErrors).toHaveLength(0);
  });
});
