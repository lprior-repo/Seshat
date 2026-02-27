import { expect, test } from "@playwright/test";
import { Buffer } from "node:buffer";
import {
  clearCanvasOverlays,
  createTextNode,
  nodeCount,
  runEffect,
  trapPageErrors,
  waitForNoRebuildOverlay,
  waitForUiReady,
} from "./helpers";

test.describe("diagram panel persistence and resiliency", () => {
  test("panel toggles preserve canvas interactivity", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() =>
      expect(page.getByTestId("canvas-container")).toBeVisible({ timeout: 30_000 }),
    );
    await runEffect(() => waitForUiReady(page));

    const icons = page.getByRole("button", { name: "Icons", exact: true });
    const props = page.getByRole("button", { name: "Props", exact: true });
    const mini = page.getByRole("button", { name: "Mini", exact: true });
    const valid = page.getByRole("button", { name: "Valid", exact: true });

    for (let i = 0; i < 3; i += 1) {
      await runEffect(() => icons.click());
      await runEffect(() => props.click());
      await runEffect(() => mini.click());
      await runEffect(() => valid.click());
    }

    const canvas = page.getByTestId("canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 780, 320));
    expect(await nodeCount(page)).toBe(2);

    const textNode = canvas.getByText("Text", { exact: true }).first();
    const before = await runEffect(() => textNode.boundingBox());
    if (!before) {
      throw new Error("text node bounds missing before drag");
    }

    await runEffect(() => page.mouse.move(before.x + 8, before.y + 8));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(before.x + 52, before.y + 40));
    await runEffect(() => page.mouse.up());

    const after = await runEffect(() => textNode.boundingBox());
    if (!after) {
      throw new Error("text node bounds missing after drag");
    }

    expect(after.x).toBeGreaterThan(before.x + 14);
    expect(after.y).toBeGreaterThan(before.y + 14);
    expect(pageErrors).toHaveLength(0);
  });

  test("validation panel badge update path stays stable", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    await expect(page.getByText("Validation", { exact: true })).toBeVisible();

    const badge = page.getByTestId("validation-status");
    await expect(badge).toHaveText("Valid");

    const canvas = page.getByTestId("canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 780, 320));
    expect(await nodeCount(page)).toBe(2);

    await runEffect(() => page.getByRole("button", { name: "Edge", exact: true }).click());
    const textNodes = canvas.getByText("Text", { exact: true });
    const first = await runEffect(() => textNodes.first().boundingBox());
    const second = await runEffect(() => textNodes.nth(1).boundingBox());
    if (!first || !second) {
      throw new Error("text node bounds missing for edge creation");
    }
    await runEffect(() => page.mouse.move(first.x + first.width / 2, first.y + first.height / 2));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await runEffect(() => page.mouse.move(second.x + second.width / 2, second.y + second.height / 2));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await expect(page.getByTestId("edge-count")).toHaveText("1 edges");

    await runEffect(() => page.getByRole("button", { name: "Validate", exact: true }).click());
    if (
      !(await runEffect(() =>
        page.getByText("Validation", { exact: true }).isVisible().catch(() => false),
      ))
    ) {
      await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    }
    await expect(page.getByText("Validation", { exact: true })).toBeVisible();
    await expect(badge).toHaveText("Valid");

    await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    await expect(badge).toHaveText("Valid");
    expect(pageErrors).toHaveLength(0);
  });

  test("export buttons survive populated canvas without runtime errors", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-container");
    await runEffect(() => createTextNode(page, canvas, 520, 210));
    await runEffect(() => createTextNode(page, canvas, 740, 290));
    await runEffect(() => createTextNode(page, canvas, 900, 360));
    expect(await nodeCount(page)).toBe(3);

    await runEffect(() => page.getByRole("button", { name: "Export JSON", exact: true }).click());
    await runEffect(() => page.getByRole("button", { name: "Export SVG", exact: true }).click());
    await runEffect(() => page.getByRole("button", { name: "Export PNG", exact: true }).click());

    await expect(page.getByTestId("canvas-container")).toBeVisible();
    expect(await nodeCount(page)).toBe(3);
    await runEffect(() => waitForNoRebuildOverlay(page));
    expect(pageErrors).toHaveLength(0);
  });

  test("icon sidebar search and load-more remain sane", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));

    await expect(page.getByRole("heading", { name: "Diagram Icons" })).toBeVisible();
    const iconGridItems = page.locator(".icon-item");
    await expect(iconGridItems.first()).toBeVisible();

    const search = page.getByPlaceholder("Search icons...");
    await runEffect(() => search.fill("aws"));
    await expect(iconGridItems.first()).toBeVisible();

    await expect(
      page.getByRole("button", { name: "Load more", exact: true }),
    ).toHaveCount(0);

    await runEffect(() => search.fill(""));
    const initialVisibleCount = await runEffect(() => iconGridItems.count());

    const loadMore = page.getByRole("button", { name: "Load more", exact: true }).first();
    await expect(loadMore).toBeVisible();
    await runEffect(() => loadMore.click());
    await runEffect(() => waitForNoRebuildOverlay(page));

    await expect(iconGridItems.first()).toBeVisible();
    const afterLoadMoreCount = await runEffect(() => iconGridItems.count());
    expect(afterLoadMoreCount).toBeGreaterThanOrEqual(initialVisibleCount);

    await runEffect(() => search.fill("zzzzzz-no-match"));
    await expect(iconGridItems).toHaveCount(0);

    await runEffect(() => search.fill(""));
    await expect(iconGridItems.first()).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("malformed import does not mutate scene or consume undo history", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-container");
    await runEffect(() => createTextNode(page, canvas, 620, 260));
    expect(await nodeCount(page)).toBe(1);

    const node = page.getByTestId("diagram-node").first();
    const beforeDrag = await runEffect(() => node.boundingBox());
    if (!beforeDrag) {
      throw new Error("node bounds missing before drag");
    }

    await runEffect(() => node.click());
    await runEffect(() => page.mouse.move(beforeDrag.x + 8, beforeDrag.y + 8));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(beforeDrag.x + 56, beforeDrag.y + 44));
    await runEffect(() => page.mouse.up());

    const afterDrag = await runEffect(() => node.boundingBox());
    if (!afterDrag) {
      throw new Error("node bounds missing after drag");
    }
    expect(afterDrag.x).toBeGreaterThan(beforeDrag.x + 14);
    expect(afterDrag.y).toBeGreaterThan(beforeDrag.y + 14);

    const chooserPromise = page.waitForEvent("filechooser");
    await runEffect(() => page.getByTestId("toolbar-open").click());
    const chooser = await chooserPromise;
    await runEffect(() =>
      chooser.setFiles([
        {
          name: "broken.json",
          mimeType: "application/json",
          buffer: Buffer.from("{not valid json"),
        },
      ]),
    );

    await expect(page.getByText("Load failed", { exact: true })).toBeVisible();
    expect(await nodeCount(page)).toBe(1);

    const afterFailedImport = await runEffect(() => node.boundingBox());
    if (!afterFailedImport) {
      throw new Error("node bounds missing after failed import");
    }

    expect(Math.abs(afterFailedImport.x - afterDrag.x)).toBeLessThanOrEqual(1);
    expect(Math.abs(afterFailedImport.y - afterDrag.y)).toBeLessThanOrEqual(1);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());

    const afterUndo = await runEffect(() => node.boundingBox());
    if (!afterUndo) {
      throw new Error("node bounds missing after undo");
    }

    expect(Math.abs(afterUndo.x - beforeDrag.x)).toBeLessThanOrEqual(2);
    expect(Math.abs(afterUndo.y - beforeDrag.y)).toBeLessThanOrEqual(2);
    expect(pageErrors).toHaveLength(0);
  });

  test("schema-invalid import does not mutate scene or consume undo history", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-container");
    await runEffect(() => createTextNode(page, canvas, 600, 250));
    expect(await nodeCount(page)).toBe(1);

    const node = page.getByTestId("diagram-node").first();
    const beforeDrag = await runEffect(() => node.boundingBox());
    if (!beforeDrag) {
      throw new Error("node bounds missing before schema-invalid import test");
    }

    await runEffect(() => node.click());
    await runEffect(() => page.mouse.move(beforeDrag.x + 8, beforeDrag.y + 8));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(beforeDrag.x + 50, beforeDrag.y + 34));
    await runEffect(() => page.mouse.up());

    const afterDrag = await runEffect(() => node.boundingBox());
    if (!afterDrag) {
      throw new Error("node bounds missing after drag in schema-invalid test");
    }
    expect(afterDrag.x).toBeGreaterThan(beforeDrag.x + 14);
    expect(afterDrag.y).toBeGreaterThan(beforeDrag.y + 10);

    const chooserPromise = page.waitForEvent("filechooser");
    await runEffect(() => page.getByTestId("toolbar-open").click());
    const chooser = await chooserPromise;
    await runEffect(() =>
      chooser.setFiles([
        {
          name: "schema-invalid.json",
          mimeType: "application/json",
          buffer: Buffer.from(
            JSON.stringify({
              version: 2,
              revision: 0,
              document: {
                nodes: {},
                edges: {
                  e1: {
                    source: "missing-a",
                    target: "missing-b",
                  },
                },
              },
            }),
          ),
        },
      ]),
    );

    await expect(page.getByText("Load failed", { exact: true })).toBeVisible();
    expect(await nodeCount(page)).toBe(1);

    const afterFailedImport = await runEffect(() => node.boundingBox());
    if (!afterFailedImport) {
      throw new Error("node bounds missing after schema-invalid import");
    }

    expect(Math.abs(afterFailedImport.x - afterDrag.x)).toBeLessThanOrEqual(1);
    expect(Math.abs(afterFailedImport.y - afterDrag.y)).toBeLessThanOrEqual(1);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());

    const afterUndo = await runEffect(() => node.boundingBox());
    if (!afterUndo) {
      throw new Error("node bounds missing after undo in schema-invalid test");
    }

    expect(Math.abs(afterUndo.x - beforeDrag.x)).toBeLessThanOrEqual(2);
    expect(Math.abs(afterUndo.y - beforeDrag.y)).toBeLessThanOrEqual(2);
    expect(pageErrors).toHaveLength(0);
  });

  test("cancelled import picker does not mutate scene or history", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.getByTestId("canvas-container");
    await runEffect(() => createTextNode(page, canvas, 610, 255));
    expect(await nodeCount(page)).toBe(1);

    const node = page.getByTestId("diagram-node").first();
    const beforeDrag = await runEffect(() => node.boundingBox());
    if (!beforeDrag) {
      throw new Error("node bounds missing before cancel-import test");
    }

    await runEffect(() => node.click());
    await runEffect(() => page.mouse.move(beforeDrag.x + 8, beforeDrag.y + 8));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(beforeDrag.x + 42, beforeDrag.y + 30));
    await runEffect(() => page.mouse.up());

    const afterDrag = await runEffect(() => node.boundingBox());
    if (!afterDrag) {
      throw new Error("node bounds missing after drag in cancel-import test");
    }
    expect(afterDrag.x).toBeGreaterThan(beforeDrag.x + 12);
    expect(afterDrag.y).toBeGreaterThan(beforeDrag.y + 8);

    const chooserPromise = page.waitForEvent("filechooser");
    await runEffect(() => page.getByTestId("toolbar-open").click());
    const chooser = await chooserPromise;
    await runEffect(() => chooser.setFiles([]));

    await expect(page.getByText("Load failed", { exact: true })).toHaveCount(0);
    expect(await nodeCount(page)).toBe(1);

    const afterCancel = await runEffect(() => node.boundingBox());
    if (!afterCancel) {
      throw new Error("node bounds missing after cancelled import");
    }
    expect(Math.abs(afterCancel.x - afterDrag.x)).toBeLessThanOrEqual(1);
    expect(Math.abs(afterCancel.y - afterDrag.y)).toBeLessThanOrEqual(1);

    await runEffect(() => page.getByRole("button", { name: "Undo", exact: true }).click());
    const afterUndo = await runEffect(() => node.boundingBox());
    if (!afterUndo) {
      throw new Error("node bounds missing after undo in cancel-import test");
    }

    expect(Math.abs(afterUndo.x - beforeDrag.x)).toBeLessThanOrEqual(2);
    expect(Math.abs(afterUndo.y - beforeDrag.y)).toBeLessThanOrEqual(2);
    expect(pageErrors).toHaveLength(0);
  });
});
