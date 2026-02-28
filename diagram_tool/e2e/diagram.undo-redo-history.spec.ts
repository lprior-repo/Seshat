import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  freshStart,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
} from "./helpers";

test.describe("diagram undo and redo", () => {
  test("restores node counts across undo and redo", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 560, 220),
      () => createTextNode(page, canvas, 790, 320),
    ]);
    await expectNodeCount(page, 2);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expectNodeCount(page, 1);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    await expectNodeCount(page, 0);

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expectNodeCount(page, 1);

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    await expectNodeCount(page, 2);

    expect(pageErrors).toHaveLength(0);
  });
});
