import { expect, test, type Page } from "@playwright/test";
import {
  expectEdgeCount,
  expectNodeCount,
  loadDocument,
  resetDocument,
  runEffect,
  trapPageErrors,
  waitForCleanState,
  waitForE2eReady,
  waitForNoRebuildOverlay,
  waitForUiReady,
} from "./helpers";

function twoNodeDocument(): Record<string, unknown> {
  return {
    version: 2,
    revision: 1,
    document: {
      nodes: {
        source: {
          kind: "text",
          icon: "",
          label: "Source",
          x: 220,
          y: 220,
          width: 120,
          height: 48,
          fontSize: null,
          font_weight: null,
          locked: false,
          parent: null,
          dag_rank: null,
          tags: [],
          metadata: {},
          z_index: 0,
          style: "box",
          collapsed: null,
        },
        target: {
          kind: "text",
          icon: "",
          label: "Target",
          x: 460,
          y: 220,
          width: 120,
          height: 48,
          fontSize: null,
          font_weight: null,
          locked: false,
          parent: null,
          dag_rank: null,
          tags: [],
          metadata: {},
          z_index: 0,
          style: "box",
          collapsed: null,
        },
      },
      edges: {},
    },
    editor_state: {
      camera_x: 0,
      camera_y: 0,
      zoom: 1,
      snap_to_grid: true,
      grid_size: 20,
      selected_items: [],
      show_grid: true,
      minimap_visible: false,
    },
  };
}

async function freshBasePathStart(page: Page) {
  await page.goto("/Seshat/", { waitUntil: "domcontentloaded" });
  await waitForUiReady(page);
  await waitForE2eReady(page);
  await resetDocument(page);
  await waitForCleanState(page);
}

test("renders a newly created connection arrow immediately @baseline", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(page, twoNodeDocument());
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);

  const source = page.locator('[data-testid="node"][data-node-id="source"]');
  const target = page.locator('[data-testid="node"][data-node-id="target"]');
  await expect(source).toBeVisible();
  await expect(target).toBeVisible();

  await runEffect(() => source.hover());
  await expect(page.getByTestId("connection-dot")).toHaveCount(4);

  const sourceBox = await runEffect(() => source.boundingBox());
  const targetBox = await runEffect(() => target.boundingBox());
  if (!sourceBox || !targetBox) {
    throw new Error("node boxes missing");
  }

  await page.mouse.move(sourceBox.x + sourceBox.width, sourceBox.y + sourceBox.height / 2);
  await page.mouse.down();
  await page.mouse.move(targetBox.x + 1, targetBox.y + targetBox.height / 2, { steps: 8 });
  await page.mouse.up();

  await expectEdgeCount(page, 1);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(1);
  expect(pageErrors).toHaveLength(0);
});
