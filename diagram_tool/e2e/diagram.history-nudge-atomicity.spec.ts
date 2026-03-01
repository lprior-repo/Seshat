import { expect, test, type Locator } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  freshStart,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
} from "./helpers";

async function nodeX(node: Locator): Promise<number> {
  const box = await runEffect(() => node.boundingBox());
  if (!box) {
    throw new Error("expected node frame");
  }
  return box.x;
}

test.describe("history nudge gesture atomicity", () => {
  test("repeated arrow keydowns collapse to one undo entry @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 620, 260));

    const node = page.getByTestId("node").first();
    await runEffect(() => node.click());

    const before = await nodeX(node);

    await runEffect(() =>
      page.evaluate(() => {
        for (let index = 0; index < 6; index += 1) {
          window.dispatchEvent(
            new KeyboardEvent("keydown", {
              key: "ArrowRight",
              repeat: index > 0,
              bubbles: true,
            }),
          );
        }
        window.dispatchEvent(
          new KeyboardEvent("keyup", {
            key: "ArrowRight",
            bubbles: true,
          }),
        );
      }),
    );

    const afterBatch = await nodeX(node);
    expect(afterBatch).toBeGreaterThan(before + 1);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    const undone = await nodeX(node);
    expect(undone).toBeGreaterThanOrEqual(before - 1);
    expect(undone).toBeLessThanOrEqual(before + 1);

    await runEffect(() => page.getByRole("button", { name: "Redo", exact: true }).click());
    const redone = await nodeX(node);
    expect(redone).toBeGreaterThanOrEqual(afterBatch - 1);
    expect(redone).toBeLessThanOrEqual(afterBatch + 1);
    expect(pageErrors).toHaveLength(0);
  });

  test("blur between nudges starts a fresh undo gesture @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    await runEffect(() => createTextNode(page, canvas, 640, 280));

    const node = page.getByTestId("node").first();
    await runEffect(() => node.click());

    const before = await nodeX(node);

    await runEffect(() =>
      page.evaluate(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "ArrowRight",
            bubbles: true,
          }),
        );
      }),
    );

    const afterFirstNudge = await nodeX(node);
    expect(afterFirstNudge).toBeGreaterThan(before);

    await runEffect(() =>
      page.evaluate(() => {
        window.dispatchEvent(new Event("blur"));
      }),
    );

    await runEffect(() => page.keyboard.press("ArrowRight"));
    const afterSecondNudge = await nodeX(node);
    expect(afterSecondNudge).toBeGreaterThan(afterFirstNudge);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    const afterUndo = await nodeX(node);

    expect(afterUndo).toBeLessThan(afterSecondNudge);
    expect(afterUndo).toBeGreaterThanOrEqual(afterFirstNudge - 1);
    expect(afterUndo).toBeLessThanOrEqual(afterFirstNudge + 1);
    expect(pageErrors).toHaveLength(0);
  });
});
