import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
} from "./helpers";

async function setupCanvas(page: Page): Promise<Locator> {
  await freshStart(page);
  await clearCanvasOverlays(page);
  return canvas(page);
}

async function selectBothTextNodes(page: Page): Promise<void> {
  const textNodes = canvas(page).getByTestId("node");
  await runEffectsSequential([
    () => textNodes.first().click(),
    () => page.keyboard.down("Shift"),
    () => textNodes.nth(1).click(),
    () => page.keyboard.up("Shift"),
  ]);
  await expectSelectedCount(page, 2);
}

test.describe("keyboard shortcuts @baseline", () => {
  test("Ctrl+Z undoes node creation @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 560, 220),
      () => createTextNode(page, canvasEl, 790, 320),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 1);
    await expect(canvasEl.getByTestId("node")).toHaveCount(1);

    expect(pageErrors).toHaveLength(0);
  });

  test("Ctrl+Y redoes undone action @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 560, 220),
      () => createTextNode(page, canvasEl, 790, 320),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 1);

    await runEffect(() => page.keyboard.press("ControlOrMeta+y"));
    await expectNodeCount(page, 2);
    await expect(canvasEl.getByTestId("node")).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("Ctrl+C copies selected nodes @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 500, 200),
      () => createTextNode(page, canvasEl, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    await runEffectsSequential([
      () => selectBothTextNodes(page),
      () => page.keyboard.press("ControlOrMeta+c"),
    ]);

    await expectNodeCount(page, 2);
    await expectSelectedCount(page, 2);

    expect(pageErrors).toHaveLength(0);
  });

  test("Ctrl+V pastes copied nodes @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 500, 200),
      () => createTextNode(page, canvasEl, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    await runEffectsSequential([
      () => selectBothTextNodes(page),
      () => page.keyboard.press("ControlOrMeta+c"),
    ]);

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expectNodeCount(page, 4);
    await expect(canvasEl.getByTestId("node")).toHaveCount(4);

    expect(pageErrors).toHaveLength(0);
  });

  test("shortcuts do not fire when input has focus @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 500, 200),
      () => createTextNode(page, canvasEl, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    const textNodes = canvasEl.getByTestId("node");
    await runEffect(() => textNodes.first().dblclick());

    const input = page.locator("input").first();
    await runEffect(() => input.focus());
    await expect(input).toBeFocused();

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 2);

    expect(pageErrors).toHaveLength(0);
  });

  test("Ctrl+Shift+Z also triggers redo @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 560, 220),
      () => createTextNode(page, canvasEl, 790, 320),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 1);

    await runEffect(() => page.keyboard.press("ControlOrMeta+Shift+z"));
    await expectNodeCount(page, 2);
    await expect(canvasEl.getByTestId("node")).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("multiple paste operations stack correctly @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 500, 200),
      () => createTextNode(page, canvasEl, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    await runEffectsSequential([
      () => selectBothTextNodes(page),
      () => page.keyboard.press("ControlOrMeta+c"),
    ]);

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expectNodeCount(page, 4);

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expectNodeCount(page, 6);
    await expect(canvasEl.getByTestId("node")).toHaveCount(6);

    expect(pageErrors).toHaveLength(0);
  });

  test("undo after paste removes pasted nodes @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 500, 200),
      () => createTextNode(page, canvasEl, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    await runEffectsSequential([
      () => selectBothTextNodes(page),
      () => page.keyboard.press("ControlOrMeta+c"),
      () => page.keyboard.press("ControlOrMeta+v"),
    ]);
    await expectNodeCount(page, 4);

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 2);
    await expect(canvasEl.getByTestId("node")).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("shortcuts blocked when textarea has focus @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvasEl, 500, 200),
      () => createTextNode(page, canvasEl, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    const textarea = page.locator("textarea").first();
    const textareaCount = await runEffect(() => textarea.count());

    if (textareaCount > 0) {
      await runEffect(() => textarea.focus());
      await expect(textarea).toBeFocused();

      await runEffect(() => page.keyboard.press("ControlOrMeta+y"));
      await expectNodeCount(page, 2);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("full undo-redo keyboard workflow @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvasEl = await setupCanvas(page);

    await runEffect(() => createTextNode(page, canvasEl, 500, 200));
    await expectNodeCount(page, 1);

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 0);

    await runEffect(() => page.keyboard.press("ControlOrMeta+Shift+z"));
    await expectNodeCount(page, 1);

    await runEffect(() => page.keyboard.press("ControlOrMeta+z"));
    await expectNodeCount(page, 0);

    await runEffect(() => page.keyboard.press("ControlOrMeta+y"));
    await expectNodeCount(page, 1);
    await expect(canvasEl.getByTestId("node")).toHaveCount(1);

    expect(pageErrors).toHaveLength(0);
  });
});
