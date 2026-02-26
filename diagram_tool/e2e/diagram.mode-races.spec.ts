import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeCenters,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

async function canvasPoint(
  page: Page,
  xOffset: number,
  yOffset: number,
) {
  const canvas = page.locator(".canvas-container");
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  return { x: box.x + xOffset, y: box.y + yOffset };
}

async function loadDiagram(page: Page) {
  let lastError: Error | undefined;
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      await runEffect(() => page.goto("/", { waitUntil: "domcontentloaded" }));
      await runEffect(() => waitForUiReady(page));
      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await runEffect(() => page.waitForTimeout(1_000));
    }
  }

  throw new Error(
    `unable to load diagram after retries: ${lastError?.message ?? "unknown error"}`,
  );
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

    const canvas = page.locator(".canvas-container");
    const box = await runEffect(() => canvas.boundingBox());
    if (!box) {
      throw new Error("canvas bounding box not available");
    }

    const targetX = box.x + 320;
    const targetY = box.y + 180;
    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    await runEffect(() => page.mouse.click(targetX, targetY));

    await expect(page.getByText(/1 nodes/)).toBeVisible();
    const textNode = canvas.getByText("Text", { exact: true }).first();
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

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 520, 220));
    await runEffect(() => createTextNode(page, canvas, 820, 300));
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(page.getByText(/0 edges/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes for edge cancel test");
    }

    await runEffect(() => page.mouse.move(centers[0].x, centers[0].y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await runEffect(() => page.keyboard.press("Escape"));
    await runEffect(() => page.mouse.move(centers[1].x, centers[1].y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await runEffect(() => page.keyboard.press("Escape"));

    await expect(page.getByText(/0 edges/)).toBeVisible();
    await expect(page.getByRole("button", { name: "Select", exact: true })).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("switching tools mid-edge gesture does not create ghost edge", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 540, 240));
    await runEffect(() => createTextNode(page, canvas, 820, 340));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes for tool switch test");
    }

    await runEffect(() => page.mouse.move(centers[0].x, centers[0].y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );
    await runEffect(() => page.mouse.move(centers[1].x, centers[1].y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());

    await expect(page.getByText(/0 edges/)).toBeVisible();
    await expect(page.getByText(/1 selected/)).toBeVisible();
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
    await runEffect(() => page.mouse.move(start.x, start.y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(end.x, end.y));
    await runEffect(() => page.mouse.up());

    const canvas = page.locator(".canvas-container");
    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    await runEffect(() => createTextNode(page, canvas, 700, 260));

    await expect(page.getByText(/1 nodes/)).toBeVisible();
    await expect(page.getByText(/0 edges/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("drag continues across canvas boundary and releases outside", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 240));
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    const textNode = canvas.getByText("Text", { exact: true }).first();
    const before = await runEffect(() => textNode.boundingBox());
    const canvasBox = await runEffect(() => canvas.boundingBox());
    if (!before || !canvasBox) {
      throw new Error("missing bounds for outside-release drag test");
    }

    await runEffect(() => page.mouse.move(before.x + 8, before.y + 8));
    await runEffect(() => page.mouse.down());
    const outsideX = canvasBox.x + canvasBox.width + 40;
    const outsideY = before.y + 24;
    await runEffect(() => page.mouse.move(outsideX, outsideY));
    await runEffect(() => page.mouse.up());

    const after = await runEffect(() => textNode.boundingBox());
    if (!after) {
      throw new Error("missing node bounds after outside-release drag");
    }
    expect(after.x).toBeGreaterThan(before.x + 20);

    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    await runEffect(() => page.mouse.click(canvasBox.x + 220, canvasBox.y + 120));
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("space-pan keyup race recovers to normal interactions", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await loadDiagram(page);
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 230));
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    const dragStart = await canvasPoint(page, 300, 220);
    const dragMid = await canvasPoint(page, 220, 220);
    const dragEnd = await canvasPoint(page, 360, 220);

    await runEffect(() => page.keyboard.down(" "));
    await runEffect(() => page.mouse.move(dragStart.x, dragStart.y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(dragMid.x, dragMid.y));
    await runEffect(() => page.keyboard.up(" "));
    await runEffect(() => page.mouse.move(dragEnd.x, dragEnd.y));
    await runEffect(() => page.mouse.up());

    await runEffect(() =>
      page.getByRole("button", { name: "Text", exact: true }).click(),
    );
    const dropPoint = await canvasPoint(page, 780, 320);
    await runEffect(() => page.mouse.click(dropPoint.x, dropPoint.y));

    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(page.getByRole("button", { name: "Pan", exact: true })).toBeVisible();
    await expect(page.getByText(/0 edges/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
