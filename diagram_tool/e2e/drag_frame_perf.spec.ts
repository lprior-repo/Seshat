// @perf-latency
// Empirical drag-frame benchmark — measures rendering performance during
// a sustained drag across a 50-node synthetic diagram.
//
// Uses TWO orthogonal measurement methods:
//   1. MutationObserver: counts DOM attribute/child mutations per drag frame
//      (measures Dioxus reconciliation output — directly affected by our
//       peek()/Memo optimizations which reduce unnecessary re-renders)
//   2. Playwright page.mouse: triggers real event pipeline via CDP
//      (not synthetic dispatchEvent — ensures full Dioxus signal propagation)
//   3. Wall-clock timing per drag step via performance.now()
//
// Scientific method:
//   H₀: Treatment (post-opt) and control (pre-opt) have equal DOM mutations
//       and drag step durations.
//   H₁: Treatment has fewer DOM mutations and faster drag step durations.
//
// Run (treatment — current code at 0efe302):
//   npx playwright test --project=perf-latency --grep "drag-frame" --reporter=list
//
// Run (control — checkout 125393f first):
//   git stash && git checkout 125393f
//   npx playwright test --project=perf-latency --grep "drag-frame" --reporter=list
//   git checkout - && git stash pop

import { expect, test } from "@playwright/test";
import {
  canvas,
  freshStart,
  loadDocument,
  runEffect,
  waitForNoRebuildOverlay,
  trapPageErrors,
} from "./helpers";
import { attachPerfMetric, summarizeFrames } from "./perf.helpers";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TOTAL_NODES = 50;
const DRAG_STEPS = 50;
const DRAG_STEP_PX = 5; // 250px total drag distance
const TRIALS_PER_TEST = 5; // Repeats within a single test invocation
const JANK_CUTOFF_MS = 20; // >20ms = dropped frame at 60fps

// ---------------------------------------------------------------------------
// Synthetic document builder
// ---------------------------------------------------------------------------

function buildSyntheticDoc(nodeCount: number) {
  const cols = Math.ceil(Math.sqrt(nodeCount * 1.2));
  const nodesObj: Record<string, unknown> = {};
  for (let i = 0; i < nodeCount; i++) {
    const col = i % cols;
    const row = Math.floor(i / cols);
    nodesObj[`node-${i}`] = {
      kind: "text",
      icon: "",
      label: `N${i}`,
      x: 50 + col * 150,
      y: 50 + row * 100,
      width: 100,
      height: 24,
      fontSize: null,
      font_weight: null,
      lock_state: "unlocked",
      parent: null,
      dag_rank: null,
      tags: [],
      metadata: {},
      z_index: 0,
      style: "box",
      collapsed: null,
    };
  }
  return {
    version: 2,
    revision: 0,
    document: { nodes: nodesObj, edges: {} },
    editor_state: {
      camera_x: 0.0,
      camera_y: 0.0,
      zoom: 1.0,
      snap_to_grid: true,
      grid_size: 20,
      selected_items: [],
      show_grid: true,
      minimap_visible: false,
    },
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("drag-frame — rendering performance during sustained drag", () => {
  test.describe.configure({ mode: "serial" });

  test(`${TOTAL_NODES} nodes — DOM mutations and wall-clock during ${DRAG_STEPS}-step drag`, async ({
    page,
  }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    // Load synthetic document
    const doc = buildSyntheticDoc(TOTAL_NODES);
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Verify nodes rendered
    const domBefore = await page.evaluate(
      () => document.querySelectorAll('[data-testid="node"]').length,
    );
    console.log(
      `[DRAG-FRAME] ${TOTAL_NODES}n: ${domBefore} DOM nodes at default zoom`,
    );
    expect(domBefore).toBeGreaterThanOrEqual(TOTAL_NODES);

    // Warm-up: one throwaway drag to prime Dioxus reconciliation
    const warmupNode = page.getByTestId("node").first();
    const warmupBox = await runEffect(() => warmupNode.boundingBox());
    if (warmupBox) {
      await page.mouse.move(warmupBox.x + 10, warmupBox.y + 10);
      await page.mouse.down();
      await page.mouse.move(warmupBox.x + 30, warmupBox.y + 30, { steps: 5 });
      await page.mouse.up();
      await waitForNoRebuildOverlay(page);
      await page.waitForTimeout(200);
    }

    // Install MutationObserver inside the browser to count DOM changes
    await page.evaluate(() => {
      (window as Record<string, unknown>).__dragMutations = {
        attributes: 0,
        childList: 0,
        perFrame: [] as Array<{ attrs: number; children: number }>,
      };
      const data = (window as Record<string, unknown>).__dragMutations as {
        attributes: number;
        childList: number;
        perFrame: Array<{ attrs: number; children: number }>;
      };
      const observer = new MutationObserver((records) => {
        for (const record of records) {
          if (record.type === "attributes") data.attributes++;
          if (record.type === "childList") data.childList++;
        }
      });
      observer.observe(document.body, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["style", "class", "transform", "data-testid", "data-dragging"],
      });
      (window as Record<string, unknown>).__dragObserver = observer;
    });

    // Run N trials using real Playwright mouse events (via CDP)
    const allStepTimes: number[] = [];
    const allMutationCounts: number[] = [];
    const trialResults: Array<{
      trial: number;
      avgStepMs: number;
      maxStepMs: number;
      totalMutations: number;
      totalSteps: number;
    }> = [];

    for (let trial = 0; trial < TRIALS_PER_TEST; trial++) {
      // Find first node
      const node = page.getByTestId("node").first();
      const box = await runEffect(() => node.boundingBox());
      if (!box) throw new Error("node not visible for drag");

      const cx = box.x + box.width / 2;
      const cy = box.y + box.height / 2;

      // Reset mutation counters
      await page.evaluate(() => {
        const data = (window as Record<string, unknown>).__dragMutations as {
          attributes: number;
          childList: number;
          perFrame: Array<{ attrs: number; children: number }>;
        };
        data.attributes = 0;
        data.childList = 0;
        data.perFrame = [];
      });

      // Perform drag using real Playwright mouse events
      // page.mouse.move with steps generates real mousemove events via CDP
      const stepTimes: number[] = [];

      await page.mouse.move(cx, cy);
      await page.mouse.down();

      const dragStart = performance.now();

      for (let step = 1; step <= DRAG_STEPS; step++) {
        const stepStart = performance.now();
        await page.mouse.move(cx + step * DRAG_STEP_PX, cy + step * DRAG_STEP_PX, {
          steps: 1, // One step per move to measure each individually
        });
        const stepEnd = performance.now();
        stepTimes.push(stepEnd - stepStart);
      }

      await page.mouse.up();
      await waitForNoRebuildOverlay(page);

      const dragEnd = performance.now();
      const totalDragMs = dragEnd - dragStart;

      // Collect mutation counts
      const mutations = await page.evaluate(() => {
        const data = (window as Record<string, unknown>).__dragMutations as {
          attributes: number;
          childList: number;
          perFrame: Array<{ attrs: number; children: number }>;
        };
        return { attributes: data.attributes, childList: data.childList };
      });

      const totalMutations = mutations.attributes + mutations.childList;
      const avgStepMs = stepTimes.reduce((a, b) => a + b, 0) / stepTimes.length;
      const maxStepMs = Math.max(...stepTimes);

      allStepTimes.push(...stepTimes);
      allMutationCounts.push(totalMutations);

      trialResults.push({
        trial: trial + 1,
        avgStepMs,
        maxStepMs,
        totalMutations,
        totalSteps: DRAG_STEPS,
      });

      console.log(
        `[DRAG-FRAME] Trial ${trial + 1}: ` +
          `avg_step=${avgStepMs.toFixed(1)}ms, max_step=${maxStepMs.toFixed(1)}ms, ` +
          `total_drag=${totalDragMs.toFixed(0)}ms, ` +
          `mutations(attrs=${mutations.attributes}, children=${mutations.childList}, total=${totalMutations})`,
      );

      // Brief settle between trials
      await page.waitForTimeout(200);
    }

    // Cleanup observer
    await page.evaluate(() => {
      const obs = (window as Record<string, unknown>).__dragObserver as MutationObserver;
      obs?.disconnect();
    });

    // Aggregate statistics
    const stepFrameSummary = summarizeFrames(allStepTimes, JANK_CUTOFF_MS);
    const sortedSteps = [...allStepTimes].sort((a, b) => a - b);
    const p50Idx = Math.max(0, Math.min(sortedSteps.length - 1, Math.ceil(sortedSteps.length * 0.5) - 1));
    const p50Ms = sortedSteps[p50Idx] ?? 0;

    const avgMutations = allMutationCounts.reduce((a, b) => a + b, 0) / allMutationCounts.length;

    console.log("\n[DRAG-FRAME] === AGGREGATE RESULTS ===");
    console.log(
      `[DRAG-FRAME] ${stepFrameSummary.totalFrames} drag steps across ${TRIALS_PER_TEST} trials`,
    );
    console.log(
      `[DRAG-FRAME] step time: avg=${stepFrameSummary.avgFrameMs.toFixed(2)}ms, ` +
        `p50=${p50Ms.toFixed(2)}ms, ` +
        `p95=${stepFrameSummary.p95FrameMs.toFixed(2)}ms, ` +
        `max=${stepFrameSummary.maxFrameMs.toFixed(2)}ms`,
    );
    console.log(
      `[DRAG-FRAME] jank: ${stepFrameSummary.jankFrames}/${stepFrameSummary.totalFrames} ` +
        `(${(stepFrameSummary.jankRatio * 100).toFixed(1)}% steps > ${JANK_CUTOFF_MS}ms)`,
    );
    console.log(
      `[DRAG-FRAME] mutations: avg=${avgMutations.toFixed(0)}/trial ` +
        `(min=${Math.min(...allMutationCounts)}, max=${Math.max(...allMutationCounts)})`,
    );

    // Attach to Playwright reporter
    attachPerfMetric(testInfo, "drag-frame-summary", {
      nodeCount: TOTAL_NODES,
      dragSteps: DRAG_STEPS,
      dragDistancePx: DRAG_STEPS * DRAG_STEP_PX,
      trials: TRIALS_PER_TEST,
      totalSteps: stepFrameSummary.totalFrames,
      avgStepMs: Math.round(stepFrameSummary.avgFrameMs * 100) / 100,
      p50StepMs: Math.round(p50Ms * 100) / 100,
      p95StepMs: Math.round(stepFrameSummary.p95FrameMs * 100) / 100,
      maxStepMs: Math.round(stepFrameSummary.maxFrameMs * 100) / 100,
      jankSteps: stepFrameSummary.jankFrames,
      jankRatio: Math.round(stepFrameSummary.jankRatio * 1000) / 1000,
      avgMutationsPerTrial: Math.round(avgMutations * 10) / 10,
      minMutations: Math.min(...allMutationCounts),
      maxMutations: Math.max(...allMutationCounts),
      trialBreakdown: trialResults,
    });

    // Sanity checks
    expect(stepFrameSummary.totalFrames).toBeGreaterThan(0);
    expect(pageErrors).toHaveLength(0);
  });
});
