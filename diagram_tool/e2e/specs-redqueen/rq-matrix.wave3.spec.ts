import { test, expect } from "../fixtures/rq-fixtures";
import { runEffectsSequential, trapPageErrors } from "../helpers";
import { runTrace } from "../redqueen/harness";
import { traceForSeed } from "../redqueen/operators";

type SceneName = "scene_nested_subgraph_v1" | "scene_stress_1k_v1";

type CaseDef = Readonly<{
  id: string;
  scene: SceneName;
  category: string;
  description: string;
}>;

const mk = (
  prefix: string,
  count: number,
  scene: SceneName,
  category: string,
  description: string,
): ReadonlyArray<CaseDef> =>
  Array.from({ length: count }, (_, index) => ({
    id: `${prefix}${String(index + 1).padStart(2, "0")}`,
    scene,
    category,
    description: `${description} variant ${index + 1}`,
  }));

const cases: ReadonlyArray<CaseDef> = [
  ...mk("RQ-L", 4, "scene_nested_subgraph_v1", "Race/Chaos", "Race condition and chaos determinism"),
  ...mk("RQ-M", 3, "scene_stress_1k_v1", "Performance", "Performance budget and jank detection"),
];

test.describe("redqueen priority matrix wave3 @rq @rq-wave3", () => {
  test.describe.configure({ mode: "parallel" });

  for (const entry of cases) {
    test(`${entry.id} escalated ${entry.description} @rq @rq-wave3 @stress`, async ({
      page,
      seed,
      loadScene,
      assertInvariants,
      seededRng,
    }) => {
      const pageErrors = trapPageErrors(page);
      await runEffectsSequential([() => page.goto("/"), () => loadScene(entry.scene)]);

      const sampledSeedA = Math.floor(seededRng(seed % 23) * 10_000);
      const sampledSeedB = Math.floor(seededRng(seed % 29) * 10_000);

      await runTrace(page, {
        sceneId: entry.scene,
        seed: sampledSeedA,
        wave: 3,
        operators: traceForSeed(sampledSeedA, 3),
        timestamp: new Date().toISOString(),
      });

      await runTrace(page, {
        sceneId: entry.scene,
        seed: sampledSeedB,
        wave: 3,
        operators: traceForSeed(sampledSeedB, 3),
        timestamp: new Date().toISOString(),
      });

      await assertInvariants();
      expect(pageErrors).toHaveLength(0);
    });
  }
});
