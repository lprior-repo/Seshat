import { test, expect } from "../fixtures/rq-fixtures";
import { runEffect, trapPageErrors } from "../helpers";
import { runTrace } from "../redqueen/harness";
import { traceForSeed } from "../redqueen/operators";

type CaseDef = Readonly<{ id: string; scene: "scene_nested_subgraph_v1" | "scene_stress_1k_v1" }>;

const mk = (prefix: string, count: number, scene: CaseDef["scene"]): ReadonlyArray<CaseDef> =>
  Array.from({ length: count }, (_, index) => ({
    id: `${prefix}${String(index + 1).padStart(2, "0")}`,
    scene,
  }));

const cases: ReadonlyArray<CaseDef> = [
  ...mk("RQ-L", 4, "scene_nested_subgraph_v1"),
  ...mk("RQ-M", 3, "scene_stress_1k_v1"),
];

test.describe("redqueen priority matrix wave3", () => {
  for (const entry of cases) {
    test(`${entry.id} escalated deterministic replay @rq-wave3`, async ({
      page,
      seed,
      loadScene,
      assertInvariants,
      seededRng,
    }) => {
      const pageErrors = trapPageErrors(page);
      await runEffect(() => page.goto("/"));
      await loadScene(entry.scene);
      const sampledSeedA = Math.floor(seededRng(seed % 23) * 10_000);
      const sampledSeedB = Math.floor(seededRng(seed % 29) * 10_000);
      await runTrace(page, {
        sceneId: entry.scene,
        seed: sampledSeedA,
        wave: 3,
        operators: traceForSeed(sampledSeedA, 3),
      });
      await runTrace(page, {
        sceneId: entry.scene,
        seed: sampledSeedB,
        wave: 3,
        operators: traceForSeed(sampledSeedB, 3),
      });
      await assertInvariants();
      expect(pageErrors).toHaveLength(0);
    });
  }
});
