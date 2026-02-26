import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram undo and redo", () => {
  test("restores node counts across undo and redo", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 790, 320));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expect(page.getByText(/0 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    expect(pageErrors).toHaveLength(0);
  });
});
