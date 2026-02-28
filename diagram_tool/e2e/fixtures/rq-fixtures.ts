import { expect, test as base, type Page } from "@playwright/test";
import { Effect } from "effect";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  edgeCount,
  nodeCount,
  runEffectsSequential,
  runEffect,
  selectedCount,
  waitForNoRebuildOverlay,
  waitForUiReady,
  zoomPercent,
} from "../helpers";

async function recoverFromRebuildOverlay(page: Page): Promise<void> {
  const rebuilding = page.getByRole("heading", { name: "Your app is being rebuilt." });
  const visible = await runEffect(() => rebuilding.isVisible().catch(() => false));
  if (!visible) {
    return;
  }
  await runEffectsSequential([
    () => page.reload({ waitUntil: "domcontentloaded", timeout: 5_000 }),
    () => waitForUiReady(page),
  ]);
}

type SceneName =
  | "scene_mixed_selection_v1"
  | "scene_nested_subgraph_v1"
  | "scene_stress_1k_v1";

type Fixtures = {
  seed: number;
  sceneId: SceneName;
  loadScene: (scene: SceneName) => Promise<void>;
  assertInvariants: () => Promise<void>;
  assertNoNaNOrInf: () => Promise<void>;
  assertSelectionBounds: () => Promise<void>;
  assertZoomBounds: () => Promise<void>;
  seededRng: (step: number) => number;
};

type SceneContract = Readonly<{
  checksum: string;
  nodeCount: number;
  edgeCount: number;
  requiredNodeIds: ReadonlyArray<string>;
}>;

type SceneJson = Readonly<{
  version: number;
  revision: number;
  document: {
    nodes: Record<string, { x: number; y: number; width: number; height: number }>;
    edges: Record<string, { source: string; target: string }>;
  };
}>;

const sceneContracts: Readonly<Record<SceneName, SceneContract>> = {
  scene_mixed_selection_v1: {
    checksum: "c35183ffbfbdb9a776ce0aa28b16bf9dcf6bb2c4403015c7e2273fbb32419d3e",
    nodeCount: 3,
    edgeCount: 1,
    requiredNodeIds: ["n1", "n2", "sg1"],
  },
  scene_nested_subgraph_v1: {
    checksum: "e2500f6a9f517943334ecbbf1c50086c2113b3f52e7f75d9b5b3e94287fec753",
    nodeCount: 4,
    edgeCount: 1,
    requiredNodeIds: ["outer", "inner", "t1", "t2"],
  },
  scene_stress_1k_v1: {
    checksum: "c4bda7ab343c358ba62b13de8d95da134a9d8eedf38741769c3e0e3a42072ca4",
    nodeCount: 12,
    edgeCount: 11,
    requiredNodeIds: ["n01", "n02", "n12"],
  },
};

function failSceneContract(sceneName: SceneName, message: string): never {
  throw new Error(`[rq-fixtures] Scene contract violation for ${sceneName}: ${message}`);
}

function parseAndValidateScenePayload(sceneName: SceneName, payload: string): void {
  const contract = sceneContracts[sceneName];
  const checksum = createHash("sha256").update(payload, "utf8").digest("hex");
  if (checksum !== contract.checksum) {
    failSceneContract(
      sceneName,
      `checksum mismatch (expected ${contract.checksum}, received ${checksum})`,
    );
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(payload) as unknown;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    failSceneContract(sceneName, `invalid JSON (${message})`);
  }

  const scene = parsed as Partial<SceneJson>;
  const nodes = scene.document?.nodes;
  const edges = scene.document?.edges;

  if (scene.version !== 2) {
    failSceneContract(sceneName, `expected version 2, received ${String(scene.version)}`);
  }
  if (scene.revision !== 0) {
    failSceneContract(sceneName, `expected revision 0, received ${String(scene.revision)}`);
  }
  if (!nodes || typeof nodes !== "object") {
    failSceneContract(sceneName, "missing document.nodes object");
  }
  if (!edges || typeof edges !== "object") {
    failSceneContract(sceneName, "missing document.edges object");
  }

  const nodeIds = Object.keys(nodes);
  const edgeIds = Object.keys(edges);

  if (nodeIds.length !== contract.nodeCount) {
    failSceneContract(sceneName, `expected ${contract.nodeCount} nodes, received ${nodeIds.length}`);
  }
  if (edgeIds.length !== contract.edgeCount) {
    failSceneContract(sceneName, `expected ${contract.edgeCount} edges, received ${edgeIds.length}`);
  }

  for (const requiredId of contract.requiredNodeIds) {
    if (!nodeIds.includes(requiredId)) {
      failSceneContract(sceneName, `missing required node id '${requiredId}'`);
    }
  }

  for (const nodeId of nodeIds) {
    const node = nodes[nodeId];
    if (!node) {
      failSceneContract(sceneName, `node '${nodeId}' is nullish`);
    }
    const numericFields = [
      ["x", node.x],
      ["y", node.y],
      ["width", node.width],
      ["height", node.height],
    ] as const;
    for (const [field, value] of numericFields) {
      if (!Number.isFinite(value)) {
        failSceneContract(sceneName, `node '${nodeId}' has non-finite ${field}`);
      }
    }
    if (node.width <= 0 || node.height <= 0) {
      failSceneContract(sceneName, `node '${nodeId}' has non-positive dimensions`);
    }
  }

  for (const edgeId of edgeIds) {
    const edge = edges[edgeId];
    if (!edge) {
      failSceneContract(sceneName, `edge '${edgeId}' is nullish`);
    }
    if (!nodeIds.includes(edge.source) || !nodeIds.includes(edge.target)) {
      failSceneContract(
        sceneName,
        `edge '${edgeId}' references missing node(s): ${edge.source} -> ${edge.target}`,
      );
    }
  }
}

function seededAt(seed: number, step: number): number {
  const rounds = Math.max(0, Math.floor(step));
  const next = Array.from({ length: rounds }).reduce<number>(
    (state: number) => (state * 1_664_525 + 1_013_904_223) >>> 0,
    seed >>> 0,
  );
  return next / 0xffff_ffff;
}

async function importScene(page: Page, sceneName: SceneName): Promise<void> {
  // Recover from any stale rebuild overlay before loading
  await recoverFromRebuildOverlay(page);

  const cwdPath = resolve(process.cwd(), "diagram_tool", "e2e", "scenes", `${sceneName}.json`);
  const fixtureRelativePath = resolve(__dirname, "..", "scenes", `${sceneName}.json`);
  const filePath = existsSync(cwdPath) ? cwdPath : fixtureRelativePath;
  const payload = readFileSync(filePath, "utf8");
  parseAndValidateScenePayload(sceneName, payload);
  const contract = sceneContracts[sceneName];
  await runEffectsSequential([
    () => waitForUiReady(page),
    () => page.waitForTimeout(1000),
    () => page.evaluate((jsonPayload) => {
      (window as { __SESHAT_E2E_IMPORT_JSON?: string }).__SESHAT_E2E_IMPORT_JSON = jsonPayload;
    }, payload),
    () => expect(page.getByTestId("toolbar-open")).toBeEnabled({ timeout: 15_000 }),
    () => page.getByTestId("toolbar-open").click(),
    () => page.waitForTimeout(2000),
    () => waitForNoRebuildOverlay(page),
    () => page.waitForTimeout(1000),
  ]);

  await expect.poll(() => nodeCount(page), { timeout: 15_000 }).toBe(contract.nodeCount);
  await expect.poll(() => edgeCount(page), { timeout: 15_000 }).toBe(contract.edgeCount);
}

export const test = base.extend<Fixtures>({
  seed: async ({ browserName }, use, testInfo) => {
    void browserName;
    const hash = [...testInfo.title].reduce((acc, char) => acc + char.charCodeAt(0), 0);
    await use(13_337 + hash);
  },
  sceneId: async ({ browserName }, use) => {
    void browserName;
    await use("scene_mixed_selection_v1");
  },
  loadScene: async ({ page }, use) => {
    await use(async (scene) => {
      await importScene(page, scene);
    });
  },
  seededRng: async ({ seed }, use) => {
    await use((step) => seededAt(seed, step));
  },
  assertInvariants: async ({ page }, use) => {
    await use(async () => {
      const [nodes, edges, selected] = await Effect.runPromise(
        Effect.all([
          Effect.tryPromise(() => nodeCount(page)),
          Effect.tryPromise(() => edgeCount(page)),
          Effect.tryPromise(() => selectedCount(page)),
        ]),
      );

      expect(nodes).toBeGreaterThanOrEqual(0);
      expect(edges).toBeGreaterThanOrEqual(0);
      expect(selected).toBeGreaterThanOrEqual(0);
      expect(selected).toBeLessThanOrEqual(nodes + edges);

      const allFrames = await Effect.runPromise(
        Effect.tryPromise(() =>
          page.getByTestId("node").evaluateAll((elements) =>
            elements.map((el) => {
              const rect = el.getBoundingClientRect();
              return {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
              };
            }),
          ),
        ),
      );

      for (const frame of allFrames) {
        expect(Number.isFinite(frame.x)).toBe(true);
        expect(Number.isFinite(frame.y)).toBe(true);
        expect(Number.isFinite(frame.width)).toBe(true);
        expect(Number.isFinite(frame.height)).toBe(true);
        expect(frame.width).toBeGreaterThan(0);
        expect(frame.height).toBeGreaterThan(0);
      }
    });
  },
  assertNoNaNOrInf: async ({ page }, use) => {
    await use(async () => {
      const zoom = await zoomPercent(page);
      expect(Number.isFinite(zoom)).toBe(true);

      const allNumeric = await Effect.runPromise(
        Effect.tryPromise(() =>
          page.evaluate(() => {
            const results: Array<{ selector: string; value: number; isFinite: boolean }> = [];

            const counters = document.querySelectorAll("[data-count]");
            counters.forEach((el) => {
              const count = el.getAttribute("data-count");
              if (count) {
                const num = Number.parseFloat(count);
                results.push({
                  selector: el.id || el.className || "counter",
                  value: num,
                  isFinite: Number.isFinite(num),
                });
              }
            });

            const transforms = document.querySelectorAll("[style*='transform']");
            transforms.forEach((el) => {
              const style = (el as HTMLElement).style.transform;
              const match = style.match(/[+-]?\d*\.?\d+/g);
              if (match) {
                match.forEach((numStr) => {
                  const num = Number.parseFloat(numStr);
                  results.push({
                    selector: "transform",
                    value: num,
                    isFinite: Number.isFinite(num),
                  });
                });
              }
            });

            return results;
          }),
        ),
      );

      expect(allNumeric.length).toBeGreaterThan(0);
      for (const item of allNumeric) {
        expect(item.isFinite).toBe(true);
      }
    });
  },
  assertSelectionBounds: async ({ page }, use) => {
    await use(async () => {
      const [nodes, edges, selected] = await Effect.runPromise(
        Effect.all([
          Effect.tryPromise(() => nodeCount(page)),
          Effect.tryPromise(() => edgeCount(page)),
          Effect.tryPromise(() => selectedCount(page)),
        ]),
      );

      expect(selected).toBeGreaterThanOrEqual(0);
      expect(selected).toBeLessThanOrEqual(nodes + edges);
    });
  },
  assertZoomBounds: async ({ page }, use) => {
    await use(async () => {
      const zoom = await zoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);
      expect(Number.isFinite(zoom)).toBe(true);
    });
  },
});

export { expect };
