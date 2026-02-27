import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  expectSelectedCount,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

async function setupCanvas(page: Page): Promise<Locator> {
  await runEffectsSequential([
    () => page.goto("/"),
    () => waitForUiReady(page),
    () => clearCanvasOverlays(page),
  ]);
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

test.describe("diagram history and clipboard", () => {
  test("tracks mixed clipboard/delete operations across undo-redo timeline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvas, 560, 220),
      () => createTextNode(page, canvas, 800, 320),
    ]);
    await expectNodeCount(page, 2);
    await expect(canvas.getByTestId("node")).toHaveCount(2);

    await runEffectsSequential([
      () => selectBothTextNodes(page),
      () => page.keyboard.press("ControlOrMeta+c"),
      () => page.keyboard.press("ControlOrMeta+v"),
    ]);
    await expectNodeCount(page, 4);
    await expect(canvas.getByTestId("node")).toHaveCount(4);

    await runEffect(() => page.getByTestId("toolbar-delete").click());
    await expectNodeCount(page, 2);

    await runEffect(() => page.getByTestId("toolbar-undo").click());
    await expectNodeCount(page, 4);

    await runEffect(() => page.getByTestId("toolbar-undo").click());
    await expectNodeCount(page, 2);

    await runEffect(() => page.getByTestId("toolbar-redo").click());
    await expectNodeCount(page, 4);

    await runEffect(() => page.getByTestId("toolbar-redo").click());
    await expectNodeCount(page, 2);
    await expect(canvas.getByTestId("node")).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("invalidates redo stack after a new edit", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 200),
      () => createTextNode(page, canvas, 760, 300),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() => page.getByTestId("toolbar-undo").click());
    await expectNodeCount(page, 1);

    await runEffect(() => createTextNode(page, canvas, 920, 360));
    await expectNodeCount(page, 2);

    await runEffect(() => page.getByTestId("toolbar-redo").click());
    await expectNodeCount(page, 2);
    await expect(canvas.getByTestId("node")).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("duplicates selected nodes with copy paste and preserves counts", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvas, 500, 200),
      () => createTextNode(page, canvas, 740, 280),
    ]);
    await expectNodeCount(page, 2);

    await runEffectsSequential([
      () => selectBothTextNodes(page),
      () => page.keyboard.press("ControlOrMeta+c"),
    ]);

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expectNodeCount(page, 4);
    await expect(canvas.getByTestId("node")).toHaveCount(4);

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expectNodeCount(page, 6);
    await expect(canvas.getByTestId("node")).toHaveCount(6);

    await runEffect(() => page.getByTestId("toolbar-undo").click());
    await expectNodeCount(page, 4);

    expect(pageErrors).toHaveLength(0);
  });

  test("restores deleted nodes through undo", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffectsSequential([
      () => createTextNode(page, canvas, 600, 220),
      () => createTextNode(page, canvas, 840, 300),
    ]);
    await expectNodeCount(page, 2);
    await expect(canvas.getByTestId("node")).toHaveCount(2);

    const textNodes = canvas.getByTestId("node");
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    await runEffect(() => page.keyboard.press("Delete"));
    await expectNodeCount(page, 1);
    await expect(canvas.getByTestId("node")).toHaveCount(1);

    await runEffect(() => page.getByTestId("toolbar-undo").click());
    await expectNodeCount(page, 2);
    await expect(canvas.getByTestId("node")).toHaveCount(2);

    await runEffect(() => page.getByTestId("toolbar-redo").click());
    await expectNodeCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });
});
