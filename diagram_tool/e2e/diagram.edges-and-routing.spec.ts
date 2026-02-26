import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeCenters,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram edges and routing", () => {
  async function edgeClick(page: Page, x: number, y: number) {
    await runEffect(() => page.mouse.move(x, y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
  }

  test("connects nodes with edge tool", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");

    await runEffect(() => createTextNode(page, canvas, 560, 210));
    await runEffect(() => createTextNode(page, canvas, 820, 330));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes to connect");
    }

    await edgeClick(page, centers[0].x, centers[0].y);
    await edgeClick(page, centers[1].x, centers[1].y);

    await expect(page.getByText(/1 edges/)).toBeVisible();
    await expect(page.getByText(/\d+ selected/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("rejects cycle-forming edge in dag flow", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 520, 210));
    await runEffect(() => createTextNode(page, canvas, 760, 230));
    await runEffect(() => createTextNode(page, canvas, 980, 260));
    await expect(page.getByText(/3 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 3) {
      throw new Error("expected three nodes for cycle rejection test");
    }

    await edgeClick(page, centers[0].x, centers[0].y);
    await edgeClick(page, centers[1].x, centers[1].y);
    await edgeClick(page, centers[2].x, centers[2].y);
    await expect(page.getByText(/2 edges/)).toBeVisible();

    await edgeClick(page, centers[2].x, centers[2].y);
    await edgeClick(page, centers[0].x, centers[0].y);
    await expect(page.getByText(/2 edges/)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
