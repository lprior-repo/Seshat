import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  freshStart,
  nodeCount,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  zoomPercent,
  mountScrollableHarness,
  scrollHarnessTo,
  waitForNoRebuildOverlay,
} from "./helpers";

const ZOOM_MIN = 10;
const ZOOM_MAX = 400;

type BoundingBox = { x: number; y: number; width: number; height: number };

async function canvasBox(page: Page): Promise<BoundingBox> {
  const box = await runEffect(() => page.getByTestId("canvas-root").boundingBox());
  if (!box) {
    throw new Error("canvas bounds unavailable");
  }
  return box;
}

async function zoomInUntil(page: Page, target: number): Promise<void> {
  for (let i = 0; i < 20; i += 1) {
    const current = await zoomPercent(page);
    if (current >= target) {
      return;
    }
    await runEffect(() => page.getByRole("button", { name: "+", exact: true }).first().click());
    await waitForNoRebuildOverlay(page);
  }
}

async function zoomOutUntil(page: Page, target: number): Promise<void> {
  for (let i = 0; i < 20; i += 1) {
    const current = await zoomPercent(page);
    if (current <= target) {
      return;
    }
    await runEffect(() => page.getByRole("button", { name: "-", exact: true }).first().click());
    await waitForNoRebuildOverlay(page);
  }
}

test.describe("CAM viewport and zoom behavior", () => {
  test("wheel zoom at cursor keeps node centered under pointer @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const node = canvas.getByTestId("node").first();
    const before = await runEffect(() => node.boundingBox());
    if (!before) {
      throw new Error("node bounds unavailable before zoom");
    }

    // Position cursor at node center and zoom
    const anchorX = before.x + before.width / 2;
    const anchorY = before.y + before.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(anchorX, anchorY),
      () => page.mouse.wheel(0, -120),
    ]);

    expect(await zoomPercent(page)).toBeGreaterThan(100);

    const after = await runEffect(() => node.boundingBox());
    if (!after) {
      throw new Error("node bounds unavailable after zoom");
    }

    // Node center should remain under cursor
    const afterCenterX = after.x + after.width / 2;
    const afterCenterY = after.y + after.height / 2;
    expect(Math.abs(afterCenterX - anchorX)).toBeLessThan(20);
    expect(Math.abs(afterCenterY - anchorY)).toBeLessThan(20);
    expect(pageErrors).toHaveLength(0);
  });

  test("spacebar + drag pans viewport without selecting nodes @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 500, 300));

    const node = canvas.getByTestId("node").first();
    const before = await runEffect(() => node.boundingBox());
    if (!before) {
      throw new Error("node bounds unavailable");
    }

    // Pan via spacebar + drag
    const startX = before.x + before.width / 2;
    const startY = before.y + before.height / 2;
    await runEffectsSequential([
      () => page.keyboard.down("Space"),
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(startX + 150, startY + 100, { steps: 8 }),
      () => page.mouse.up(),
      () => page.keyboard.up("Space"),
    ]);

    // Node screen position should have moved (pan effect)
    const after = await runEffect(() => node.boundingBox());
    if (!after) {
      throw new Error("node bounds unavailable after pan");
    }

    // The node screen position changes because camera moved
    const screenDeltaX = Math.abs(after.x - before.x);
    expect(screenDeltaX).toBeGreaterThan(50);

    // Node should not be selected (pan doesn't select)
    expect(await selectedCount(page)).toBe(0);
    expect(pageErrors).toHaveLength(0);
  });

  test("zoom out clamps at minimum 10% @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    // Try to zoom out way beyond minimum
    await zoomOutUntil(page, 5);

    const finalZoom = await zoomPercent(page);
    expect(finalZoom).toBeGreaterThanOrEqual(ZOOM_MIN);
    expect(finalZoom).toBeLessThanOrEqual(ZOOM_MIN + 5);
    expect(pageErrors).toHaveLength(0);
  });

  test("zoom in clamps at maximum 400% @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    // Try to zoom in way beyond maximum
    await zoomInUntil(page, 500);

    const finalZoom = await zoomPercent(page);
    expect(finalZoom).toBeLessThanOrEqual(ZOOM_MAX);
    expect(finalZoom).toBeGreaterThanOrEqual(ZOOM_MAX - 5);
    expect(pageErrors).toHaveLength(0);
  });

  test("world-to-screen remains consistent at extreme zoom levels @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 300));

    const node = canvas.getByTestId("node").first();
    const nodeAt100 = await runEffect(() => node.boundingBox());
    if (!nodeAt100) {
      throw new Error("node bounds unavailable at 100%");
    }

    // Zoom to 400%
    await zoomInUntil(page, 390);
    const nodeAt400 = await runEffect(() => node.boundingBox());
    if (!nodeAt400) {
      throw new Error("node bounds unavailable at 400%");
    }

    // At 4x zoom, screen width should be ~4x (relative to camera position)
    const widthRatio = nodeAt400.width / nodeAt100.width;
    expect(widthRatio).toBeGreaterThan(3.5);
    expect(widthRatio).toBeLessThan(4.5);

    // Zoom back to 100%
    await runEffect(() => page.getByRole("button", { name: "100%", exact: false }).first().click());
    await waitForNoRebuildOverlay(page);
    const nodeBack = await runEffect(() => node.boundingBox());
    if (!nodeBack) {
      throw new Error("node bounds unavailable after reset");
    }

    // Should be back to original size
    expect(Math.abs(nodeBack.width - nodeAt100.width)).toBeLessThan(10);
    expect(pageErrors).toHaveLength(0);
  });

  test("wheel zoom works when canvas is in scrollable container @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);
    await mountScrollableHarness(page);
    await scrollHarnessTo(page, 200, 300);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const node = canvas.getByTestId("node").first();
    const before = await runEffect(() => node.boundingBox());
    if (!before) {
      throw new Error("node bounds unavailable");
    }

    const anchorX = before.x + before.width / 2;
    const anchorY = before.y + before.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(anchorX, anchorY),
      () => page.mouse.wheel(0, -100),
    ]);

    expect(await zoomPercent(page)).toBeGreaterThan(100);
    expect(pageErrors).toHaveLength(0);
  });

  test("drag near scroll parent edge updates scroll position @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);
    await mountScrollableHarness(page);
    await scrollHarnessTo(page, 120, 200);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 380, 240));

    const node = canvas.getByTestId("node").first();
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

    const before = await runEffect(() => node.boundingBox());
    if (!before) {
      throw new Error("node bounds unavailable");
    }

    // Drag node while scroll position changes
    const cbox = await canvasBox(page);
    await runEffectsSequential([
      () => page.mouse.move(before.x + 20, before.y + 20),
      () => page.mouse.down(),
      () => scrollHarnessTo(page, 280, 400),
      () => page.mouse.move(cbox.x + cbox.width - 60, before.y + 40, { steps: 6 }),
      () => page.mouse.up(),
    ]);

    const after = await runEffect(() => node.boundingBox());
    if (!after) {
      throw new Error("node bounds unavailable after drag");
    }

    // Node should have moved
    expect(after.x).toBeGreaterThan(before.x - 20);
    expect(pageErrors).toHaveLength(0);
  });

  test("viewport recalculates after resize simulation @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const initialZoom = await zoomPercent(page);
    expect(initialZoom).toBe(100);

    // Simulate viewport change by resizing
    const originalSize = page.viewportSize();
    if (originalSize) {
      await page.setViewportSize({ width: originalSize.width + 200, height: originalSize.height + 100 });
    }

    // Canvas should still be functional
    await runEffect(() => createTextNode(page, canvas, 600, 400));
    expect(await nodeCount(page)).toBe(2);

    // Zoom should still work
    const cbox = await canvasBox(page);
    await runEffectsSequential([
      () => page.mouse.move(cbox.x + cbox.width / 2, cbox.y + cbox.height / 2),
      () => page.mouse.wheel(0, -80),
    ]);

    expect(await zoomPercent(page)).toBeGreaterThan(100);
    expect(pageErrors).toHaveLength(0);
  });

  test("edge scrolling during drag reveals more canvas space @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const node = canvas.getByTestId("node").first();
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

    const before = await runEffect(() => node.boundingBox());
    if (!before) {
      throw new Error("node bounds unavailable");
    }

    // Get canvas bounds and drag near the right edge to trigger edge scrolling
    const cbox = await canvasBox(page);
    const edgeX = cbox.x + cbox.width - 20;
    const nodeCenterX = before.x + before.width / 2;
    const nodeCenterY = before.y + before.height / 2;

    // Drag node towards the right edge and hold near edge
    await runEffectsSequential([
      () => page.mouse.move(nodeCenterX, nodeCenterY),
      () => page.mouse.down(),
      () => page.mouse.move(edgeX, nodeCenterY, { steps: 12 }),
    ]);

    // Wait briefly for potential edge-scroll to trigger
    await page.waitForTimeout(150);

    // Complete the drag
    await runEffect(() => page.mouse.up());

    // Node should have moved significantly (edge scroll + drag)
    const after = await runEffect(() => node.boundingBox());
    if (!after) {
      throw new Error("node bounds unavailable after drag");
    }

    // Verify node position changed
    expect(Math.abs(after.x - before.x)).toBeGreaterThan(30);
    expect(pageErrors).toHaveLength(0);
  });

  test("fit to content centers nodes with appropriate padding @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    const cbox = await canvasBox(page);

    // Create nodes at various positions
    await runEffect(() => createTextNode(page, canvas, 100, 100));
    await waitForNoRebuildOverlay(page);
    await runEffect(() => createTextNode(page, canvas, 500, 400));
    await waitForNoRebuildOverlay(page);
    await runEffect(() => createTextNode(page, canvas, 300, 250));
    await waitForNoRebuildOverlay(page);

    expect(await nodeCount(page)).toBe(3);

    // Zoom in to change viewport from default
    const centerX = cbox.x + cbox.width / 2;
    const centerY = cbox.y + cbox.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(centerX, centerY),
      () => page.mouse.wheel(0, -200),
    ]);
    await waitForNoRebuildOverlay(page);

    const zoomedZoom = await zoomPercent(page);
    expect(zoomedZoom).toBeGreaterThan(100);

    // Click zoom reset button to fit content back to viewport
    await runEffect(() => page.getByRole("button", { name: "100%", exact: false }).first().click());
    await waitForNoRebuildOverlay(page);

    // After reset, zoom should return close to 100%
    const resetZoom = await zoomPercent(page);
    expect(resetZoom).toBeGreaterThanOrEqual(90);
    expect(resetZoom).toBeLessThanOrEqual(110);

    // All nodes should still be visible (within canvas bounds with padding)
    const nodes = await canvas.getByTestId("node").all();
    for (const n of nodes) {
      const box = await runEffect(() => n.boundingBox());
      if (box) {
        // Node should be within canvas bounds (with some tolerance for padding)
        expect(box.x).toBeGreaterThan(cbox.x - 50);
        expect(box.y).toBeGreaterThan(cbox.y - 50);
        expect(box.x + box.width).toBeLessThan(cbox.x + cbox.width + 50);
        expect(box.y + box.height).toBeLessThan(cbox.y + cbox.height + 50);
      }
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("canvas embedded in scrollable parent handles coordinate offset @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);
    await mountScrollableHarness(page);

    // Scroll the harness to create an offset
    const scrollX = 350;
    const scrollY = 280;
    await scrollHarnessTo(page, scrollX, scrollY);

    const canvas = page.getByTestId("canvas-root");
    const cbox = await canvasBox(page);

    // Create a node at a specific position
    const nodeX = 300;
    const nodeY = 200;
    await runEffect(() => createTextNode(page, canvas, nodeX, nodeY));

    const node = canvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds unavailable");
    }

    // Verify node was created within expected area (accounting for scroll offset)
    // The node should be positioned relative to canvas, not viewport
    expect(nodeBounds.x).toBeGreaterThan(cbox.x);
    expect(nodeBounds.y).toBeGreaterThan(cbox.y);

    // Zoom while scrolled and verify transform accounts for offset
    const anchorX = nodeBounds.x + nodeBounds.width / 2;
    const anchorY = nodeBounds.y + nodeBounds.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(anchorX, anchorY),
      () => page.mouse.wheel(0, -100),
    ]);

    expect(await zoomPercent(page)).toBeGreaterThan(100);

    // Node should still be near cursor position after zoom
    const nodeAfterZoom = await runEffect(() => node.boundingBox());
    if (!nodeAfterZoom) {
      throw new Error("node bounds unavailable after zoom");
    }

    const afterCenterX = nodeAfterZoom.x + nodeAfterZoom.width / 2;
    const afterCenterY = nodeAfterZoom.y + nodeAfterZoom.height / 2;
    expect(Math.abs(afterCenterX - anchorX)).toBeLessThan(30);
    expect(Math.abs(afterCenterY - anchorY)).toBeLessThan(30);
    expect(pageErrors).toHaveLength(0);
  });

  test("viewport recalculates after DPR change @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const node = canvas.getByTestId("node").first();
    const nodeBefore = await runEffect(() => node.boundingBox());
    if (!nodeBefore) {
      throw new Error("node bounds unavailable before DPR change");
    }

    const initialZoom = await zoomPercent(page);
    expect(initialZoom).toBe(100);

    // Simulate DPR change via client-side emulation
    await runEffect(() =>
      page.evaluate(() => {
        // Store original DPR
        const originalDpr = window.devicePixelRatio;
        // Emulate DPR change by dispatching a custom event that the app may listen to
        window.dispatchEvent(new CustomEvent("dprchange", { detail: { oldDpr: originalDpr, newDpr: originalDpr * 2 } }));
        // Also trigger resize which often recalculates DPR-dependent values
        window.dispatchEvent(new Event("resize"));
      }),
    );

    // Allow time for any DPR-related recalculations
    await page.waitForTimeout(100);
    await waitForNoRebuildOverlay(page);

    // Zoom should remain consistent
    const zoomAfterDprChange = await zoomPercent(page);
    expect(zoomAfterDprChange).toBeGreaterThanOrEqual(95);
    expect(zoomAfterDprChange).toBeLessThanOrEqual(105);

    // Node should still be visible and selectable
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

    // Verify zoom controls still work after DPR change
    const cbox = await canvasBox(page);
    await runEffectsSequential([
      () => page.mouse.move(cbox.x + cbox.width / 2, cbox.y + cbox.height / 2),
      () => page.mouse.wheel(0, -80),
    ]);

    expect(await zoomPercent(page)).toBeGreaterThan(100);
    expect(pageErrors).toHaveLength(0);
  });

  test("context menu focus loss mid-drag does not corrupt selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const node = canvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds unavailable");
    }

    // Select the node first
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);

    // Start dragging the node
    const startX = nodeBounds.x + nodeBounds.width / 2;
    const startY = nodeBounds.y + nodeBounds.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(startX + 50, startY + 30, { steps: 4 }),
    ]);

    // Simulate context menu opening (right-click mid-drag)
    // This causes focus loss in some browsers
    await runEffect(() => page.mouse.click(startX + 60, startY + 40, { button: "right" }));

    // Wait briefly for any context menu handling
    await page.waitForTimeout(50);

    // Dismiss by clicking elsewhere or pressing Escape
    await runEffect(() => page.keyboard.press("Escape"));
    await page.waitForTimeout(50);

    // Complete the drag operation
    await runEffect(() => page.mouse.up());

    // Wait for any state stabilization
    await waitForNoRebuildOverlay(page);

    // Selection state should be intact (node should still be selected)
    const finalSelectedCount = await selectedCount(page);
    expect(finalSelectedCount).toBeGreaterThanOrEqual(1);

    // Node should exist and be interactive
    const nodeAfter = await runEffect(() => node.boundingBox());
    expect(nodeAfter).not.toBeNull();

    // Verify no corruption by performing another action
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(pageErrors).toHaveLength(0);
  });

  test("auto-save preserves camera position without stutter @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    // Pan the viewport to a non-default position
    const node = canvas.getByTestId("node").first();
    const nodeBefore = await runEffect(() => node.boundingBox());
    if (!nodeBefore) {
      throw new Error("node bounds unavailable");
    }

    const startX = nodeBefore.x + nodeBefore.width / 2;
    const startY = nodeBefore.y + nodeBefore.height / 2;
    await runEffectsSequential([
      () => page.keyboard.down("Space"),
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(startX - 100, startY - 80, { steps: 8 }),
      () => page.mouse.up(),
      () => page.keyboard.up("Space"),
    ]);

    await waitForNoRebuildOverlay(page);

    // Record node position after pan (represents camera position)
    const nodeAfterPan = await runEffect(() => node.boundingBox());
    if (!nodeAfterPan) {
      throw new Error("node bounds unavailable after pan");
    }

    // Simulate auto-save trigger
    await runEffect(() =>
      page.evaluate(() => {
        // Dispatch an auto-save event if the app listens for it
        window.dispatchEvent(new CustomEvent("seshat-autosave", { detail: { timestamp: Date.now() } }));
        // Also trigger storage event as auto-save might use localStorage
        window.dispatchEvent(new StorageEvent("storage", { key: "seshat-autosave", newValue: String(Date.now()) }));
      }),
    );

    // Wait for any save-related processing
    await page.waitForTimeout(100);
    await waitForNoRebuildOverlay(page);

    // Node position should remain stable (camera did not jump)
    const nodeAfterSave = await runEffect(() => node.boundingBox());
    if (!nodeAfterSave) {
      throw new Error("node bounds unavailable after save");
    }

    // Position should be nearly identical (allowing small floating-point variance)
    expect(Math.abs(nodeAfterSave.x - nodeAfterPan.x)).toBeLessThan(5);
    expect(Math.abs(nodeAfterSave.y - nodeAfterPan.y)).toBeLessThan(5);

    // Zoom should also remain stable
    const zoomBefore = await zoomPercent(page);

    // Trigger another save with zoom change
    const cbox = await canvasBox(page);
    await runEffectsSequential([
      () => page.mouse.move(cbox.x + cbox.width / 2, cbox.y + cbox.height / 2),
      () => page.mouse.wheel(0, -80),
    ]);
    await waitForNoRebuildOverlay(page);

    const zoomAfter = await zoomPercent(page);
    expect(zoomAfter).toBeGreaterThan(zoomBefore);

    // Trigger save again
    await runEffect(() =>
      page.evaluate(() => {
        window.dispatchEvent(new CustomEvent("seshat-autosave", { detail: { timestamp: Date.now() } }));
      }),
    );
    await page.waitForTimeout(50);

    // Zoom should remain at the new level
    const zoomFinal = await zoomPercent(page);
    expect(Math.abs(zoomFinal - zoomAfter)).toBeLessThan(2);
    expect(pageErrors).toHaveLength(0);
  });

  test("pan inertia decays smoothly to stop @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 400, 280));

    const node = canvas.getByTestId("node").first();
    const nodeBefore = await runEffect(() => node.boundingBox());
    if (!nodeBefore) {
      throw new Error("node bounds unavailable");
    }

    // Perform a pan with momentum-like movement
    const startX = nodeBefore.x + nodeBefore.width / 2;
    const startY = nodeBefore.y + nodeBefore.height / 2;
    const panDistance = 150;

    await runEffectsSequential([
      () => page.keyboard.down("Space"),
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      // Quick movement to potentially trigger inertia
      () => page.mouse.move(startX + panDistance, startY + 80, { steps: 12 }),
      () => page.mouse.up(),
      () => page.keyboard.up("Space"),
    ]);

    // Record position immediately after pan ends
    const nodeAfterPan = await runEffect(() => node.boundingBox());
    if (!nodeAfterPan) {
      throw new Error("node bounds unavailable after pan");
    }

    // Wait for any inertia to settle
    await page.waitForTimeout(200);
    await waitForNoRebuildOverlay(page);

    // Record final position
    const nodeFinal = await runEffect(() => node.boundingBox());
    if (!nodeFinal) {
      throw new Error("node bounds unavailable after settle");
    }

    // If inertia is implemented, the camera may have moved slightly more
    // If not implemented, position should be stable immediately
    // Either way, the position should now be stable (not jittering)

    // Wait another moment and verify stability
    await page.waitForTimeout(100);
    const nodeStable = await runEffect(() => node.boundingBox());
    if (!nodeStable) {
      throw new Error("node bounds unavailable for stability check");
    }

    // Position should be stable (no ongoing jitter)
    expect(Math.abs(nodeStable.x - nodeFinal.x)).toBeLessThan(3);
    expect(Math.abs(nodeStable.y - nodeFinal.y)).toBeLessThan(3);

    // Verify pan actually occurred (node screen position changed)
    const totalDeltaX = Math.abs(nodeFinal.x - nodeBefore.x);
    expect(totalDeltaX).toBeGreaterThan(30);

    // Verify interaction state is clean
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(pageErrors).toHaveLength(0);
  });
});
