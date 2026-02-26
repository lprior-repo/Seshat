import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

async function setupCanvas(page: Parameters<typeof test>[0]["page"]) {
  await runEffect(() => page.goto("/"));
  await runEffect(() => waitForUiReady(page));
  await runEffect(() => clearCanvasOverlays(page));
  return page.locator(".canvas-container");
}

async function selectBothTextNodes(page: Parameters<typeof test>[0]["page"]) {
  const textNodes = page.locator(".canvas-container").getByText("Text", { exact: true });
  await runEffect(() => textNodes.first().click());
  await runEffect(() => page.keyboard.down("Shift"));
  await runEffect(() => textNodes.nth(1).click());
  await runEffect(() => page.keyboard.up("Shift"));
  await expect(page.getByText(/2 selected/)).toBeVisible();
}

test.describe("diagram history and clipboard", () => {
  test("tracks mixed clipboard/delete operations across undo-redo timeline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 800, 320));
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(2);

    await runEffect(() => selectBothTextNodes(page));
    await runEffect(() => page.keyboard.press("ControlOrMeta+c"));
    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expect(page.getByText(/4 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(4);

    await runEffect(() => page.getByRole("button", { name: "Delete", exact: true }).click());
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/4 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expect(page.getByText(/4 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("invalidates redo stack after a new edit", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffect(() => createTextNode(page, canvas, 520, 200));
    await runEffect(() => createTextNode(page, canvas, 760, 300));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    await runEffect(() => createTextNode(page, canvas, 920, 360));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(2);

    expect(pageErrors).toHaveLength(0);
  });

  test("duplicates selected nodes with copy paste and preserves counts", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffect(() => createTextNode(page, canvas, 500, 200));
    await runEffect(() => createTextNode(page, canvas, 740, 280));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => selectBothTextNodes(page));
    await runEffect(() => page.keyboard.press("ControlOrMeta+c"));

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expect(page.getByText(/4 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(4);

    await runEffect(() => page.keyboard.press("ControlOrMeta+v"));
    await expect(page.getByText(/6 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(6);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/4 nodes/)).toBeVisible();

    expect(pageErrors).toHaveLength(0);
  });

  test("restores deleted nodes through undo", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupCanvas(page);

    await runEffect(() => createTextNode(page, canvas, 600, 220));
    await runEffect(() => createTextNode(page, canvas, 840, 300));
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(2);

    const textNodes = canvas.getByText("Text", { exact: true });
    await runEffect(() => textNodes.first().click());
    await expect(page.getByText(/1 selected/)).toBeVisible();

    await runEffect(() => page.keyboard.press("Delete"));
    await expect(page.getByText(/1 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(1);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/2 nodes/)).toBeVisible();
    await expect(canvas.getByText("Text", { exact: true })).toHaveCount(2);

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    expect(pageErrors).toHaveLength(0);
  });
});
