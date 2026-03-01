import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  freshStart,
  mountScrollableHarness,
  nodeCount,
  runEffectsSequential,
  runEffect,
  scrollHarnessTo,
  trapPageErrors,
  zoomPercent,
} from "./helpers";

function requireTouchProject(hasTouch: boolean) {
  test.skip(!hasTouch, "mobile touch assertions run only on touch-enabled projects");
}

test.describe("diagram mobile touch viewport stability", () => {
  test("tap placement remains accurate after embedded scroll", async ({ page, hasTouch }) => {
    requireTouchProject(hasTouch);
    const pageErrors = trapPageErrors(page);

    await freshStart(page);
    await clearCanvasOverlays(page);
    await mountScrollableHarness(page);
    await scrollHarnessTo(page, 160, 260);

    const canvas = page.getByTestId("canvas-root");
    const cbox = await runEffect(() => canvas.boundingBox());
    if (!cbox) {
      throw new Error("canvas bounds unavailable on mobile project");
    }

    const targetX = cbox.x + cbox.width * 0.45;
    const targetY = cbox.y + cbox.height * 0.4;
    await runEffectsSequential([
      () => page.getByRole("button", { name: "Text", exact: true }).click(),
      () => page.touchscreen.tap(targetX, targetY),
    ]);

    await expect(page.getByTestId("counter-nodes")).toHaveText(/1 nodes/);
    const node = page.getByTestId("node").first();
    const placed = await runEffect(() => node.boundingBox());
    if (!placed) {
      throw new Error("touch-placed node bounds unavailable");
    }

    expect(Math.abs(placed.x - targetX)).toBeLessThan(34);
    expect(Math.abs(placed.y - targetY)).toBeLessThan(34);
    expect(pageErrors).toHaveLength(0);
  });

  test("pinch-like touch events do not corrupt zoom state", async ({ page, hasTouch }) => {
    requireTouchProject(hasTouch);
    const pageErrors = trapPageErrors(page);

    await freshStart(page);
    await clearCanvasOverlays(page);

    const canvas = page.getByTestId("canvas-root");
    const cbox = await runEffect(() => canvas.boundingBox());
    if (!cbox) {
      throw new Error("canvas bounds unavailable for pinch stability");
    }

    const beforeZoom = await zoomPercent(page);
    const cx = cbox.x + cbox.width * 0.5;
    const cy = cbox.y + cbox.height * 0.45;

    await runEffect(() =>
      page.evaluate(
        ({ x, y }) => {
          const target = document.elementFromPoint(x, y);
          if (!target) {
            return;
          }

          const start = new TouchEvent("touchstart", {
            bubbles: true,
            cancelable: true,
            touches: [
              new Touch({ identifier: 1, target, clientX: x - 36, clientY: y }),
              new Touch({ identifier: 2, target, clientX: x + 36, clientY: y }),
            ],
          });
          target.dispatchEvent(start);

          const move = new TouchEvent("touchmove", {
            bubbles: true,
            cancelable: true,
            touches: [
              new Touch({ identifier: 1, target, clientX: x - 60, clientY: y }),
              new Touch({ identifier: 2, target, clientX: x + 60, clientY: y }),
            ],
          });
          target.dispatchEvent(move);

          const end = new TouchEvent("touchend", {
            bubbles: true,
            cancelable: true,
            touches: [],
          });
          target.dispatchEvent(end);
        },
        { x: cx, y: cy },
      ),
    );

    const afterZoom = await zoomPercent(page);
    expect(Number.isFinite(beforeZoom)).toBe(true);
    expect(Number.isFinite(afterZoom)).toBe(true);
    expect(afterZoom).toBeGreaterThanOrEqual(10);
    expect(afterZoom).toBeLessThanOrEqual(400);
    expect(pageErrors).toHaveLength(0);
  });

  test("orientation viewport changes preserve touch hit testing", async ({ page, hasTouch }) => {
    requireTouchProject(hasTouch);
    const pageErrors = trapPageErrors(page);

    await freshStart(page);
    await clearCanvasOverlays(page);
    await page.setViewportSize({ width: 412, height: 915 });
    await page.setViewportSize({ width: 915, height: 412 });

    const canvas = page.getByTestId("canvas-root");
    const cbox = await runEffect(() => canvas.boundingBox());
    if (!cbox) {
      throw new Error("canvas bounds unavailable after viewport change");
    }

    await runEffectsSequential([
      () => page.getByRole("button", { name: "Text", exact: true }).click(),
      () => page.touchscreen.tap(cbox.x + 260, cbox.y + 160),
    ]);
    await expect(page.getByTestId("counter-nodes")).toHaveText(/1 nodes/);
    expect(pageErrors).toHaveLength(0);
  });

  test("two-finger touch move does not mutate node count without explicit placement", async ({
    page,
    hasTouch,
  }) => {
    requireTouchProject(hasTouch);
    const pageErrors = trapPageErrors(page);

    await freshStart(page);
    await clearCanvasOverlays(page);
    await mountScrollableHarness(page);
    await scrollHarnessTo(page, 120, 220);

    const canvas = page.getByTestId("canvas-root");
    const cbox = await runEffect(() => canvas.boundingBox());
    if (!cbox) {
      throw new Error("canvas bounds unavailable for two-finger move contract");
    }

    const beforeNodes = await nodeCount(page);
    const cx = cbox.x + cbox.width * 0.52;
    const cy = cbox.y + cbox.height * 0.48;

    await runEffect(() =>
      page.evaluate(
        ({ x, y }) => {
          const target = document.elementFromPoint(x, y);
          if (!target) {
            return;
          }

          const start = new TouchEvent("touchstart", {
            bubbles: true,
            cancelable: true,
            touches: [
              new Touch({ identifier: 10, target, clientX: x - 28, clientY: y - 6 }),
              new Touch({ identifier: 11, target, clientX: x + 28, clientY: y + 6 }),
            ],
          });
          target.dispatchEvent(start);

          const move = new TouchEvent("touchmove", {
            bubbles: true,
            cancelable: true,
            touches: [
              new Touch({ identifier: 10, target, clientX: x - 52, clientY: y - 24 }),
              new Touch({ identifier: 11, target, clientX: x + 52, clientY: y + 24 }),
            ],
          });
          target.dispatchEvent(move);

          const end = new TouchEvent("touchend", {
            bubbles: true,
            cancelable: true,
            touches: [],
          });
          target.dispatchEvent(end);
        },
        { x: cx, y: cy },
      ),
    );

    const afterNodes = await nodeCount(page);
    expect(afterNodes).toBe(beforeNodes);
    expect(pageErrors).toHaveLength(0);
  });
});
