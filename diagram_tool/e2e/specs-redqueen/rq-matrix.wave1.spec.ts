import { test, expect } from "../fixtures/rq-fixtures";
import { runEffect, trapPageErrors } from "../helpers";
import { runTrace } from "../redqueen/harness";
import { traceForSeed } from "../redqueen/operators";

type CaseDef = Readonly<{ id: string; scene: "scene_mixed_selection_v1" | "scene_nested_subgraph_v1" | "scene_stress_1k_v1" }>;

const mk = (prefix: string, count: number, scene: CaseDef["scene"]): ReadonlyArray<CaseDef> =>
  Array.from({ length: count }, (_, index) => ({
    id: `${prefix}${String(index + 1).padStart(2, "0")}`,
    scene,
  }));

const cases: ReadonlyArray<CaseDef> = [
  ...mk("RQ-A", 4, "scene_mixed_selection_v1"),
  ...mk("RQ-B", 4, "scene_mixed_selection_v1"),
  ...mk("RQ-C", 5, "scene_mixed_selection_v1"),
  ...mk("RQ-D", 5, "scene_mixed_selection_v1"),
  ...mk("RQ-E", 5, "scene_stress_1k_v1"),
  ...mk("RQ-F", 4, "scene_nested_subgraph_v1"),
  ...mk("RQ-G", 5, "scene_nested_subgraph_v1"),
  ...mk("RQ-H", 5, "scene_mixed_selection_v1"),
  ...mk("RQ-I", 4, "scene_mixed_selection_v1"),
  ...mk("RQ-J", 4, "scene_mixed_selection_v1"),
  ...mk("RQ-K", 4, "scene_stress_1k_v1"),
  ...mk("RQ-N", 4, "scene_mixed_selection_v1"),
];

test.describe("redqueen priority matrix wave1", () => {
  for (const entry of cases) {
    test(`${entry.id} deterministic seed replay @rq-wave1`, async ({
      page,
      seed,
      loadScene,
      assertInvariants,
      seededRng,
    }) => {
      const pageErrors = trapPageErrors(page);
      await runEffect(() => page.goto("/"));
      await loadScene(entry.scene);
      const sampledSeed = Math.floor(seededRng(seed % 17) * 10_000);
      await runTrace(page, {
        sceneId: entry.scene,
        seed: sampledSeed,
        wave: 1,
        operators: traceForSeed(sampledSeed, 1),
      });
      await assertInvariants();
      expect(pageErrors).toHaveLength(0);
    });
  }
});
