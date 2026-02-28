import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  nodeCenters,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
} from "./helpers";

async function canvasPoint(
  page: Page,
  xOffset: number,
  yOffset: number,
) {
  const canvas = page.getByTestId("canvas-root");
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  return { x: box.x + xOffset, y: box.y + yOffset };
}

async function loadDiagram(page: Page) {
  await freshStart(page);
}

test.describe("diagram mode-switch race hardening", () => {
  test("text placement remains accurate when panels shift canvas origin", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);

    const iconsPanel = page.getByRole("heading", { name: "Diagram Icons" });
    if (!(await iconsPanel.isVisible().catch(() => false))) {
      await runEffect(() =>
        page.getByRole("button", { name: "Icons", exact: true }).click(),
      );
    }

    const propertiesPanel = page.getByRole("heading", { name: "Properties" });
    if (!(await propertiesPanel.isVisible().catch(() => false))) {
      await runEffect(() =>
        page.getByRole("button", { name: "Props", exact: true }).click(),
      );
    }

    const canvas = page.getByTestId("canvas-root");
    const box = await runEffect(() => canvas.boundingBox());
    if (!box) {
      throw new Error("canvas bounding box not available");
    }

    const targetX = box.x + 320;
    const targetY = box.y + 180;
    await runEffectsSequential([
      () => page.getByRole("button", { name: "Text", exact: true }).click(),
      () => page.mouse.click(targetX, targetY),
    ]);

    await expectNodeCount(page, 1);
    const textNode = canvas.getByTestId("node").first();
    const placed = await runEffect(() => textNode.boundingBox());
    if (!placed) {
      throw new Error("text node bounds unavailable after placement");
    }

    expect(Math.abs(placed.x - targetX)).toBeLessThan(20);
    expect(Math.abs(placed.y - targetY)).toBeLessThan(20);
    expect(pageErrors).toHaveLength(0);
  });

  test("cancels pending edge draw with Escape", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 220),
      () => createTextNode(page, canvas, 820, 300),
    ]);
    await expectNodeCount(page, 2);
    await expectEdgeCount(page, 0);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes for edge cancel test");
    }

    await runEffectsSequential([
      () => page.mouse.move(centers[0].x, centers[0].y),
      () => page.mouse.down(),
      () => page.mouse.up(),
      () => page.keyboard.press("Escape"),
      () => page.mouse.move(centers[1].x, centers[1].y),
      () => page.mouse.down(),
      () => page.mouse.up(),
      () => page.keyboard.press("Escape"),
    ]);

    await expectEdgeCount(page, 0);
    await expect(page.getByRole("button", { name: "Select", exact: true })).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("switching tools mid-edge gesture does not create ghost edge", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 540, 240),
      () => createTextNode(page, canvas, 820, 340),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes for tool switch test");
    }

    await runEffectsSequential([
      () => page.mouse.move(centers[0].x, centers[0].y),
      () => page.mouse.down(),
      () => page.mouse.up(),
      () => page.getByRole("button", { name: "Select", exact: true }).click(),
      () => page.mouse.move(centers[1].x, centers[1].y),
      () => page.mouse.down(),
      () => page.mouse.up(),
    ]);

    await expectEdgeCount(page, 0);
    await expectSelectedCount(page, 1);
    expect(pageErrors).toHaveLength(0);
  });

  test("pan tool releases cleanly after drag", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const start = await canvasPoint(page, 260, 220);
    const end = await canvasPoint(page, 380, 250);

    await runEffect(() =>
      page.getByRole("button", { name: "Pan", exact: true }).click(),
    );
    await runEffectsSequential([
      () => page.mouse.move(start.x, start.y),
      () => page.mouse.down(),
      () => page.mouse.move(end.x, end.y),
      () => page.mouse.up(),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    await runEffect(() => createTextNode(page, canvas, 700, 260));

    await expectNodeCount(page, 1);
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("drag continues across canvas boundary and releases outside", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 560, 240));
    await expectNodeCount(page, 1);

    const textNode = canvas.getByTestId("node").first();
    const before = await runEffect(() => textNode.boundingBox());
    const canvasBox = await runEffect(() => canvas.boundingBox());
    if (!before || !canvasBox) {
      throw new Error("missing bounds for outside-release drag test");
    }

    await runEffectsSequential([
      () => page.mouse.move(before.x + 8, before.y + 8),
      () => page.mouse.down(),
    ]);
    const outsideX = canvasBox.x + canvasBox.width + 40;
    const outsideY = before.y + 24;
    await runEffectsSequential([
      () => page.mouse.move(outsideX, outsideY),
      () => page.mouse.up(),
    ]);

    const after = await runEffect(() => textNode.boundingBox());
    if (!after) {
      throw new Error("missing node bounds after outside-release drag");
    }
    expect(after.x).toBeGreaterThan(before.x + 20);

    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    await runEffect(() => page.mouse.click(canvasBox.x + 220, canvasBox.y + 120));
    await expectNodeCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  test("space-pan keyup race recovers to normal interactions", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 560, 230));
    await expectNodeCount(page, 1);

    const dragStart = await canvasPoint(page, 300, 220);
    const dragMid = await canvasPoint(page, 220, 220);
    const dragEnd = await canvasPoint(page, 360, 220);

    await runEffectsSequential([
      () => page.keyboard.down(" "),
      () => page.mouse.move(dragStart.x, dragStart.y),
      () => page.mouse.down(),
      () => page.mouse.move(dragMid.x, dragMid.y),
      () => page.keyboard.up(" "),
      () => page.mouse.move(dragEnd.x, dragEnd.y),
      () => page.mouse.up(),
    ]);

    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    const dropPoint = await canvasPoint(page, 780, 320);
    await runEffect(() => page.mouse.click(dropPoint.x, dropPoint.y));

    await expectNodeCount(page, 2);
    await expect(page.getByRole("button", { name: "Pan", exact: true })).toBeVisible();
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });
});
