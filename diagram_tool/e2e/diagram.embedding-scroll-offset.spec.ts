import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  minimapViewport,
  mountPageScrollHarness,
  mountScrollableHarness,
  runEffectsSequential,
  runEffect,
  scrollPageTo,
  scrollHarnessTo,
  trapPageErrors,
  waitForUiReady,
  zoomPercent,
} from "./helpers";

type BoundingBox = { x: number; y: number; width: number; height: number };

async function canvasBox(page: Page): Promise<BoundingBox> {
  const box = await runEffect(() => page.getByTestId("canvas-root").boundingBox());
  if (!box) {
    throw new Error("canvas bounds unavailable");
  }
  return box;
}

async function dragMinimapViewportBy(
  page: Page,
  dx: number,
  dy: number,
) {
  const viewportRect = minimapViewport(page);
  const box = await runEffect(() => viewportRect.boundingBox());
  if (!box) {
    throw new Error("minimap viewport rectangle unavailable");
  }
  await runEffectsSequential([
    () => page.mouse.move(box.x + box.width / 2, box.y + box.height / 2),
    () => page.mouse.down(),
    () => page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 6 }),
    () => page.mouse.up(),
  ]);
}

async function ensureMinimapVisible(page: Page) {
  const viewport = minimapViewport(page);
  if (await runEffect(() => viewport.count()) > 0) {
    return;
  }
  await runEffect(() => page.getByRole("button", { name: "Mini", exact: true }).click());
  await expect(viewport).toHaveCount(1);
}

async function expectLastNodeNear(
  page: Page,
  targetX: number,
  targetY: number,
  tolerance = 30,
) {
  const node = page.getByTestId("node").last();
  const placed = await runEffect(() => node.boundingBox());
  if (!placed) {
    throw new Error("placed node bounds unavailable");
  }

  expect(Math.abs(placed.x - targetX)).toBeLessThan(tolerance);
  expect(Math.abs(placed.y - targetY)).toBeLessThan(tolerance);
}

test.describe("diagram ancestor-scroll offset calibration", () => {
  test("scroll-parent offset preserves text placement hit testing @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountScrollableHarness(page),
      () => scrollHarnessTo(page, 210, 340),
      () => page.waitForTimeout(500), // Wait for canvas_origin to update in Rust
    ]);

    const canvas = page.getByTestId("canvas-root");
    const before = await canvasBox(page);
    const targetX = before.x + 380;
    const targetY = before.y + 240;

    await createTextNode(page, canvas, targetX - before.x, targetY - before.y);

    await expectNodeCount(page, 1);
    const node = page.getByTestId("node").first();
    const placed = await runEffect(() => node.boundingBox());
    if (!placed) {
      throw new Error("placed node bounds unavailable");
    }

    expect(Math.abs(placed.x - targetX)).toBeLessThan(28);
    expect(Math.abs(placed.y - targetY)).toBeLessThan(28);
    expect(pageErrors).toHaveLength(0);
  });

  test("wheel zoom anchor remains stable after ancestor scroll @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountScrollableHarness(page),
      () => scrollHarnessTo(page, 180, 300),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await createTextNode(page, canvas, 520, 260);

    const node = page.getByTestId("node").first();
    const before = await runEffect(() => node.boundingBox());
    if (!before) {
      throw new Error("node bounds unavailable before wheel zoom");
    }

    const anchorX = before.x + before.width / 2;
    const anchorY = before.y + before.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(anchorX, anchorY),
      () => page.mouse.wheel(0, -180),
    ]);

    expect(await zoomPercent(page)).not.toBe(100);
    const after = await runEffect(() => node.boundingBox());
    if (!after) {
      throw new Error("node bounds unavailable after wheel zoom");
    }

    const afterCenterX = after.x + after.width / 2;
    const afterCenterY = after.y + after.height / 2;
    expect(Math.abs(afterCenterX - anchorX)).toBeLessThan(26);
    expect(Math.abs(afterCenterY - anchorY)).toBeLessThan(26);
    expect(pageErrors).toHaveLength(0);
  });

  test("scrolling during active drag still finalizes pointer release coherently @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountScrollableHarness(page),
      () => scrollHarnessTo(page, 120, 200),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await createTextNode(page, canvas, 420, 220);

    const node = page.getByTestId("node").first();
    const before = await runEffect(() => node.boundingBox());
    const cbox = await canvasBox(page);
    if (!before) {
      throw new Error("node bounds unavailable before drag");
    }

    await runEffectsSequential([
      () => page.mouse.move(before.x + 12, before.y + 12),
      () => page.mouse.down(),
      () => scrollHarnessTo(page, 220, 360),
      () => page.mouse.move(cbox.x + cbox.width + 48, before.y + 24),
      () => page.mouse.up(),
    ]);

    const after = await runEffect(() => node.boundingBox());
    if (!after) {
      throw new Error("node bounds unavailable after drag release");
    }

    expect(after.x).toBeGreaterThan(before.x + 12);
    expect(pageErrors).toHaveLength(0);
  });

  test("repeated ancestor scroll recalibrates origin before additional placement @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountScrollableHarness(page),
      () => scrollHarnessTo(page, 170, 260),
      () => page.waitForTimeout(500),
    ]);

    const canvas = page.getByTestId("canvas-root");
    const firstBox = await canvasBox(page);
    const firstTargetX = firstBox.x + 360;
    const firstTargetY = firstBox.y + 230;

    await createTextNode(page, canvas, firstTargetX - firstBox.x, firstTargetY - firstBox.y);
    await expectNodeCount(page, 1);

    await runEffect(() => scrollHarnessTo(page, 420, 520));
    await page.waitForTimeout(100);

    const secondBox = await canvasBox(page);
    const secondTargetX = secondBox.x + 360;
    const secondTargetY = secondBox.y + 230;

    await createTextNode(page, canvas, secondTargetX - secondBox.x, secondTargetY - secondBox.y);
    await expectNodeCount(page, 2);

    const secondNode = page.getByTestId("node").nth(1);
    const placed = await runEffect(() => secondNode.boundingBox());
    if (!placed) {
      throw new Error("second node bounds unavailable");
    }

    expect(Math.abs(placed.x - secondTargetX)).toBeLessThan(28);
    expect(Math.abs(placed.y - secondTargetY)).toBeLessThan(28);
    expect(pageErrors).toHaveLength(0);
  });

  test("page scroll offset updates prevent stale placement drift @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountPageScrollHarness(page),
      () => scrollPageTo(page, 0, 320),
      () => page.waitForTimeout(500),
    ]);

    const canvas = page.getByTestId("canvas-root");
    const firstBox = await canvasBox(page);
    const firstTargetX = firstBox.x + 410;
    const firstTargetY = firstBox.y + 250;

    await createTextNode(page, canvas, firstTargetX - firstBox.x, firstTargetY - firstBox.y);
    await expectNodeCount(page, 1);

    await runEffect(() => scrollPageTo(page, 0, 760));
    await page.waitForTimeout(100);

    const secondBox = await canvasBox(page);
    const secondTargetX = secondBox.x + 410;
    const secondTargetY = secondBox.y + 250;

    await createTextNode(page, canvas, secondTargetX - secondBox.x, secondTargetY - secondBox.y);
    await expectNodeCount(page, 2);

    const secondNode = page.getByTestId("node").nth(1);
    const placed = await runEffect(() => secondNode.boundingBox());
    if (!placed) {
      throw new Error("page-scroll node bounds unavailable");
    }

    expect(Math.abs(placed.x - secondTargetX)).toBeLessThan(30);
    expect(Math.abs(placed.y - secondTargetY)).toBeLessThan(30);
    expect(pageErrors).toHaveLength(0);
  });

  test("offset updates immediately after page scroll before any canvas interaction @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountPageScrollHarness(page),
      () => scrollPageTo(page, 0, 220),
      () => scrollPageTo(page, 0, 860),
      () => page.waitForTimeout(500),
    ]);

    const box = await canvasBox(page);
    const targetX = box.x + 420;
    const targetY = box.y + 260;

    await createTextNode(page, page.getByTestId("canvas-root"), targetX - box.x, targetY - box.y);

    await expectNodeCount(page, 1);
    await expectLastNodeNear(page, targetX, targetY, 30);
    expect(pageErrors).toHaveLength(0);
  });

  test("offset updates after nested scroll container move @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountScrollableHarness(page),
      () => scrollHarnessTo(page, 140, 220),
      () => scrollHarnessTo(page, 520, 700),
      () => page.waitForTimeout(500),
    ]);

    const box = await canvasBox(page);
    const targetX = box.x + 360;
    const targetY = box.y + 220;

    await createTextNode(page, page.getByTestId("canvas-root"), targetX - box.x, targetY - box.y);

    await expectNodeCount(page, 1);
    await expectLastNodeNear(page, targetX, targetY, 30);
    expect(pageErrors).toHaveLength(0);
  });

  test("hit-testing still aligns after scroll plus zoom sequence @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountPageScrollHarness(page),
      () => scrollPageTo(page, 0, 320),
      () => page.waitForTimeout(500),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await createTextNode(page, canvas, 500, 280);
    await expectSelectedCount(page, 1);

    const clearBox = await canvasBox(page);
    await runEffectsSequential([
      () => page.mouse.click(clearBox.x + 44, clearBox.y + 44),
      () => expectSelectedCount(page, 0),
      () => scrollPageTo(page, 0, 760),
      () => page.waitForTimeout(500),
    ]);

    const node = page.getByTestId("node").first();
    const scrolled = await runEffect(() => node.boundingBox());
    if (!scrolled) {
      throw new Error("node bounds unavailable after scroll");
    }

    const anchorX = scrolled.x + scrolled.width / 2;
    const anchorY = scrolled.y + scrolled.height / 2;
    await runEffectsSequential([
      () => page.mouse.move(anchorX, anchorY),
      () => page.mouse.wheel(0, -220),
      () => expect.poll(() => zoomPercent(page)).not.toBe(100),
    ]);

    const zoomed = await runEffect(() => node.boundingBox());
    if (!zoomed) {
      throw new Error("node bounds unavailable after zoom");
    }

    await runEffect(() =>
      page.mouse.click(zoomed.x + zoomed.width / 2, zoomed.y + zoomed.height / 2),
    );

    await expectSelectedCount(page, 1);
    expect(pageErrors).toHaveLength(0);
  });

  test("minimap drag remains stable after ancestor scroll recalibration @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => freshStart(page),
      () => clearCanvasOverlays(page),
      () => mountScrollableHarness(page),
      () => scrollHarnessTo(page, 140, 260),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 320, 180),
      () => createTextNode(page, canvas, 760, 420),
      () => ensureMinimapVisible(page),
    ]);

    await runEffect(() => scrollHarnessTo(page, 300, 470));
    await runEffect(() => dragMinimapViewportBy(page, 40, 28));
    await runEffect(() => dragMinimapViewportBy(page, -36, -24));

    await expectNodeCount(page, 2);
    expect(await zoomPercent(page)).toBeGreaterThan(0);
    expect(pageErrors).toHaveLength(0);
  });
});
