import { test, expect } from "../fixtures/rq-fixtures";
import { runEffectsSequential, trapPageErrors } from "../helpers";
import { runTrace } from "../redqueen/harness";
import { traceForSeed } from "../redqueen/operators";

type SceneName = "scene_mixed_selection_v1" | "scene_nested_subgraph_v1" | "scene_stress_1k_v1";

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
  ...mk("RQ-A", 4, "scene_mixed_selection_v1", "Shell/Boot", "Application boot and shell stability"),
  ...mk("RQ-B", 4, "scene_mixed_selection_v1", "Tool Modes", "Tool mode switching resilience"),
  ...mk("RQ-C", 5, "scene_mixed_selection_v1", "Selection", "Selection semantics under stress"),
  ...mk("RQ-D", 5, "scene_mixed_selection_v1", "Transform", "Node transform and drag operations"),
  ...mk("RQ-E", 5, "scene_stress_1k_v1", "Viewport", "Zoom/pan/viewport/minimap stability"),
  ...mk("RQ-F", 4, "scene_nested_subgraph_v1", "Edges", "Edge creation and routing under DAG guards"),
  ...mk("RQ-G", 5, "scene_nested_subgraph_v1", "Subgraph", "Subgraph nesting and proportion constraints"),
  ...mk("RQ-H", 5, "scene_mixed_selection_v1", "History", "Undo/redo/clipboard timeline integrity"),
  ...mk("RQ-I", 4, "scene_mixed_selection_v1", "Panels", "Panel/theme/persistence UI stability"),
  ...mk("RQ-J", 4, "scene_mixed_selection_v1", "Keyboard", "Keyboard-only and focus safety"),
  ...mk("RQ-K", 4, "scene_stress_1k_v1", "Numeric", "Numeric stability/bounds/NaN prevention"),
  ...mk("RQ-N", 4, "scene_mixed_selection_v1", "Import/Export", "Import/export/validation contracts"),
];

test.describe("redqueen priority matrix wave1 @rq @rq-wave1", () => {
  test.describe.configure({ mode: "parallel" });

  for (const entry of cases) {
    test(`${entry.id} ${entry.description} @rq @rq-wave1 @seeded`, async ({
      page,
      seed,
      loadScene,
      assertInvariants,
      seededRng,
    }) => {
      const pageErrors = trapPageErrors(page);
      await runEffectsSequential([() => page.goto("/"), () => loadScene(entry.scene)]);

      const sampledSeed = Math.floor(seededRng(seed % 17) * 10_000);
      await runTrace(page, {
        sceneId: entry.scene,
        seed: sampledSeed,
        wave: 1,
        operators: traceForSeed(sampledSeed, 1),
        timestamp: new Date().toISOString(),
      });

      await assertInvariants();
      expect(pageErrors).toHaveLength(0);
    });
  }
});
