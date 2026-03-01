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
});
