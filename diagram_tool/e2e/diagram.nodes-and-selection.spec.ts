import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram nodes and selection", () => {
  test("creates, selects, and drags text nodes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 780, 320));

    await expect(page.getByText(/2 nodes/)).toBeVisible();

    const textNodes = canvas.getByText("Text", { exact: true });
    await runEffect(() => textNodes.first().click());
    await expect(page.getByText(/1 selected/)).toBeVisible();

    await runEffect(() => page.keyboard.down("Shift"));
    await runEffect(() => textNodes.nth(1).click());
    await runEffect(() => page.keyboard.up("Shift"));
    await expect(page.getByText(/2 selected/)).toBeVisible();

    const nodeBoundsBefore = await runEffect(() => textNodes.first().boundingBox());
    if (!nodeBoundsBefore) {
      throw new Error("text node bounds missing before drag");
    }

    await runEffect(() => page.mouse.move(nodeBoundsBefore.x + 6, nodeBoundsBefore.y + 6));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(nodeBoundsBefore.x + 56, nodeBoundsBefore.y + 46));
    await runEffect(() => page.mouse.up());

    const nodeBoundsAfter = await runEffect(() => textNodes.first().boundingBox());
    if (!nodeBoundsAfter) {
      throw new Error("text node bounds missing after drag");
    }

    expect(nodeBoundsAfter.x).toBeGreaterThan(nodeBoundsBefore.x + 20);
    expect(nodeBoundsAfter.y).toBeGreaterThan(nodeBoundsBefore.y + 20);
    expect(pageErrors).toHaveLength(0);
  });
});
