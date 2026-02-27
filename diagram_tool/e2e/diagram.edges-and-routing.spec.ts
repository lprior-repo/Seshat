import { expect, test, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  edgeCount,
  nodeCenters,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForUiReady,
  zoomPercent,
} from "./helpers";

test.describe("diagram edges and routing", () => {
  async function edgeClick(page: Page, x: number, y: number) {
    await runEffectsSequential([
      () => page.mouse.move(x, y),
      () => page.mouse.down(),
      () => page.mouse.up(),
    ]);
  }

  async function resetZoom(page: Page) {
    await runEffect(() => page.getByTestId("zoom-reset").click());
    await expect.poll(() => zoomPercent(page)).toBe(100);
  }

  async function zoomInToAtLeast(page: Page, targetPercent: number) {
    for (let i = 0; i < 16; i += 1) {
      const current = await zoomPercent(page);
      if (current >= targetPercent) {
        return;
      }
      await runEffect(() => page.getByTestId("zoom-in").click());
    }
    throw new Error(`failed to reach zoom >= ${targetPercent}%`);
  }

  test("connects nodes with edge tool @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");

    await runEffectsSequential([
      () => createTextNode(page, canvas, 560, 210),
      () => createTextNode(page, canvas, 820, 330),
    ]);
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

  test("rejects cycle-forming edge in dag flow @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 210),
      () => createTextNode(page, canvas, 760, 230),
      () => createTextNode(page, canvas, 980, 260),
    ]);
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

  test("edge overlap hit-selection stays deterministic across undo/redo cycle @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 360, 320),
      () => createTextNode(page, canvas, 780, 320),
      () => createTextNode(page, canvas, 620, 160),
      () => createTextNode(page, canvas, 620, 480),
    ]);
    await expect(page.getByText(/4 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await edgeClick(page, 410, 332);
    await edgeClick(page, 830, 332);
    await edgeClick(page, 670, 172);
    await edgeClick(page, 670, 492);
    await expect(page.getByText(/2 edges/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const detectRemainingOrientation = async (): Promise<"horizontal" | "vertical"> => {
      await edgeClick(page, 670, 332);
      expect(await selectedCount(page)).toBe(1);

      await runEffect(() => page.keyboard.press("Delete"));
      expect(await edgeCount(page)).toBe(1);

      await edgeClick(page, 250, 120);
      await edgeClick(page, 520, 332);
      const horizontalHit = (await selectedCount(page)) === 1;

      await edgeClick(page, 250, 120);
      await edgeClick(page, 670, 230);
      const verticalHit = (await selectedCount(page)) === 1;

      expect(horizontalHit).not.toBe(verticalHit);
      return horizontalHit ? "horizontal" : "vertical";
    };

    const firstRemaining = await detectRemainingOrientation();
    await runEffect(() =>
      page.getByRole("button", { name: "Undo", exact: true }).click(),
    );
    expect(await edgeCount(page)).toBe(2);

    const secondRemaining = await detectRemainingOrientation();
    expect(secondRemaining).toBe(firstRemaining);
    expect(pageErrors).toHaveLength(0);
  });

  test("overlapping edge hit-selection is deterministic across repeated clicks @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 360, 320),
      () => createTextNode(page, canvas, 780, 320),
      () => createTextNode(page, canvas, 620, 160),
      () => createTextNode(page, canvas, 620, 480),
    ]);
    await expect(page.getByText(/4 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await edgeClick(page, 410, 332);
    await edgeClick(page, 830, 332);
    await edgeClick(page, 670, 172);
    await edgeClick(page, 670, 492);
    await expect(page.getByText(/2 edges/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const detectRemovedOrientation = async (): Promise<"horizontal" | "vertical"> => {
      await edgeClick(page, 670, 332);
      expect(await selectedCount(page)).toBe(1);

      await runEffect(() => page.keyboard.press("Delete"));
      expect(await edgeCount(page)).toBe(1);

      await edgeClick(page, 250, 120);
      await edgeClick(page, 520, 332);
      const horizontalRemains = (await selectedCount(page)) === 1;

      await edgeClick(page, 250, 120);
      await edgeClick(page, 670, 230);
      const verticalRemains = (await selectedCount(page)) === 1;

      expect(horizontalRemains).not.toBe(verticalRemains);
      return horizontalRemains ? "vertical" : "horizontal";
    };

    const firstRemoved = await detectRemovedOrientation();
    for (let i = 0; i < 2; i += 1) {
      await runEffect(() =>
        page.getByRole("button", { name: "Undo", exact: true }).click(),
      );
      expect(await edgeCount(page)).toBe(2);
      expect(await detectRemovedOrientation()).toBe(firstRemoved);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("thin vertical edge remains selectable across zoom levels @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 680, 160),
      () => createTextNode(page, canvas, 680, 520),
    ]);
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );

    const centers = await runEffect(() => nodeCenters(canvas));
    if (centers.length < 2) {
      throw new Error("expected at least two nodes for thin-edge zoom test");
    }
    const byY = [...centers].sort((a, b) => a.y - b.y);
    await edgeClick(page, byY[0].x, byY[0].y);
    await edgeClick(page, byY[1].x, byY[1].y);
    await expect(page.getByText(/1 edges/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const probeX = byY[0].x + 3;
    const probeY = (byY[0].y + byY[1].y) / 2;

    await resetZoom(page);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    await resetZoom(page);
    await runEffect(() => page.getByTestId("zoom-out").click());
    await edgeClick(page, 250, 120);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    await resetZoom(page);
    await zoomInToAtLeast(page, 200);
    await edgeClick(page, 250, 120);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    await resetZoom(page);
    await zoomInToAtLeast(page, 300);
    await edgeClick(page, 250, 120);
    await edgeClick(page, probeX, probeY);
    expect(await selectedCount(page)).toBe(1);

    expect(pageErrors).toHaveLength(0);
  });

  test("endpoint-near clicks keep selecting the same edge endpoint @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 260),
      () => createTextNode(page, canvas, 860, 260),
      () => createTextNode(page, canvas, 700, 460),
    ]);
    await expect(page.getByText(/3 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await edgeClick(page, 570, 272);
    await edgeClick(page, 910, 272);
    await edgeClick(page, 750, 472);
    await edgeClick(page, 910, 272);
    await expect(page.getByText(/2 edges/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    const detectRemoved = async (): Promise<"horizontal" | "diagonal"> => {
      await edgeClick(page, 852, 279);
      expect(await selectedCount(page)).toBe(1);

      await runEffect(() => page.keyboard.press("Delete"));
      expect(await edgeCount(page)).toBe(1);

      await edgeClick(page, 250, 120);
      await edgeClick(page, 710, 272);
      const horizontalRemains = (await selectedCount(page)) === 1;

      await edgeClick(page, 250, 120);
      await edgeClick(page, 850, 402);
      const diagonalRemains = (await selectedCount(page)) === 1;

      expect(horizontalRemains).not.toBe(diagonalRemains);
      return horizontalRemains ? "diagonal" : "horizontal";
    };

    const firstRemoved = await detectRemoved();
    for (let i = 0; i < 2; i += 1) {
      await runEffect(() =>
        page.getByRole("button", { name: "Undo", exact: true }).click(),
      );
      expect(await edgeCount(page)).toBe(2);
      expect(await detectRemoved()).toBe(firstRemoved);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("selects thin edge reliably near target-side endpoint @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const canvas = page.getByTestId("canvas-root");
    await runEffectsSequential([
      () => createTextNode(page, canvas, 520, 260),
      () => createTextNode(page, canvas, 860, 260),
    ]);
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Edge", exact: true }).click(),
    );
    await edgeClick(page, 570, 272);
    await edgeClick(page, 910, 272);
    await expect(page.getByText(/1 edges/)).toBeVisible();

    await runEffect(() =>
      page.getByRole("button", { name: "Select", exact: true }).click(),
    );

    await edgeClick(page, 852, 279);
    expect(await selectedCount(page)).toBe(1);

    await edgeClick(page, 250, 120);
    await edgeClick(page, 852, 279);
    expect(await selectedCount(page)).toBe(1);
    expect(pageErrors).toHaveLength(0);
  });
});
