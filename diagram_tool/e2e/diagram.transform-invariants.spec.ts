import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForNoRebuildOverlay,
  waitForUiReady,
} from "./helpers";

type Box = {
  x: number;
  y: number;
  width: number;
  height: number;
  cx: number;
  cy: number;
};

type Point = {
  x: number;
  y: number;
};

async function nodeBoxes(canvas: Locator): Promise<Box[]> {
  return runEffect(() =>
    canvas
      .getByTestId("diagram-node")
      .evaluateAll((elements) =>
        elements
          .map((element) => {
            const rect = element.getBoundingClientRect();
            return {
              x: rect.x,
              y: rect.y,
              width: rect.width,
              height: rect.height,
              cx: rect.x + rect.width / 2,
              cy: rect.y + rect.height / 2,
            };
          })
          .sort((a, b) => a.x - b.x),
      ),
  );
}

async function canvasPoint(canvas: Locator, offset: Point): Promise<Point> {
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  return { x: box.x + offset.x, y: box.y + offset.y };
}

async function dragMouse(page: Page, from: Point, to: Point) {
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 6 });
  await page.mouse.up();
}

async function resizeHandleCenters(canvas: Locator, cursor: string): Promise<Point[]> {
  const handleName = cursor === "ew-resize" ? "e" : "se";
  return runEffect(() =>
    canvas
      .locator(`[data-testid="selection-handle"][data-handle="${handleName}"]`)
      .evaluateAll((elements) =>
        elements
          .map((element) => {
            const rect = element.getBoundingClientRect();
            return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
          })
          .sort((a, b) => a.x - b.x),
      ),
  );
}

test.describe("diagram transform invariants", () => {
  test("drag threshold no-op vs real drag", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 240));
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);

    const before = (await nodeBoxes(canvas))[0];

    await runEffect(() =>
      dragMouse(
        page,
        { x: before.x + before.width / 2, y: before.y + before.height / 2 },
        { x: before.x + before.width / 2 + 1, y: before.y + before.height / 2 + 1 },
      ),
    );

    const afterTinyDrag = (await nodeBoxes(canvas))[0];

    expect(Math.abs(afterTinyDrag.x - before.x)).toBeLessThan(2);
    expect(Math.abs(afterTinyDrag.y - before.y)).toBeLessThan(2);
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);

    await runEffect(() =>
      dragMouse(
        page,
        { x: afterTinyDrag.x + 8, y: afterTinyDrag.y + 8 },
        { x: afterTinyDrag.x + 68, y: afterTinyDrag.y + 48 },
      ),
    );

    const afterRealDrag = (await nodeBoxes(canvas))[0];

    expect(afterRealDrag.x - afterTinyDrag.x).toBeGreaterThan(30);
    expect(afterRealDrag.y - afterTinyDrag.y).toBeGreaterThan(20);
    await expect(page.getByText(/1 nodes/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("resize min clamp behavior", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 620, 260));
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(0);
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);

    const before = (await nodeBoxes(canvas))[0];

    const horizontalHandles = await resizeHandleCenters(canvas, "ew-resize");
    if (horizontalHandles.length < 2) {
      throw new Error("expected horizontal resize handles");
    }
    const eastHandle = horizontalHandles[horizontalHandles.length - 1];

    await runEffect(() =>
      dragMouse(page, eastHandle, { x: eastHandle.x - 240, y: eastHandle.y }),
    );

    const after = (await nodeBoxes(canvas))[0];

    expect(before.width - after.width).toBeGreaterThan(60);
    expect(after.width).toBeGreaterThan(21);
    expect(after.width).toBeLessThan(30);
    expect(Math.abs(after.height - before.height)).toBeLessThan(2);
    await expect(page.getByText(/1 selected/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("shift multi-select drag moves cohort", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 520, 220));
    await runEffect(() => createTextNode(page, canvas, 740, 240));
    await runEffect(() => createTextNode(page, canvas, 960, 260));
    await expect(page.getByText(/3 nodes/)).toBeVisible();

    const initial = await nodeBoxes(canvas);
    expect(initial).toHaveLength(3);

    await runEffect(() =>
      page.mouse.click(initial[0].x + initial[0].width / 2, initial[0].y + initial[0].height / 2),
    );
    await runEffect(() => page.keyboard.down("Shift"));
    await runEffect(() =>
      page.mouse.click(initial[1].x + initial[1].width / 2, initial[1].y + initial[1].height / 2),
    );
    await runEffect(() => page.keyboard.up("Shift"));
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);

    const before = await nodeBoxes(canvas);
    expect(before).toHaveLength(3);

    await runEffect(() =>
      dragMouse(
        page,
        { x: before[0].x + 8, y: before[0].y + 8 },
        { x: before[0].x + 88, y: before[0].y + 58 },
      ),
    );

    const after = await nodeBoxes(canvas);
    expect(after).toHaveLength(3);

    const movedA = { dx: after[0].x - before[0].x, dy: after[0].y - before[0].y };
    const movedB = { dx: after[1].x - before[1].x, dy: after[1].y - before[1].y };
    const staticC = { dx: after[2].x - before[2].x, dy: after[2].y - before[2].y };

    expect(movedA.dx).toBeGreaterThan(40);
    expect(movedA.dy).toBeGreaterThan(30);
    expect(movedB.dx).toBeGreaterThan(40);
    expect(movedB.dy).toBeGreaterThan(30);
    expect(Math.abs(movedA.dx - movedB.dx)).toBeLessThan(2);
    expect(Math.abs(movedA.dy - movedB.dy)).toBeLessThan(2);
    expect(Math.abs(staticC.dx)).toBeLessThan(2);
    expect(Math.abs(staticC.dy)).toBeLessThan(2);
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);
    expect(pageErrors).toHaveLength(0);
  });

  test("rubber-band selection invariants", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 520, 220));
    await runEffect(() => createTextNode(page, canvas, 700, 240));
    await runEffect(() => createTextNode(page, canvas, 980, 280));
    await runEffect(() => waitForNoRebuildOverlay(page));
    await expect(page.getByText(/3 nodes/)).toBeVisible();

    const nodes = canvas.getByText("Text", { exact: true });
    await runEffect(() => nodes.nth(2).click());
    await expect(page.getByText(/1 selected/)).toBeVisible();

    const initialBoxes = await nodeBoxes(canvas);
    expect(initialBoxes).toHaveLength(3);

    const firstTwo = initialBoxes.slice(0, 2);
    const bandStart = {
      x: Math.min(firstTwo[0].x, firstTwo[1].x) - 20,
      y: Math.min(firstTwo[0].y, firstTwo[1].y) - 20,
    };
    const bandEnd = {
      x: Math.max(firstTwo[0].x + firstTwo[0].width, firstTwo[1].x + firstTwo[1].width) + 20,
      y: Math.max(firstTwo[0].y + firstTwo[0].height, firstTwo[1].y + firstTwo[1].height) + 20,
    };

    await runEffect(() => page.mouse.move(bandStart.x, bandStart.y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(bandEnd.x, bandEnd.y, { steps: 8 }));
    await runEffect(() => page.mouse.up());

    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);

    const toggleStart = {
      x: initialBoxes[0].x - 10,
      y: initialBoxes[0].y - 10,
    };
    const toggleEnd = {
      x: initialBoxes[0].x + initialBoxes[0].width + 10,
      y: initialBoxes[0].y + initialBoxes[0].height + 10,
    };

    await runEffect(() => page.keyboard.down("Shift"));
    await runEffect(() => page.mouse.move(toggleStart.x, toggleStart.y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(toggleEnd.x, toggleEnd.y, { steps: 8 }));
    await runEffect(() => page.mouse.up());
    await runEffect(() => page.keyboard.up("Shift"));

    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
    expect(await selectedCount(page)).toBeLessThanOrEqual(2);
    await expect(page.getByText(/3 nodes/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
