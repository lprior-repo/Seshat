import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram undo and redo", () => {
  test("restores node counts across undo and redo", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 560, 220),
      () => createTextNode(page, canvas, 790, 320),
    ]);
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
