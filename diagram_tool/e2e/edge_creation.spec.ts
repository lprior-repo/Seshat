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

function twoNodeDocument(
  revision = 1,
  edges: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    version: 2,
    revision,
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
      edges,
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

async function nodeBox(page: Page, id: string) {
  const node = page.locator(`[data-testid="node"][data-node-id="${id}"]`);
  await expect(node).toBeVisible();
  const box = await runEffect(() => node.boundingBox());
  if (!box) {
    throw new Error(`node box missing for ${id}`);
  }
  return { node, box };
}

async function dragFromSourceEdgeToTarget(page: Page) {
  const source = await nodeBox(page, "source");
  const target = await nodeBox(page, "target");
  await page.mouse.move(source.box.x + source.box.width - 2, source.box.y + 12);
  await page.mouse.down();
  await page.mouse.move(target.box.x + 1, target.box.y + target.box.height / 2, {
    steps: 8,
  });
  await page.mouse.up();
}

test("renders a newly created connection arrow immediately @baseline", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(page, twoNodeDocument());
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);
  await page.getByTestId("tool-edge").click();

  const source = await nodeBox(page, "source");
  await page.mouse.move(source.box.x + source.box.width - 2, source.box.y + 12);
  await expect(page.getByTestId("connection-edge-hit-zone")).toHaveCount(8);
  await dragFromSourceEdgeToTarget(page);

  await expectEdgeCount(page, 1);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(1);
  expect(pageErrors).toHaveLength(0);
});

test("select mode node-edge drag moves the node instead of starting an arrow @baseline", async ({
  page,
}) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(page, twoNodeDocument());
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);

  const { node, box } = await nodeBox(page, "source");
  await page.mouse.move(box.x + box.width - 2, box.y + 12);
  await expect(page.getByTestId("connection-dot")).toHaveCount(4);
  await expect(page.getByTestId("connection-edge-hit-zone")).toHaveCount(0);

  await page.mouse.down();
  await page.mouse.move(box.x + box.width + 48, box.y + 32, { steps: 8 });
  await page.mouse.up();

  await expectEdgeCount(page, 0);
  await expect
    .poll(async () => (await node.boundingBox())?.x ?? box.x)
    .toBeGreaterThan(box.x + 10);
  expect(pageErrors).toHaveLength(0);
});

test("duplicate and self-edge releases do not create extra arrows @baseline", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(
    page,
    twoNodeDocument(1, {
      existing: {
        source: "source",
        target: "target",
        directed: true,
      },
    }),
  );
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);
  await expectEdgeCount(page, 1);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(1);

  await page.getByTestId("tool-edge").click();
  await dragFromSourceEdgeToTarget(page);
  await expectEdgeCount(page, 1);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(1);

  await resetDocument(page);
  await waitForCleanState(page);
  const reloaded = await loadDocument(page, twoNodeDocument());
  expect(reloaded).toBe(true);
  await expectNodeCount(page, 2);
  await page.getByTestId("tool-edge").click();
  const source = await nodeBox(page, "source");
  await page.mouse.move(source.box.x + source.box.width - 2, source.box.y + 12);
  await page.mouse.down();
  await page.mouse.move(
    source.box.x + source.box.width / 2,
    source.box.y + source.box.height / 2,
    {
      steps: 6,
    },
  );
  await page.mouse.up();

  await expectEdgeCount(page, 0);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(0);
  expect(pageErrors).toHaveLength(0);
});

test("background release cancels arrow drawing without creating a path @baseline", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(page, twoNodeDocument());
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);
  await page.getByTestId("tool-edge").click();

  const source = await nodeBox(page, "source");
  await page.mouse.move(source.box.x + source.box.width - 2, source.box.y + 12);
  await page.mouse.down();
  await page.mouse.move(source.box.x + 360, source.box.y + 220, { steps: 8 });
  await page.mouse.up();

  await expectEdgeCount(page, 0);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(0);
  expect(pageErrors).toHaveLength(0);
});

test("right-click on a node edge does not start arrow drawing @baseline", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(page, twoNodeDocument());
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);
  await page.getByTestId("tool-edge").click();

  const source = await nodeBox(page, "source");
  const target = await nodeBox(page, "target");
  await page.mouse.move(source.box.x + source.box.width - 2, source.box.y + 12);
  await page.mouse.down({ button: "right" });
  await page.mouse.move(target.box.x + 1, target.box.y + target.box.height / 2, {
    steps: 8,
  });
  await page.mouse.up({ button: "right" });

  await expectEdgeCount(page, 0);
  await expect(page.locator('path[data-node-kind="edge"]')).toHaveCount(0);
  expect(pageErrors).toHaveLength(0);
});

test("renders an e2e-loaded document when revision matches reset state @baseline", async ({
  page,
}) => {
  const pageErrors = trapPageErrors(page);
  await freshBasePathStart(page);

  const loaded = await loadDocument(page, twoNodeDocument(0));
  expect(loaded).toBe(true);
  await waitForNoRebuildOverlay(page);
  await expectNodeCount(page, 2);
  await expect(page.locator('[data-testid="node"][data-node-id="source"]')).toBeVisible();
  await expect(page.locator('[data-testid="node"][data-node-id="target"]')).toBeVisible();
  expect(pageErrors).toHaveLength(0);
});
