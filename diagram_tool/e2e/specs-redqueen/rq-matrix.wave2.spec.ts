import { test, expect } from "../fixtures/rq-fixtures";
import { freshStart, nodeCount, edgeCount, zoomPercent, trapPageErrors, waitForUiReady } from "../helpers";
import { runTrace } from "../redqueen/harness";
import { tracesForReplay } from "../redqueen/corpus-manager";

function annotateSeed(seed: number, wave: 1 | 2 | 3, sceneId: string): void {
  test.info().annotations.push({
    type: "seed",
    description: `seed=${seed};wave=${wave};scene=${sceneId}`,
  });
}

test.describe("redqueen wave2 replay gate @rq @rq-wave2", () => {
  test.describe.configure({ mode: "serial" });

  const knownSeeds: ReadonlyArray<{
    seed: number;
    sceneId: string;
    wave: 1 | 2 | 3;
    description: string;
  }> = [
    { seed: 1337, sceneId: "scene_mixed_selection_v1", wave: 1, description: "zoom clamp bounds" },
    { seed: 4242, sceneId: "scene_nested_subgraph_v1", wave: 1, description: "resized geometry finite" },
    { seed: 7777, sceneId: "scene_stress_1k_v1", wave: 3, description: "stress selection consistency" },
    { seed: 8080, sceneId: "scene_stress_1k_v1", wave: 1, description: "jank budget smoke" },
  ];

  for (const known of knownSeeds) {
    test(`replay seed-${known.seed} ${known.description} passes @rq @rq-wave2 @seeded`, async ({
      page,
      loadScene,
      assertInvariants,
    }) => {
      const pageErrors = trapPageErrors(page);
      await freshStart(page);
      await waitForUiReady(page);
      await loadScene(known.sceneId as "scene_mixed_selection_v1" | "scene_nested_subgraph_v1" | "scene_stress_1k_v1");

      const { traceForSeed } = await import("../redqueen/operators");
      annotateSeed(known.seed, known.wave, known.sceneId);
      await runTrace(page, {
        sceneId: known.sceneId,
        seed: known.seed,
        wave: known.wave,
        operators: traceForSeed(known.seed, known.wave),
        timestamp: new Date().toISOString(),
      });

      await assertInvariants();

      const zoom = await zoomPercent(page);
      expect(zoom).toBeGreaterThanOrEqual(10);
      expect(zoom).toBeLessThanOrEqual(400);

      expect(pageErrors).toHaveLength(0);
    });
  }

  test("all promoted corpus seeds replay green @rq @rq-wave2 @seeded", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const corpusTraces = tracesForReplay();

    if (corpusTraces.length === 0) {
      test.skip();
      return;
    }

    const pageErrors = trapPageErrors(page);

    for (const trace of corpusTraces) {
      annotateSeed(trace.seed, trace.wave, trace.sceneId);
      await freshStart(page);
      await waitForUiReady(page);
      await loadScene(trace.sceneId as "scene_mixed_selection_v1" | "scene_nested_subgraph_v1" | "scene_stress_1k_v1");

      await runTrace(page, trace);
      await assertInvariants();
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("baseline counters remain nonnegative after replay @rq @rq-wave2 @seeded", async ({
    page,
    loadScene,
    assertInvariants,
  }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await waitForUiReady(page);
    await loadScene("scene_mixed_selection_v1");

    const { traceForSeed } = await import("../redqueen/operators");
    for (const seed of [1001, 2002, 3003]) {
      annotateSeed(seed, 2, "scene_mixed_selection_v1");
      await runTrace(page, {
        sceneId: "scene_mixed_selection_v1",
        seed,
        wave: 2,
        operators: traceForSeed(seed, 2),
        timestamp: new Date().toISOString(),
      });
    }

    const nodes = await nodeCount(page);
    const edges = await edgeCount(page);

    expect(nodes).toBeGreaterThanOrEqual(0);
    expect(edges).toBeGreaterThanOrEqual(0);
    await assertInvariants();
    expect(pageErrors).toHaveLength(0);
  });
});
