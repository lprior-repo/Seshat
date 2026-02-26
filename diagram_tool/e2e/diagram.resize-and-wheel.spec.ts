import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

async function firstNodeBox(page: Parameters<typeof test>[0]["page"]) {
  const node = page.locator(".canvas-container").getByText("Text", { exact: true }).first();
  const box = await runEffect(() => node.boundingBox());
  if (!box) {
    throw new Error("node bounds unavailable");
  }
  return box;
}

test.describe("diagram resize and wheel behavior", () => {
  test("wheel on canvas zooms editor and does not scroll page", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const zoomLabel = page.getByRole("button", { name: /\d+%/ }).first();
    await expect(zoomLabel).toHaveText("100%");

    const canvas = page.locator(".canvas-container");
    const box = await runEffect(() => canvas.boundingBox());
    if (!box) {
      throw new Error("canvas bounds unavailable");
    }

    const beforeScroll = await runEffect(() => page.evaluate(() => window.scrollY));
    await runEffect(() => page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.4));
    await runEffect(() => page.mouse.wheel(0, -180));
    await expect(zoomLabel).not.toHaveText("100%");

    const afterScroll = await runEffect(() => page.evaluate(() => window.scrollY));
    expect(afterScroll).toBe(beforeScroll);
    expect(pageErrors).toHaveLength(0);
  });

  test("resize interaction updates dimensions progressively and stays finite", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 620, 280));
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());
    await expect(page.getByText(/1 selected/)).toBeVisible();

    const before = await firstNodeBox(page);
    const handles = canvas.locator('button[style*="ew-resize"]');
    const east = await runEffect(() => handles.last().boundingBox());
    if (!east) {
      throw new Error("resize handle bounds unavailable");
    }

    await runEffect(() => page.mouse.move(east.x + east.width / 2, east.y + east.height / 2));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(east.x - 60, east.y, { steps: 4 }));
    const mid = await firstNodeBox(page);
    await runEffect(() => page.mouse.move(east.x - 140, east.y, { steps: 4 }));
    await runEffect(() => page.mouse.up());
    const after = await firstNodeBox(page);

    expect(Number.isFinite(mid.width)).toBe(true);
    expect(Number.isFinite(after.width)).toBe(true);
    expect(mid.width).toBeLessThanOrEqual(before.width);
    expect(after.width).toBeLessThanOrEqual(mid.width);
    expect(after.width).toBeGreaterThanOrEqual(24);
    expect(pageErrors).toHaveLength(0);
  });

  test("small handle drag does not jump from viewport-offset math", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 700, 320));
    await expect(page.getByText(/1 nodes/)).toBeVisible();

    const node = canvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());

    const before = await firstNodeBox(page);
    const handles = canvas.locator('button[style*="ew-resize"]');
    const east = await runEffect(() => handles.last().boundingBox());
    if (!east) {
      throw new Error("east resize handle unavailable");
    }

    const cx = east.x + east.width / 2;
    const cy = east.y + east.height / 2;
    await runEffect(() => page.mouse.move(cx, cy));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(cx - 12, cy, { steps: 3 }));
    await runEffect(() => page.mouse.up());

    const after = await firstNodeBox(page);
    const delta = Math.abs(after.width - before.width);

    expect(delta).toBeGreaterThanOrEqual(0);
    expect(delta).toBeLessThanOrEqual(40);
    expect(pageErrors).toHaveLength(0);
  });
});
