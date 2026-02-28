import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  createTextNode,
  expectSelectedCount,
  freshStart,
  runEffect,
  runEffectsSequential,
  trapPageErrors,
} from "./helpers";

async function setupFreshPage(page: Page): Promise<Locator> {
  await freshStart(page);
  await clearCanvasOverlays(page);
  return canvas(page);
}

test.describe("toolbar button disabled states", () => {
  test("Undo disabled on fresh document @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await setupFreshPage(page);

    const undoButton = page.getByTestId("toolbar-undo");
    await expect(undoButton).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Undo enabled after edit @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));

    const undoButton = page.getByTestId("toolbar-undo");
    await expect(undoButton).toBeEnabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Redo disabled on fresh document @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await setupFreshPage(page);

    const redoButton = page.getByTestId("toolbar-redo");
    await expect(redoButton).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Redo enabled after undo @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => page.getByTestId("toolbar-undo").click());

    const redoButton = page.getByTestId("toolbar-redo");
    await expect(redoButton).toBeEnabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Redo disabled after all redos exhausted @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));

    await runEffect(() => page.getByTestId("toolbar-undo").click());

    const redoButton = page.getByTestId("toolbar-redo");
    await expect(redoButton).toBeEnabled();

    await runEffect(() => redoButton.click());
    await expect(redoButton).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Copy disabled with no selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await setupFreshPage(page);

    const copyButton = page.getByTestId("toolbar-copy");
    await expect(copyButton).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Copy enabled with selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => canvas.getByTestId("node").first().click());
    await expectSelectedCount(page, 1);

    const copyButton = page.getByTestId("toolbar-copy");
    await expect(copyButton).toBeEnabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Paste disabled with empty clipboard @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await setupFreshPage(page);

    const pasteButton = page.getByTestId("toolbar-paste");
    await expect(pasteButton).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Paste enabled after copy @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffectsSequential([
      () => canvas.getByTestId("node").first().click(),
      () => expectSelectedCount(page, 1),
      () => page.getByTestId("toolbar-copy").click(),
    ]);

    await runEffect(() => page.getByTestId("tool-select").click());

    const pasteButton = page.getByTestId("toolbar-paste");
    await expect(pasteButton).toBeEnabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("Copy disabled after selection cleared @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => canvas.getByTestId("node").first().click());
    await expectSelectedCount(page, 1);

    const copyButton = page.getByTestId("toolbar-copy");
    await expect(copyButton).toBeEnabled();

    await runEffect(() => page.keyboard.press("Escape"));
    await expectSelectedCount(page, 0);
    await expect(copyButton).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("All buttons disabled initially @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await setupFreshPage(page);

    await expect(page.getByTestId("toolbar-undo")).toBeDisabled();
    await expect(page.getByTestId("toolbar-redo")).toBeDisabled();
    await expect(page.getByTestId("toolbar-copy")).toBeDisabled();
    await expect(page.getByTestId("toolbar-paste")).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });

  test("State transitions after edit cycle @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupFreshPage(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffectsSequential([
      () => canvas.getByTestId("node").first().click(),
      () => expectSelectedCount(page, 1),
      () => page.getByTestId("toolbar-copy").click(),
      () => page.getByTestId("tool-select").click(),
    ]);

    await expect(page.getByTestId("toolbar-copy")).toBeEnabled();
    await expect(page.getByTestId("toolbar-paste")).toBeEnabled();
    await expect(page.getByTestId("toolbar-undo")).toBeEnabled();
    await expect(page.getByTestId("toolbar-redo")).toBeDisabled();

    expect(pageErrors).toHaveLength(0);
  });
});
