import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForNoRebuildOverlay,
  waitForUiReady,
} from "./helpers";

async function getZoomLabel(page: Page): Promise<string> {
  const label = await runEffect(() =>
    page
      .locator("div")
      .filter({ hasText: /^\d+%$/ })
      .first()
      .textContent()
      .then((t) => t ?? "100%"),
  );
  return label.trim();
}

async function getZoomPercent(page: Page): Promise<number> {
  const label = await getZoomLabel(page);
  const match = label.match(/(\d+)%/);
  return match ? Number.parseInt(match[1], 10) : 100;
}

async function clickZoomButton(page: Page, label: "+" | "-") {
  const buttons = page.getByRole("button", { name: label, exact: true });
  const count = await runEffect(() => buttons.count());
  for (let i = 0; i < count; i += 1) {
    const candidate = buttons.nth(i);
    const visible = await runEffect(() => candidate.isVisible().catch(() => false));
    if (visible) {
      await runEffect(() => candidate.click({ timeout: 1_500 }));
      return;
    }
  }
}

async function performWheelZoom(page: Page, canvas: Locator, deltaY: number, ctrl = true) {
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) return;
  await runEffect(() =>
    page.mouse.move(box.x + box.width / 2, box.y + box.height / 2),
  );
  await runEffect(() =>
    page.evaluate(
      ({ x, y, deltaY: dY, ctrlKey }) => {
        const target = document.elementFromPoint(x, y);
        if (target) {
          target.dispatchEvent(
            new WheelEvent("wheel", {
              bubbles: true,
              cancelable: true,
              clientX: x,
              clientY: y,
              deltaY: dY,
              ctrlKey,
            }),
          );
        }
      },
      { x: box.x + box.width / 2, y: box.y + box.height / 2, deltaY, ctrlKey: ctrl },
    ),
  );
}

async function dragNodeBy(page: Page, canvas: Locator, dx: number, dy: number) {
  const nodes = canvas.getByTestId("diagram-node").first();
  const box = await runEffect(() => nodes.boundingBox());
  if (!box) return;
  await runEffect(() => page.mouse.move(box.x + box.width / 2, box.y + box.height / 2));
  await runEffect(() => page.mouse.down());
  await runEffect(() =>
    page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 5 }),
  );
  await runEffect(() => page.mouse.up());
}

async function getMinimapViewportRect(page: Page): Promise<{
  x: number;
  y: number;
  width: number;
  height: number;
} | null> {
  const minimap = page.locator("div").filter({ hasText: /^\d+%$/ }).first().locator("..");
  const rect = await runEffect(() =>
    minimap.locator("rect").nth(2).evaluate((el) => {
      const x = Number.parseFloat(el.getAttribute("x") ?? "NaN");
      const y = Number.parseFloat(el.getAttribute("y") ?? "NaN");
      const width = Number.parseFloat(el.getAttribute("width") ?? "NaN");
      const height = Number.parseFloat(el.getAttribute("height") ?? "NaN");
      return { x, y, width, height };
    }),
  );
  return rect;
}

async function enableMinimap(page: Page) {
  const miniBtn = page.getByRole("button", { name: "Mini", exact: true });
  const visible = await runEffect(() => miniBtn.isVisible().catch(() => false));
  if (!visible) {
    await runEffect(() => miniBtn.click());
    await runEffect(() => waitForNoRebuildOverlay(page));
  }
}

async function dragMinimapViewport(page: Page, dx: number, dy: number) {
  const minimap = page.locator("div").filter({ hasText: /^\d+%$/ }).first().locator("..");
  const viewportRect = minimap.locator("rect").nth(2);
  const box = await runEffect(() => viewportRect.boundingBox());
  if (!box) return;
  await runEffect(() => page.mouse.move(box.x + box.width / 2, box.y + box.height / 2));
  await runEffect(() => page.mouse.down());
  await runEffect(() =>
    page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 5 }),
  );
  await runEffect(() => page.mouse.up());
}

function isFiniteNumber(n: number): boolean {
  return Number.isFinite(n);
}

test.describe("diagram numeric stability", () => {
  test.describe.configure({ timeout: 90_000 });

  test("zoom_clamps_at_extremes_under_mixed_inputs", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");

    for (let burst = 0; burst < 3; burst += 1) {
      for (let i = 0; i < 8; i += 1) {
        await clickZoomButton(page, "+");
        await runEffect(() => waitForNoRebuildOverlay(page));
      }
      let zoom = await getZoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      await runEffect(() => page.keyboard.press("+"));
      await runEffect(() => page.keyboard.press("="));
      zoom = await getZoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      await performWheelZoom(page, canvas, -500, true);
      await performWheelZoom(page, canvas, -500, true);
      zoom = await getZoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      await createTextNode(page, canvas, 400 + burst * 50, 200 + burst * 30);
      await dragNodeBy(page, canvas, 30, 20);

      for (let i = 0; i < 8; i += 1) {
        await clickZoomButton(page, "-");
        await runEffect(() => waitForNoRebuildOverlay(page));
      }

      await runEffect(() => page.keyboard.press("-"));
      await runEffect(() => page.keyboard.press("0"));
      zoom = await getZoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      await performWheelZoom(page, canvas, 500, true);
      await performWheelZoom(page, canvas, 500, true);
      zoom = await getZoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      await runEffect(() => page.keyboard.press("Escape"));
    }

    await expect(canvas).toBeVisible();
    const nanErrors = pageErrors.filter((e) => e.includes("NaN") || e.includes("min > max"));
    expect(nanErrors).toHaveLength(0);
  });

  test("resize_handle_cross_over_keeps_dimensions_finite", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await createTextNode(page, canvas, 400, 250);

    const node = canvas.getByTestId("diagram-node").first();
    const initialBox = await runEffect(() => node.boundingBox());
    expect(initialBox).not.toBeNull();

    for (let iteration = 0; iteration < 5; iteration += 1) {
      const box = await runEffect(() => node.boundingBox());
      if (!box) break;

      await runEffect(() => page.mouse.move(box.x + box.width, box.y + box.height / 2));
      await runEffect(() => page.mouse.down());
      await runEffect(() =>
        page.mouse.move(box.x - 200, box.y + box.height / 2, { steps: 10 }),
      );
      await runEffect(() => page.mouse.up());

      const afterEast = await runEffect(() => node.boundingBox());
      expect(afterEast).not.toBeNull();
      if (afterEast) {
        expect(afterEast.width).toBeGreaterThanOrEqual(24);
        expect(Number.isFinite(afterEast.width)).toBe(true);
        expect(Number.isFinite(afterEast.height)).toBe(true);
      }

      const box2 = await runEffect(() => node.boundingBox());
      if (!box2) break;

      await runEffect(() => page.mouse.move(box2.x + box2.width / 2, box2.y + box2.height));
      await runEffect(() => page.mouse.down());
      await runEffect(() =>
        page.mouse.move(box2.x + box2.width / 2, box2.y - 150, { steps: 10 }),
      );
      await runEffect(() => page.mouse.up());

      const afterSouth = await runEffect(() => node.boundingBox());
      expect(afterSouth).not.toBeNull();
      if (afterSouth) {
        expect(afterSouth.height).toBeGreaterThanOrEqual(24);
        expect(Number.isFinite(afterSouth.width)).toBe(true);
        expect(Number.isFinite(afterSouth.height)).toBe(true);
      }
    }

    await expect(node).toBeVisible();
    const panicErrors = pageErrors.filter(
      (e) => e.includes("panic") || e.includes("NaN") || e.includes("Infinity"),
    );
    expect(panicErrors).toHaveLength(0);
  });

  test("multi_node_resize_near_minimum_never_produces_invalid_boxes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");

    await createTextNode(page, canvas, 300, 200);
    await createTextNode(page, canvas, 450, 200);
    await createTextNode(page, canvas, 600, 200);

    const nodes = canvas.getByTestId("diagram-node");
    const count = await runEffect(() => nodes.count());
    expect(count).toBeGreaterThanOrEqual(3);

    await runEffect(() => page.keyboard.press("Control+a"));
    await runEffect(() => waitForNoRebuildOverlay(page));

    for (let burst = 0; burst < 4; burst += 1) {
      for (let i = 0; i < 5; i += 1) {
        await clickZoomButton(page, "-");
        await runEffect(() => waitForNoRebuildOverlay(page));
      }

      const selectionBox = await runEffect(() =>
        canvas
          .getByTestId("selection-bounds")
          .first()
          .boundingBox()
          .catch(() => null),
      );

      if (selectionBox) {
        await runEffect(() =>
          page.mouse.move(
            selectionBox.x + selectionBox.width,
            selectionBox.y + selectionBox.height,
          ),
        );
        await runEffect(() => page.mouse.down());
        await runEffect(() =>
          page.mouse.move(
            selectionBox.x + selectionBox.width - 40,
            selectionBox.y + selectionBox.height - 30,
            { steps: 8 },
          ),
        );
        await runEffect(() => page.mouse.up());
      }

      await runEffect(() => waitForNoRebuildOverlay(page));
    }

    const allNodes = await runEffect(() =>
      nodes.evaluateAll((elements) =>
        elements.map((el) => {
          const rect = el.getBoundingClientRect();
          return { width: rect.width, height: rect.height };
        }),
      ),
    );

    for (const nodeRect of allNodes) {
      expect(isFiniteNumber(nodeRect.width)).toBe(true);
      expect(isFiniteNumber(nodeRect.height)).toBe(true);
      expect(nodeRect.width).toBeGreaterThanOrEqual(0);
      expect(nodeRect.height).toBeGreaterThanOrEqual(0);
    }

    await runEffect(() => page.keyboard.press("Escape"));
    const runtimeErrors = pageErrors.filter(
      (e) => e.includes("panic") || e.includes("invalid") || e.includes("NaN"),
    );
    expect(runtimeErrors).toHaveLength(0);
  });

  test("minimap_drag_at_zoom_extremes_keeps_viewport_rect_valid", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");

    await createTextNode(page, canvas, 300, 200);
    await createTextNode(page, canvas, 700, 400);

    await enableMinimap(page);

    for (let extreme = 0; extreme < 2; extreme += 1) {
      const targetZoom = extreme === 0 ? 400 : 10;
      const direction = extreme === 0 ? "+" : "-";

      for (let i = 0; i < 15; i += 1) {
        await clickZoomButton(page, direction as "+" | "-");
        await runEffect(() => waitForNoRebuildOverlay(page));
      }

      const zoom = await getZoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      const corners = [
        { dx: 30, dy: 20 },
        { dx: -30, dy: 20 },
        { dx: 30, dy: -20 },
        { dx: -30, dy: -20 },
      ];

      for (const corner of corners) {
        await dragMinimapViewport(page, corner.dx, corner.dy);
        await runEffect(() => waitForNoRebuildOverlay(page));

        const rect = await getMinimapViewportRect(page);
        expect(rect).not.toBeNull();
        if (rect) {
          expect(isFiniteNumber(rect.x)).toBe(true);
          expect(isFiniteNumber(rect.y)).toBe(true);
          expect(isFiniteNumber(rect.width)).toBe(true);
          expect(isFiniteNumber(rect.height)).toBe(true);
          expect(Number.isNaN(rect.x)).toBe(false);
          expect(Number.isNaN(rect.y)).toBe(false);
          expect(Number.isNaN(rect.width)).toBe(false);
          expect(Number.isNaN(rect.height)).toBe(false);
        }
      }
    }

    await createTextNode(page, canvas, 500, 300);
    await expect(canvas).toBeVisible();

    const nanErrors = pageErrors.filter(
      (e) =>
        e.includes("NaN") ||
        e.includes("Infinity") ||
        e.includes("is not finite") ||
        e.includes("invalid"),
    );
    expect(nanErrors).toHaveLength(0);
  });
});
