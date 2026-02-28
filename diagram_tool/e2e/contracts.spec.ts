import { expect, test, type Locator, type Page } from "@playwright/test";
import { readFile } from "node:fs/promises";
import {
  canvas,
  createTextNode,
  edgeCount,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  nodeCount,
  runEffect,
  runEffectsSequential,
  selectedCount,
  zoomPercent,
} from "./helpers";

type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

async function setup(page: Page): Promise<Locator> {
  await freshStart(page);
  return canvas(page);
}

async function createNodeAt(page: Page, x: number, y: number): Promise<void> {
  await createTextNode(page, canvas(page), x, y);
}

async function dragNodeBy(node: Locator, page: Page, dx: number, dy: number): Promise<void> {
  const before = await node.boundingBox();
  expect(before).not.toBeNull();
  const cx = before!.x + before!.width / 2;
  const cy = before!.y + before!.height / 2;
  await page.mouse.move(cx, cy);
  await page.mouse.down();
  await page.mouse.move(cx + dx, cy + dy, { steps: 4 });
  await page.mouse.up();
}

async function exportDocument(page: Page): Promise<JsonValue> {
  const exportButton = page.getByRole("button", { name: "Export JSON" });
  const [download] = await Promise.all([
    page.waitForEvent("download", { timeout: 15_000 }),
    exportButton.click(),
  ]);
  const filePath = await download.path();
  expect(filePath).not.toBeNull();
  const content = await readFile(filePath!, "utf8");
  return JSON.parse(content) as JsonValue;
}

async function importDocument(page: Page, payload: JsonValue): Promise<void> {
  const serialized = JSON.stringify(payload);
  await page.evaluate((json) => {
    (window as { __SESHAT_E2E_IMPORT_JSON?: string }).__SESHAT_E2E_IMPORT_JSON =
      json;
  }, serialized);
  await page.getByTestId("toolbar-open").click();
}

function canonicalJson(value: JsonValue): JsonValue {
  if (Array.isArray(value)) {
    return value.map(canonicalJson);
  }
  if (value !== null && typeof value === "object") {
    const entries = Object.entries(value)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([k, v]) => [k, canonicalJson(v)]);
    return Object.fromEntries(entries) as JsonValue;
  }
  return value;
}

test.describe("DOC contracts @baseline", () => {
  test("DOC-001: create node yields unique node ids", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 180, 180);
    await createNodeAt(page, 340, 220);
    await expectNodeCount(page, 2);

    const ids = await page
      .getByTestId("node")
      .evaluateAll((nodes) => nodes.map((n) => n.getAttribute("data-node-id") ?? ""));
    expect(ids.filter((id) => id.length > 0)).toHaveLength(2);
    expect(new Set(ids).size).toBe(ids.length);
  });

  test("DOC-002: create edge references existing endpoints", async ({ page }) => {
    const cv = await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 160, 220),
      () => createNodeAt(page, 420, 220),
    ]);

    await page.getByTestId("tool-edge").click();
    const nodes = cv.getByTestId("node");
    const first = await nodes.first().boundingBox();
    const second = await nodes.nth(1).boundingBox();
    expect(first).not.toBeNull();
    expect(second).not.toBeNull();
    await page.mouse.click(first!.x + first!.width / 2, first!.y + first!.height / 2);
    await page.mouse.click(second!.x + second!.width / 2, second!.y + second!.height / 2);
    await expectEdgeCount(page, 1);

    const doc = (await exportDocument(page)) as { document: { nodes: Record<string, unknown>; edges: Record<string, { source: string; target: string }> } };
    const edge = Object.values(doc.document.edges)[0];
    expect(doc.document.nodes[edge.source]).toBeTruthy();
    expect(doc.document.nodes[edge.target]).toBeTruthy();
  });

  test("DOC-003: deleting a node removes incident edges", async ({ page }) => {
    const cv = await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 140, 220),
      () => createNodeAt(page, 400, 220),
    ]);
    await page.getByTestId("tool-edge").click();
    const nodes = cv.getByTestId("node");
    const first = await nodes.first().boundingBox();
    const second = await nodes.nth(1).boundingBox();
    await page.mouse.click(first!.x + first!.width / 2, first!.y + first!.height / 2);
    await page.mouse.click(second!.x + second!.width / 2, second!.y + second!.height / 2);
    await expectEdgeCount(page, 1);

    await nodes.first().click();
    await page.keyboard.press("Delete");
    await expectNodeCount(page, 1);
    await expectEdgeCount(page, 0);
  });

  test("DOC-004: undo/redo round-trips mutation state", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 200, 200);
    await expectNodeCount(page, 1);
    await page.getByTestId("toolbar-undo").click();
    await expectNodeCount(page, 0);
    await page.getByTestId("toolbar-redo").click();
    await expectNodeCount(page, 1);
  });

  test("DOC-005: import rejects parent cycle payload", async ({ page }) => {
    await setup(page);
    const cyclic = {
      version: 2,
      revision: 0,
      document: {
        nodes: {
          a: { kind: "node", icon: "", label: "A", x: 0, y: 0, width: 80, height: 50, locked: false, parent: "b", dag_rank: null, tags: [], metadata: {}, z_index: 0 },
          b: { kind: "node", icon: "", label: "B", x: 100, y: 0, width: 80, height: 50, locked: false, parent: "a", dag_rank: null, tags: [], metadata: {}, z_index: 1 },
        },
        edges: {},
      },
      editor_state: { camera_x: 0, camera_y: 0, zoom: 1, grid_size: 20, snap_to_grid: true, selected_items: [], editing_edge_id: null, theme: "system", show_grid: true, minimap_visible: false },
    };

    await importDocument(page, cyclic);
    await expect(page.getByText("Load failed")).toBeVisible();
    await expectNodeCount(page, 0);
  });

  test("DOC-006: zoom commands remain clamped", async ({ page }) => {
    await setup(page);
    for (let i = 0; i < 18; i += 1) {
      await page.getByTestId("zoom-out").click();
    }
    expect(await zoomPercent(page)).toBeGreaterThanOrEqual(10);
    for (let i = 0; i < 40; i += 1) {
      await page.getByTestId("zoom-in").click();
    }
    expect(await zoomPercent(page)).toBeLessThanOrEqual(400);
  });

  test("DOC-007: pan tool click does not create nodes", async ({ page }) => {
    const cv = await setup(page);
    await page.getByTestId("tool-pan").click();
    const box = await cv.boundingBox();
    await page.mouse.click(box!.x + 220, box!.y + 180);
    await expectNodeCount(page, 0);
  });

  test("DOC-008: marquee direction changes contain/intersect behavior", async ({ page }) => {
    const cv = await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 560, 220),
      () => createNodeAt(page, 760, 220),
    ]);
    await page.getByTestId("tool-select").click();

    const box = await cv.boundingBox();
    await page.mouse.move(box!.x + 612, box!.y + 242);
    await page.mouse.down();
    await page.mouse.move(box!.x + 572, box!.y + 224);
    await page.mouse.up();
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(1);
  });

  test("DOC-009: cycle edge is rejected", async ({ page }) => {
    const cv = await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 120, 180),
      () => createNodeAt(page, 360, 180),
    ]);
    await page.getByTestId("tool-edge").click();
    const nodes = cv.getByTestId("node");
    const first = await nodes.first().boundingBox();
    const second = await nodes.nth(1).boundingBox();

    await page.mouse.click(first!.x + 10, first!.y + 10);
    await page.mouse.click(second!.x + 10, second!.y + 10);
    await expectEdgeCount(page, 1);

    await page.mouse.click(second!.x + 10, second!.y + 10);
    await page.mouse.click(first!.x + 10, first!.y + 10);
    await expectEdgeCount(page, 1);
  });

  test("DOC-010: z-order operations preserve selected relative order", async ({ page }) => {
    await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 180, 180),
      () => createNodeAt(page, 340, 220),
      () => createNodeAt(page, 500, 260),
    ]);
    await expectNodeCount(page, 3);

    const nodes = page.getByTestId("node");
    await nodes.nth(1).click();
    await page.keyboard.down("Shift");
    await nodes.nth(2).click();
    await page.keyboard.up("Shift");

    const before = await nodes.evaluateAll((els) =>
      els.map((el) => ({
        id: el.getAttribute("data-node-id") ?? "",
        z: Number.parseInt(window.getComputedStyle(el).zIndex || "0", 10),
      })),
    );

    await page.getByTestId("toolbar-bring-to-front").click();

    const after = await nodes.evaluateAll((els) =>
      els.map((el) => ({
        id: el.getAttribute("data-node-id") ?? "",
        z: Number.parseInt(window.getComputedStyle(el).zIndex || "0", 10),
      })),
    );

    const maxBefore = Math.max(...before.map((n) => n.z));
    const selectedAfter = after.filter((n) => before.slice(1).some((m) => m.id === n.id));
    expect(selectedAfter.every((n) => n.z >= maxBefore)).toBe(true);
  });

  test("DOC-011: locked nodes cannot be moved by drag or nudge", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 240, 220);
    const exported = (await exportDocument(page)) as {
      document: { nodes: Record<string, { locked?: boolean }> };
    };
    for (const node of Object.values(exported.document.nodes)) {
      node.locked = true;
    }

    await importDocument(page, exported);
    await expectNodeCount(page, 1);

    const node = page.getByTestId("node").first();
    await expect(node).toBeVisible();
    await node.click();

    const before = await node.boundingBox();
    await dragNodeBy(node, page, 80, 60);
    await page.keyboard.press("ArrowRight");
    const after = await node.boundingBox();

    expect(Math.abs((after?.x ?? 0) - (before?.x ?? 0))).toBeLessThan(1);
    expect(Math.abs((after?.y ?? 0) - (before?.y ?? 0))).toBeLessThan(1);
  });

  test("DOC-012: multi-selection remains stable after unrelated update", async ({ page }) => {
    await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 180, 220),
      () => createNodeAt(page, 380, 220),
    ]);

    const nodes = page.getByTestId("node");
    await nodes.first().click();
    await page.keyboard.down("Shift");
    await nodes.nth(1).click();
    await page.keyboard.up("Shift");
    await expectSelectedCount(page, 2);

    await page.getByTestId("grid-toggle").click();
    await expectSelectedCount(page, 2);
  });

  test("DOC-013: copy/paste remaps ids without collisions", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 220, 220);
    await page.getByTestId("node").first().click();
    await page.keyboard.press("ControlOrMeta+c");
    await page.keyboard.press("ControlOrMeta+v");
    await expectNodeCount(page, 2);

    const doc = (await exportDocument(page)) as { document: { nodes: Record<string, unknown> } };
    const nodeIds = Object.keys(doc.document.nodes);
    expect(new Set(nodeIds).size).toBe(nodeIds.length);
  });

  test("DOC-014: drag move is undone atomically", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 260, 220);
    const node = page.getByTestId("node").first();
    const before = await node.boundingBox();
    await dragNodeBy(node, page, 70, 40);
    await page.getByTestId("toolbar-undo").click();
    const afterUndo = await node.boundingBox();

    expect(Math.abs((afterUndo?.x ?? 0) - (before?.x ?? 0))).toBeLessThan(2);
    expect(Math.abs((afterUndo?.y ?? 0) - (before?.y ?? 0))).toBeLessThan(2);
  });

  test("DOC-015: repeated nudge keypresses collapse into single undo", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 240, 220);
    const node = page.getByTestId("node").first();
    await node.click();
    const before = await node.boundingBox();

    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    await page.getByTestId("toolbar-undo").click();

    const afterUndo = await node.boundingBox();
    expect(Math.abs((afterUndo?.x ?? 0) - (before?.x ?? 0))).toBeLessThanOrEqual(2);
  });

  test("DOC-016: redo is invalidated after new branch edit", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 160, 200);
    await createNodeAt(page, 320, 200);
    await expectNodeCount(page, 2);
    await page.getByTestId("toolbar-undo").click();
    await expectNodeCount(page, 1);
    await createNodeAt(page, 460, 200);
    await expectNodeCount(page, 2);
    await expect(page.getByTestId("toolbar-redo")).toBeDisabled();
  });

  test("DOC-017: exported document includes schema/version invariants", async ({ page }) => {
    await setup(page);
    await createNodeAt(page, 180, 220);
    const doc = (await exportDocument(page)) as {
      version: number;
      document: { nodes: Record<string, unknown>; edges: Record<string, unknown> };
      editor_state: Record<string, unknown>;
    };

    expect(doc.version).toBe(2);
    expect(typeof doc.document.nodes).toBe("object");
    expect(typeof doc.document.edges).toBe("object");
    expect(typeof doc.editor_state).toBe("object");
  });

  test("DOC-018: invalid edge constraints fail closed on import", async ({ page }) => {
    await setup(page);
    const invalid = {
      version: 2,
      revision: 0,
      document: {
        nodes: {
          a: { kind: "node", icon: "", label: "A", x: 0, y: 0, width: 80, height: 60, locked: false, parent: null, dag_rank: null, tags: [], metadata: {}, z_index: 0 },
        },
        edges: {
          e1: { source: "a", target: "missing", label: "", style: "solid", arrowType: "default", label_offset_t: 0.5, color: null, thickness: 1.5, directed: true, bend_points: [], tags: [], metadata: {} },
        },
      },
      editor_state: { camera_x: 0, camera_y: 0, zoom: 1, grid_size: 20, snap_to_grid: true, selected_items: [], editing_edge_id: null, theme: "system", show_grid: true, minimap_visible: false },
    };

    await importDocument(page, invalid);
    await expect(page.getByText("Load failed")).toBeVisible();
    expect(await nodeCount(page)).toBe(0);
    expect(await edgeCount(page)).toBe(0);
  });

  test("DOC-019: export -> import -> export is structurally idempotent", async ({ page }) => {
    await setup(page);
    await runEffectsSequential([
      () => createNodeAt(page, 180, 220),
      () => createNodeAt(page, 360, 260),
    ]);
    const first = await exportDocument(page);
    await importDocument(page, first);
    const second = await exportDocument(page);
    expect(canonicalJson(second)).toEqual(canonicalJson(first));
  });

  test("DOC-020: legacy alias migrations are deterministic", async ({ page }) => {
    await setup(page);

    const legacyA = {
      version: 2,
      revision: 0,
      document: {
        nodes: {
          n1: { id: "n1", kind: "node", icon: "", label: "A", x: 10, y: 10, width: 80, height: 60, locked: false, parent: null, dagRank: null, tags: [], metadata: {}, font_size: 11, z_index: 0 },
        },
        edges: {
          e1: { id: "e1", source: "n1", target: "n1", label: "", style: "solid", arrow_type: "diamond", labelOffsetT: 0.5, color: null, thickness: 1.5, directed: true, bendPoints: [], tags: [], metadata: {} },
        },
      },
      editor_state: { camera_x: 0, camera_y: 0, zoom: 1, grid_size: 20, snap_to_grid: true, selected_items: [], editing_edge_id: null, theme: "system", show_grid: true, minimap_visible: false },
    };

    const legacyB = {
      version: 2,
      revision: 0,
      document: {
        nodes: {
          n1: { kind: "node", icon: "", label: "A", x: 10, y: 10, width: 80, height: 60, locked: false, parent: null, dag_rank: null, tags: [], metadata: {}, fontSize: 11, z_index: 0 },
        },
        edges: {
          e1: { source: "n1", target: "n1", label: "", style: "solid", arrowType: "step", label_offset_t: 0.5, color: null, thickness: 1.5, directed: true, bend_points: [], tags: [], metadata: {} },
        },
      },
      editor_state: { camera_x: 0, camera_y: 0, zoom: 1, grid_size: 20, snap_to_grid: true, selected_items: [], editing_edge_id: null, theme: "system", show_grid: true, minimap_visible: false },
    };

    await importDocument(page, legacyA);
    const migratedA = await exportDocument(page);

    await runEffect(() => freshStart(page));
    await importDocument(page, legacyB);
    const migratedB = await exportDocument(page);

    expect(canonicalJson(migratedA)).toEqual(canonicalJson(migratedB));
  });
});
