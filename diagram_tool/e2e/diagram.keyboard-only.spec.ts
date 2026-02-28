import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  createTextNode,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
  zoomPercent,
} from "./helpers";

async function bootEditor(page: Page) {
  await freshStart(page);
}

function toolButton(page: Page, label: string): Locator {
  return page.getByRole("button", { name: label, exact: true });
}

async function pressKey(page: Page, key: string) {
  await runEffect(() => page.keyboard.press(key));
}

async function clickCanvasAt(canvas: Locator, x: number, y: number) {
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  await runEffect(() => canvas.page().mouse.click(box.x + x, box.y + y));
}

async function dragCanvas(
  canvas: Locator,
  startX: number,
  startY: number,
  endX: number,
  endY: number,
) {
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  const page = canvas.page();
  await runEffectsSequential([
    () => page.mouse.move(box.x + startX, box.y + startY),
    () => page.mouse.down(),
    () => page.mouse.move(box.x + endX, box.y + endY),
    () => page.mouse.up(),
  ]);
}

async function expectCounts(
  page: Page,
  counts: { nodes?: number; edges?: number; selected?: number },
) {
  if (counts.nodes !== undefined) {
    await expectNodeCount(page, counts.nodes);
  }
  if (counts.edges !== undefined) {
    await expectEdgeCount(page, counts.edges);
  }
  if (counts.selected !== undefined) {
    await expectSelectedCount(page, counts.selected);
  }
}

test.describe("diagram keyboard-only workflows", () => {
  test.describe.configure({ mode: "serial", timeout: 120_000 });

  test("switches tools with v/h/l/r/t and updates counters", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => bootEditor(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvasArea = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, canvasArea, 220, 180),
      () => createTextNode(page, canvasArea, 430, 280),
    ]);
    await expectCounts(page, { nodes: 2, edges: 0 });

    await runEffect(() => toolButton(page, "Select").click());
    await expect(toolButton(page, "Select")).toBeFocused();

    await runEffect(() => pressKey(page, "r"));
    await runEffect(() => dragCanvas(canvasArea, 560, 120, 700, 260));
    await expectCounts(page, { nodes: 3, edges: 0 });

    await runEffect(() => pressKey(page, "l"));
    const textNodes = canvasArea.getByTestId("node");
    await runEffect(() => textNodes.first().click());
    await runEffect(() => textNodes.nth(1).click());
    await expectCounts(page, { nodes: 3, edges: 1 });

    await runEffect(() => pressKey(page, "h"));
    await runEffect(() => clickCanvasAt(canvasArea, 80, 80));
    await expectCounts(page, { nodes: 3, edges: 1 });

    await runEffect(() => pressKey(page, "v"));
    await runEffect(() => textNodes.first().click());
    await expectCounts(page, { selected: 1, nodes: 3, edges: 1 });

    await runEffect(() => pressKey(page, "t"));
    await runEffect(() => clickCanvasAt(canvasArea, 700, 420));
    await expectCounts(page, { nodes: 4, edges: 1 });

    expect(pageErrors).toHaveLength(0);
  });

  test("handles Delete and Escape via keyboard only", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => bootEditor(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvasArea = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, canvasArea, 240, 220),
      () => createTextNode(page, canvasArea, 440, 300),
    ]);
    await expectCounts(page, { nodes: 2 });

    const textNodes = canvasArea.getByTestId("node");
    await runEffect(() => textNodes.first().click());
    await expectCounts(page, { selected: 1, nodes: 2 });

    await runEffect(() => pressKey(page, "Delete"));
    await expectCounts(page, { nodes: 1 });

    await runEffect(() => textNodes.first().click());
    await expectCounts(page, { nodes: 1 });

    await runEffect(() => pressKey(page, "Escape"));
    await expectCounts(page, { nodes: 1 });

    await runEffect(() => pressKey(page, "Escape"));
    await expectCounts(page, { nodes: 1 });

    expect(pageErrors).toHaveLength(0);
  });

  test("zooms with +, -, and 0 from keyboard", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => bootEditor(page),
      () => clearCanvasOverlays(page),
    ]);

    const zoomReset = page.getByTestId("zoom-reset");
    expect(await zoomPercent(page)).toBe(100);

    await runEffect(() => zoomReset.focus());
    await expect(zoomReset).toBeFocused();

    await runEffect(() => pressKey(page, "+"));
    expect(await zoomPercent(page)).toBe(125);
    await expect(zoomReset).toBeFocused();

    await runEffect(() => pressKey(page, "-"));
    expect(await zoomPercent(page)).toBe(100);
    await expect(zoomReset).toBeFocused();

    await runEffectsSequential([
      () => pressKey(page, "+"),
      () => pressKey(page, "+"),
    ]);
    expect(await zoomPercent(page)).toBe(156);
    await runEffect(() => pressKey(page, "0"));
    expect(await zoomPercent(page)).toBe(100);

    await expectCounts(page, { nodes: 0, edges: 0, selected: 0 });
    expect(pageErrors).toHaveLength(0);
  });

  test("ignores global shortcuts while editing inputs", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => bootEditor(page));

    const canvasArea = canvas(page);
    await runEffect(() => createTextNode(page, canvasArea, 320, 220));
    await expectCounts(page, { nodes: 1 });

    const textNode = canvasArea.getByTestId("node").first();
    await runEffect(() => textNode.click());
    await expectCounts(page, { selected: 1, nodes: 1 });

    const labelInput = page.getByTestId("node-label-input");
    await expect(labelInput).toBeVisible();
    await runEffect(() => labelInput.click());
    await expect(labelInput).toBeFocused();

    await runEffect(() => labelInput.fill("NodeOne"));
    await runEffect(() => page.keyboard.press("ControlOrMeta+a"));
    await runEffect(() => pressKey(page, "Delete"));
    await expect(labelInput).toHaveValue("");
    await expectCounts(page, { nodes: 1, selected: 1 });

    await runEffect(() => pressKey(page, "h"));
    await expect(labelInput).toHaveValue("h");
    await expect(labelInput).toBeFocused();

    const zoomReset = page.getByTestId("zoom-reset");
    expect(await zoomPercent(page)).toBe(100);
    await runEffect(() => pressKey(page, "+"));
    await expect(labelInput).toHaveValue("h+");
    expect(await zoomPercent(page)).toBe(100);

    await runEffect(() => pressKey(page, "Escape"));
    await expectCounts(page, { selected: 1, nodes: 1 });
    await expect(labelInput).toBeFocused();

    expect(pageErrors).toHaveLength(0);
  });
});
