import { expect, type Locator, type Page } from "@playwright/test";
import { test } from "../fixtures/rq-fixtures";
import {
  edgeCount,
  nodeCount,
  nodeFrameByLabel,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  zoomPercent,
} from "../helpers";
import { runTrace } from "../redqueen/harness";
import { traceForSeed } from "../redqueen/operators";

type SceneId = "scene_mixed_selection_v1" | "scene_nested_subgraph_v1" | "scene_stress_1k_v1";
type BoundingBox = { x: number; y: number; width: number; height: number };

function annotateSeed(seed: number, wave: 1 | 2 | 3, sceneId: SceneId): void {
  test.info().annotations.push({
    type: "seed",
    description: `seed=${seed};wave=${wave};scene=${sceneId}`,
  });
}

async function bootScene(
  page: Page,
  loadScene: (scene: SceneId) => Promise<void>,
  sceneId: SceneId,
) {
  // Force a full page reload to ensure clean state between tests
  await runEffectsSequential([
    () => page.reload({ waitUntil: "domcontentloaded" }),
    () => page.waitForTimeout(2000),
    () => loadScene(sceneId),
  ]);
}

async function requireBox(target: Locator): Promise<BoundingBox> {
  const box = await runEffect(() => target.boundingBox());
  if (!box) {
    throw new Error("missing bounding box");
  }
  return box;
}

async function dragBy(
  page: Page,
  target: Locator,
  dx: number,
  dy: number,
) {
  const box = await requireBox(target);
  const cx = box.x + box.width / 2;
  const cy = box.y + box.height / 2;
  await runEffectsSequential([
    () => page.mouse.move(cx, cy),
    () => page.mouse.down(),
    () => page.mouse.move(cx + dx, cy + dy, { steps: 8 }),
    () => page.mouse.up(),
  ]);
}

async function minimapViewportRect(page: Page): Promise<BoundingBox> {
  return runEffect(() =>
    page.getByTestId("minimap-viewport").boundingBox().then((box) => {
      if (!box) {
        throw new Error("minimap viewport bounds unavailable");
      }
      return box;
    }),
  );
}

test.describe("redqueen first-20 deterministic intents", () => {
  test("baseline scene loads with finite counters @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    expect(await nodeCount(page)).toBeGreaterThan(0);
    expect(await edgeCount(page)).toBeGreaterThan(0);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline selection picks two nodes deterministically @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const nodes = page.getByTestId("node");
    await runEffectsSequential([
      () => nodes.nth(0).click(),
      () => page.keyboard.down("Shift"),
      () => nodes.nth(1).click(),
      () => page.keyboard.up("Shift"),
    ]);
    expect(await selectedCount(page)).toBe(2);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline drag applies stable positive delta @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const node = page.getByTestId("node").first();
    const before = await requireBox(node);
    await dragBy(page, node, 56, 38);
    const after = await requireBox(node);
    expect(after.x).toBeGreaterThan(before.x + 10);
    expect(after.y).toBeGreaterThan(before.y + 10);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline undo redo around drag restores then reapplies @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const node = page.getByTestId("node").first();
    const before = await requireBox(node);
    await dragBy(page, node, 64, 40);
    const moved = await requireBox(node);
    await runEffectsSequential([
      () => page.getByTestId("toolbar-undo").click(),
    ]);
    const undone = await requireBox(node);
    await runEffectsSequential([
      () => page.getByTestId("toolbar-redo").click(),
    ]);
    const redone = await requireBox(node);
    expect(moved.x).toBeGreaterThan(before.x + 10);
    expect(Math.abs(undone.x - before.x)).toBeLessThanOrEqual(2);
    expect(Math.abs(redone.x - moved.x)).toBeLessThanOrEqual(2);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline copy paste duplicates selected nodes @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const before = await nodeCount(page);
    const nodes = page.getByTestId("node");
    await runEffectsSequential([
      () => nodes.nth(0).click(),
      () => page.keyboard.down("Shift"),
      () => nodes.nth(1).click(),
      () => page.keyboard.up("Shift"),
      () => page.keyboard.press("ControlOrMeta+c"),
      () => page.keyboard.press("ControlOrMeta+v"),
    ]);
    expect(await nodeCount(page)).toBeGreaterThanOrEqual(before + 2);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline delete operation keeps scene valid @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const before = await nodeCount(page);
    await runEffectsSequential([
      () => page.getByTestId("node").first().click(),
      () => page.getByTestId("toolbar-delete").click(),
    ]);
    expect(await nodeCount(page)).toBeLessThanOrEqual(before - 1);
    await assertInvariants();
    await expect(page.getByTestId("validation-status")).toHaveText("Valid");
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline nested resize keeps inner outer ratios bounded @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_nested_subgraph_v1");
    const outerBefore = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerBefore = await nodeFrameByLabel(page, "Subgraph", 1);
    const ratioBefore = innerBefore.width / outerBefore.width;
    await runEffectsSequential([() => page.keyboard.press("ControlOrMeta+a")]);
    const handle = page.getByTestId("resize-handle-se");
    await dragBy(page, handle, 120, 90);
    const outerAfter = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerAfter = await nodeFrameByLabel(page, "Subgraph", 1);
    const ratioAfter = innerAfter.width / outerAfter.width;
    expect(Math.abs(ratioAfter - ratioBefore)).toBeLessThanOrEqual(0.25);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline nested minimum clamp prevents negative dimensions @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_nested_subgraph_v1");
    await runEffectsSequential([() => page.keyboard.press("ControlOrMeta+a")]);
    const handle = page.getByTestId("resize-handle-se");
    await dragBy(page, handle, -420, -260);
    const outer = await nodeFrameByLabel(page, "Subgraph", 0);
    const inner = await nodeFrameByLabel(page, "Subgraph", 1);
    expect(outer.width).toBeGreaterThan(0);
    expect(outer.height).toBeGreaterThan(0);
    expect(inner.width).toBeGreaterThan(0);
    expect(inner.height).toBeGreaterThan(0);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline rubberband selects multiple nodes in nested scene @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_nested_subgraph_v1");
    const canvas = page.getByTestId("canvas-root");
    const box = await requireBox(canvas);
    await runEffectsSequential([
      () => page.getByTestId("tool-select").click(),
      () => page.mouse.move(box.x + 130, box.y + 130),
      () => page.mouse.down(),
      () => page.mouse.move(box.x + 720, box.y + 500, { steps: 10 }),
      () => page.mouse.up(),
    ]);
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(2);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline dag boundary edge attempt remains valid @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const before = await edgeCount(page);
    const node = page.getByTestId("node").first();
    await runEffectsSequential([
      () => page.getByTestId("tool-edge").click(),
      () => node.click(),
      () => node.click(),
    ]);
    const after = await edgeCount(page);
    expect(after).toBeGreaterThanOrEqual(before);
    expect(after).toBeLessThanOrEqual(before + 1);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline minimap viewport values remain finite @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    await runEffectsSequential([
      () => page.getByTestId("panel-mini-toggle").click(),
      () => page.getByTestId("panel-mini-toggle").click(),
    ]);
    const rect = await minimapViewportRect(page);
    expect(Number.isFinite(rect.x)).toBe(true);
    expect(Number.isFinite(rect.y)).toBe(true);
    expect(Number.isFinite(rect.width)).toBe(true);
    expect(Number.isFinite(rect.height)).toBe(true);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("baseline panel toggle burst preserves interactivity @baseline", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    for (let i = 0; i < 4; i += 1) {
      await runEffectsSequential([
        () => page.getByTestId("panel-icons-toggle").click(),
        () => page.getByTestId("panel-props-toggle").click(),
        () => page.getByTestId("panel-mini-toggle").click(),
        () => page.getByTestId("panel-valid-toggle").click(),
      ]);
    }
    const node = page.getByTestId("node").first();
    const before = await requireBox(node);
    await dragBy(page, node, 40, 30);
    const after = await requireBox(node);
    expect(after.x).toBeGreaterThan(before.x + 8);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("seeded seed-1337 replay keeps zoom clamp in bounds @rq @seeded", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    annotateSeed(1337, 1, "scene_mixed_selection_v1");
    await runTrace(page, {
      sceneId: "scene_mixed_selection_v1",
      seed: 1337,
      wave: 1,
      operators: traceForSeed(1337, 1),
    });
    const zoom = await zoomPercent(page);
    expect(zoom).toBeGreaterThanOrEqual(10);
    expect(zoom).toBeLessThanOrEqual(400);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("seeded seed-4242 replay keeps resized geometry finite @rq @seeded", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_nested_subgraph_v1");
    annotateSeed(4242, 1, "scene_nested_subgraph_v1");
    await runTrace(page, {
      sceneId: "scene_nested_subgraph_v1",
      seed: 4242,
      wave: 1,
      operators: traceForSeed(4242, 1),
    });
    const frames = await runEffect(() =>
      page.getByTestId("node").evaluateAll((elements) =>
        elements.map((el) => {
          const rect = el.getBoundingClientRect();
          return { width: rect.width, height: rect.height, x: rect.x, y: rect.y };
        }),
      ),
    );
    for (const frame of frames) {
      expect(Number.isFinite(frame.x)).toBe(true);
      expect(Number.isFinite(frame.y)).toBe(true);
      expect(Number.isFinite(frame.width)).toBe(true);
      expect(Number.isFinite(frame.height)).toBe(true);
    }
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("seeded dual-seed replay remains bounded and deterministic @rq @seeded", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_stress_1k_v1");
    annotateSeed(1337, 3, "scene_stress_1k_v1");
    await runTrace(page, {
      sceneId: "scene_stress_1k_v1",
      seed: 1337,
      wave: 3,
      operators: traceForSeed(1337, 3),
    });
    annotateSeed(4242, 3, "scene_stress_1k_v1");
    await runTrace(page, {
      sceneId: "scene_stress_1k_v1",
      seed: 4242,
      wave: 3,
      operators: traceForSeed(4242, 3),
    });
    const zoom = await zoomPercent(page);
    expect(zoom).toBeGreaterThanOrEqual(10);
    expect(zoom).toBeLessThanOrEqual(400);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("stress selection consistency survives operator churn @rq @stress", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_stress_1k_v1");
    await runEffectsSequential([() => page.keyboard.press("ControlOrMeta+a")]);
    annotateSeed(1001, 1, "scene_stress_1k_v1");
    await runTrace(page, {
      sceneId: "scene_stress_1k_v1",
      seed: 1001,
      wave: 1,
      operators: traceForSeed(1001, 1),
    });
    const selected = await selectedCount(page);
    const total = (await nodeCount(page)) + (await edgeCount(page));
    expect(selected).toBeGreaterThanOrEqual(0);
    expect(selected).toBeLessThanOrEqual(total);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("stress history thrash preserves finite state @rq @stress", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    const node = page.getByTestId("node").first();
    await dragBy(page, node, 48, 30);
    for (let i = 0; i < 6; i += 1) {
      await runEffectsSequential([
        () => page.getByTestId("toolbar-undo").click(),
        () => page.getByTestId("toolbar-redo").click(),
      ]);
    }
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("stress jank budget smoke for deterministic trace @rq @stress", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_stress_1k_v1");
    const start = Date.now();
    annotateSeed(8080, 1, "scene_stress_1k_v1");
    await runTrace(page, {
      sceneId: "scene_stress_1k_v1",
      seed: 8080,
      wave: 1,
      operators: traceForSeed(8080, 1),
    });
    const elapsedMs = Date.now() - start;
    expect(elapsedMs).toBeLessThan(15_000);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("stress validation storm remains responsive @rq @stress", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_mixed_selection_v1");
    for (let i = 0; i < 8; i += 1) {
      await runEffectsSequential([
        () => page.getByTestId("panel-valid-toggle").click(),
        () => page.getByTestId("toolbar-validate").click(),
      ]);
    }
    await expect(page.getByTestId("validation-status")).toHaveText(/Valid|Invalid/);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });

  test("stress multi-seed replay chain keeps counts sane @rq @stress", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await bootScene(page, loadScene, "scene_nested_subgraph_v1");
    const seeds = [501, 777, 999];
    for (const seed of seeds) {
      annotateSeed(seed, 3, "scene_nested_subgraph_v1");
      await runTrace(page, {
        sceneId: "scene_nested_subgraph_v1",
        seed,
        wave: 3,
        operators: traceForSeed(seed, 3),
      });
    }
    expect(await nodeCount(page)).toBeGreaterThanOrEqual(2);
    expect(await edgeCount(page)).toBeGreaterThanOrEqual(0);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });
});
