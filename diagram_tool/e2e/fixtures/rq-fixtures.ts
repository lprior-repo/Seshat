import { expect, test as base, type Page } from "@playwright/test";
import { Effect } from "effect";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { edgeCount, nodeCount, runEffect, selectedCount, waitForUiReady } from "../helpers";

type SceneName =
  | "scene_mixed_selection_v1"
  | "scene_nested_subgraph_v1"
  | "scene_stress_1k_v1";

type Fixtures = {
  seed: number;
  sceneId: SceneName;
  loadScene: (scene: SceneName) => Promise<void>;
  assertInvariants: () => Promise<void>;
  seededRng: (step: number) => number;
};

function seededAt(seed: number, step: number): number {
  const rounds = Math.max(0, Math.floor(step));
  const next = Array.from({ length: rounds }).reduce(
    (state) => (state * 1_664_525 + 1_013_904_223) >>> 0,
    seed >>> 0,
  );
  return next / 0xffff_ffff;
}

async function importScene(page: Page, sceneName: SceneName): Promise<void> {
  const filePath = join(process.cwd(), "diagram_tool", "e2e", "scenes", `${sceneName}.json`);
  await Effect.runPromise(
    Effect.gen(function* () {
      const payload = readFileSync(filePath, "utf8");
      const chooserPromise = page.waitForEvent("filechooser");
      yield* Effect.tryPromise(() =>
        page.getByRole("button", { name: /Import JSON|Open/ }).click(),
      );
      const chooser = yield* Effect.tryPromise(() => chooserPromise);
      yield* Effect.tryPromise(() =>
        chooser.setFiles({
          name: `${sceneName}.json`,
          mimeType: "application/json",
          buffer: Buffer.from(payload, "utf8"),
        }),
      );
      yield* Effect.tryPromise(() => waitForUiReady(page));
    }),
  );
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
          page.getByTestId("diagram-node").evaluateAll((elements) =>
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
      }
    });
  },
});
