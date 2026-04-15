//! Performance baseline benchmark for 3000 nodes.
//!
//! Establishes a performance baseline before optimization work. Measures:
//! - FPS during pan, zoom, select, drag
//! - Selection response time
//! - Undo/redo latency
//! - Save/load latency
//! - Memory usage
//!
//! Run with:  npx playwright test --project=perf-latency --grep "baseline-3000" --reporter=list

import { expect, test } from "@playwright/test";
import {
  canvas,
  freshStart,
  loadDocument,
  nodeCount,
  selectedCount,
  runEffect,
  waitForNoRebuildOverlay,
  trapPageErrors,
} from "./helpers";
import { summarizeLatency, summarizeFrames, attachPerfMetric } from "./perf.helpers";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TARGET_NODES = 3000;
const EDGE_COUNT = Math.floor(TARGET_NODES * 0.3); // ~30% of nodes have edges
const JANK_CUTOFF_MS = 16.67; // 60fps frame budget
const WARMUP_TRIALS = 2;
const MEASURE_TRIALS = 5;

// ---------------------------------------------------------------------------
// Document builder
// ---------------------------------------------------------------------------

function build3000NodeDoc() {
  const cols = Math.ceil(Math.sqrt(TARGET_NODES * 1.2));
  const nodesObj: Record<string, unknown> = {};
  const edgesObj: Record<string, unknown> = {};

  // Create nodes
  for (let i = 0; i < TARGET_NODES; i++) {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const r = Math.random();
    const kind = r < 0.7 ? "text" : r < 0.9 ? "node" : "subgraph";

    nodesObj[`node-${i}`] = {
      kind,
      icon: "",
      label: `N${i}`,
      x: 50 + col * 150,
      y: 50 + row * 100,
      width: 80 + Math.random() * 40,
      height: 30 + Math.random() * 20,
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

  // Create edges (only for first EDGE_COUNT nodes)
  for (let i = 0; i < EDGE_COUNT; i++) {
    const src = i;
    const tgt = (i + 1) % TARGET_NODES;
    edgesObj[`edge-${i}`] = {
      source: `node-${src}`,
      target: `node-${tgt}`,
      label: "",
      style: "solid",
      arrow_type: "straight",
      label_offset_t: 0.5,
      color: null,
      thickness: 1.5,
      directed: true,
      bend_points: [],
      tags: [],
      metadata: {},
      fontSize: null,
      source_port: i % 2 === 0 ? "right" : "bottom",
      target_port: i % 2 === 0 ? "left" : "top",
    };
  }

  return {
    version: 2,
    revision: 0,
    document: { nodes: nodesObj, edges: edgesObj },
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
// Timing helpers
// ---------------------------------------------------------------------------

async function timeMs(label: string, fn: () => Promise<void>): Promise<number> {
  const start = performance.now();
  await fn();
  const elapsed = performance.now() - start;
  console.log(`[BASELINE-3000] ${label}: ${elapsed.toFixed(1)} ms`);
  return elapsed;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("baseline-3000 — Performance baseline with 3000 nodes", () => {
  test.describe.configure({ mode: "serial" });

  test("1. document load time", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const nodeCount = TARGET_NODES;
    const edgeCount = EDGE_COUNT;

    const loadTimes: number[] = [];

    for (let i = 0; i < MEASURE_TRIALS; i++) {
      // Fresh start for each trial to get clean memory state
      if (i > 0) {
        await freshStart(page);
      }

      const t = await timeMs(`load ${nodeCount} nodes + ${edgeCount} edges`, async () => {
        const loaded = await loadDocument(page, doc);
        expect(loaded).toBe(true);
        await waitForNoRebuildOverlay(page);
      });
      loadTimes.push(t);
    }

    const summary = summarizeLatency(loadTimes);
    console.log(
      `[BASELINE-3000] Load summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms max=${summary.maxMs.toFixed(1)}ms`,
    );

    attachPerfMetric(testInfo, "document-load", {
      nodeCount,
      edgeCount,
      avgMs: Math.round(summary.avgMs * 100) / 100,
      p50Ms: Math.round(summary.p50Ms * 100) / 100,
      p95Ms: Math.round(summary.p95Ms * 100) / 100,
      maxMs: Math.round(summary.maxMs * 100) / 100,
    });

    expect(pageErrors).toHaveLength(0);
  });

  test("2. zoom-to-fit + DOM visibility", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Zoom out until all nodes are visible
    const zoomTimes: number[] = [];
    let domVisible = 0;

    for (let i = 0; i < 25; i++) {
      const t = await timeMs(`zoom out step ${i + 1}`, async () => {
        await page.keyboard.press("_");
        await page.waitForTimeout(100);
      });
      zoomTimes.push(t);

      domVisible = await page.evaluate(
        () => document.querySelectorAll('[data-testid="node"]').length,
      );
      console.log(`[BASELINE-3000] After zoom ${i + 1}: ${domVisible} / ${TARGET_NODES} DOM nodes`);

      if (domVisible >= TARGET_NODES * 0.95) {
        console.log(`[BASELINE-3000] 95%+ nodes visible at step ${i + 1}`);
        break;
      }
    }

    // Get memory baseline
    const memoryInfo = await page.evaluate(() => {
      if ("memory" in performance) {
        const mem = (performance as { memory?: { usedJSHeapSize: number; totalJSHeapSize: number } }).memory;
        return mem ? { usedMB: Math.round(mem.usedJSHeapSize / 1024 / 1024), totalMB: Math.round(mem.totalJSHeapSize / 1024 / 1024) } : null;
      }
      return null;
    });
    console.log(`[BASELINE-3000] Memory: ${JSON.stringify(memoryInfo)}`);

    const summary = summarizeLatency(zoomTimes);
    attachPerfMetric(testInfo, "zoom-to-fit", {
      targetNodes: TARGET_NODES,
      domVisibleNodes: domVisible,
      zoomSteps: zoomTimes.length,
      totalZoomMs: Math.round(summary.avgMs * zoomTimes.length),
      avgMsPerStep: Math.round(summary.avgMs * 100) / 100,
      memory: memoryInfo,
    });

    expect(domVisible).toBeGreaterThan(0);
    expect(pageErrors).toHaveLength(0);
  });

  test("3. FPS during sustained drag", async ({ page }, testInfo) => {
    test.setTimeout(300_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Zoom out until nodes are visible
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(200);
      const domVisible = await page.evaluate(
        () => document.querySelectorAll('[data-testid="node"]').length,
      );
      if (domVisible >= TARGET_NODES * 0.8) break;
    }

    // Get node position for dragging
    const warmupNode = page.getByTestId("node").first();
    const warmupBox = await runEffect(() => warmupNode.boundingBox());

    // Warm up
    if (warmupBox) {
      await page.mouse.move(warmupBox.x + 10, warmupBox.y + 10);
      await page.mouse.down();
      await page.mouse.move(warmupBox.x + 50, warmupBox.y + 50, { steps: 5 });
      await page.mouse.up();
      await waitForNoRebuildOverlay(page);
      await page.waitForTimeout(100);
    }

    // Measure drag performance using wall-clock timing per drag step
    const DRAG_STEPS = 100;
    const STEP_DISTANCE = 5;

    const fpsResults = await page.evaluate(async ({ steps, distance }) => {
      const node = document.querySelector('[data-testid="node"]');
      if (!node) return { error: "no node found" };

      const rect = node.getBoundingClientRect();
      const cx = rect.x + rect.width / 2;
      const cy = rect.y + rect.height / 2;

      // Use a flag to track if we're in a drag operation
      let dragActive = false;
      let frameCount = 0;
      const frameTimes: number[] = [];
      let lastFrameTime = performance.now();

      // Hook into rAF to measure frame times during drag
      const originalRAF = window.requestAnimationFrame;
      window.requestAnimationFrame = (callback: FrameRequestCallback) => {
        return originalRAF((time: number) => {
          if (dragActive) {
            const delta = time - lastFrameTime;
            if (delta > 0 && delta < 1000) { // Filter out anomalous values
              frameTimes.push(delta);
            }
            lastFrameTime = time;
            frameCount++;
          }
          callback(time);
        });
      };

      return new Promise((resolve) => {
        let step = 0;

        function startDrag() {
          dragActive = true;
          lastFrameTime = performance.now();
          // Mouse down
          node.dispatchEvent(new MouseEvent("mousedown", {
            bubbles: true, cancelable: true,
            clientX: cx, clientY: cy, button: 0,
          }));
          doDragStep();
        }

        function doDragStep() {
          if (step >= steps) {
            // Mouse up
            node.dispatchEvent(new MouseEvent("mouseup", {
              bubbles: true, cancelable: true,
              clientX: cx + step * distance, clientY: cy + step * distance, button: 0,
            }));
            dragActive = false;
            window.requestAnimationFrame = originalRAF;
            resolve();
            return;
          }

          // Mouse move
          node.dispatchEvent(new MouseEvent("mousemove", {
            bubbles: true, cancelable: true,
            clientX: cx + step * distance, clientY: cy + step * distance,
          }));
          step++;
          requestAnimationFrame(doDragStep);
        }

        setTimeout(startDrag, 100);
      }).then(() => ({
        totalFrames: frameCount,
        frameTimes,
        avgMs: frameTimes.length > 0 ? frameTimes.reduce((a, b) => a + b, 0) / frameTimes.length : 0,
        minMs: frameTimes.length > 0 ? Math.min(...frameTimes) : 0,
        maxMs: frameTimes.length > 0 ? Math.max(...frameTimes) : 0,
        fps: frameTimes.length > 0 ? 1000 / (frameTimes.reduce((a, b) => a + b, 0) / frameTimes.length) : 0,
        jankFrames: frameTimes.filter((t) => t > 16.67).length,
      }));
    }, { steps: DRAG_STEPS, distance: STEP_DISTANCE });

    console.log(`[BASELINE-3000] Drag FPS: avg=${fpsResults.avgMs.toFixed(2)}ms (${fpsResults.fps.toFixed(1)} fps), min=${fpsResults.minMs.toFixed(2)}ms, max=${fpsResults.maxMs.toFixed(2)}ms`);
    console.log(`[BASELINE-3000] Jank: ${fpsResults.jankFrames}/${fpsResults.frameTimes.length} frames > 16.67ms`);

    attachPerfMetric(testInfo, "drag-fps", {
      nodeCount: TARGET_NODES,
      totalFrames: fpsResults.totalFrames,
      avgFrameMs: Math.round(fpsResults.avgMs * 100) / 100,
      minFrameMs: Math.round(fpsResults.minMs * 100) / 100,
      maxFrameMs: Math.round(fpsResults.maxMs * 100) / 100,
      avgFps: Math.round(fpsResults.fps * 10) / 10,
      jankFrames: fpsResults.jankFrames,
      jankRatio: Math.round((fpsResults.jankFrames / fpsResults.totalFrames) * 1000) / 1000,
      dragDurationMs: Math.round(fpsResults.dragDurationMs),
    });

    expect(fpsResults.totalFrames).toBeGreaterThan(0);
    expect(pageErrors).toHaveLength(0);
  });

  test("4. selection response time", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Zoom out
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(100);
      const domVisible = await page.evaluate(
        () => document.querySelectorAll('[data-testid="node"]').length,
      );
      if (domVisible >= TARGET_NODES * 0.8) break;
    }

    // Use DOM events for reliable selection testing at scale
    const selectTimes: number[] = [];

    for (let i = 0; i < MEASURE_TRIALS * 2; i++) {
      const t = await timeMs(`select trial ${i + 1}`, async () => {
        const start = performance.now();
        const result = await page.evaluate(async (trial: number) => {
          function waitForFrame(): Promise<void> {
            return new Promise((resolve) => requestAnimationFrame(() => resolve()));
          }

          const node = document.querySelector('[data-testid="node"]');
          if (!node) return { error: "no node" };

          const rect = node.getBoundingClientRect();
          const cx = rect.x + rect.width / 2;
          const cy = rect.y + rect.height / 2;

          // Click to select
          node.dispatchEvent(new MouseEvent("mousedown", {
            bubbles: true, cancelable: true,
            clientX: cx, clientY: cy, button: 0,
          }));
          node.dispatchEvent(new MouseEvent("mouseup", {
            bubbles: true, cancelable: true,
            clientX: cx, clientY: cy, button: 0,
          }));

          await waitForFrame();
          await waitForFrame();

          return { success: true };
        }, i);

        if ("error" in result) {
          throw new Error(result.error);
        }

        return performance.now() - start;
      });
      selectTimes.push(t);

      // Deselect by clicking canvas
      await page.evaluate(async () => {
        const canvas = document.querySelector('[data-testid="canvas-root"]');
        if (!canvas) return;
        const rect = canvas.getBoundingClientRect();
        canvas.dispatchEvent(new MouseEvent("mousedown", {
          bubbles: true, cancelable: true,
          clientX: rect.x + 10, clientY: rect.y + 10, button: 0,
        }));
        canvas.dispatchEvent(new MouseEvent("mouseup", {
          bubbles: true, cancelable: true,
          clientX: rect.x + 10, clientY: rect.y + 10, button: 0,
        }));
      });
      await page.waitForTimeout(50);
    }

    const summary = summarizeLatency(selectTimes);
    console.log(
      `[BASELINE-3000] Selection response: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms max=${summary.maxMs.toFixed(1)}ms`,
    );

    attachPerfMetric(testInfo, "selection-response", {
      nodeCount: TARGET_NODES,
      avgMs: Math.round(summary.avgMs * 100) / 100,
      p50Ms: Math.round(summary.p50Ms * 100) / 100,
      p95Ms: Math.round(summary.p95Ms * 100) / 100,
      maxMs: Math.round(summary.maxMs * 100) / 100,
    });

    expect(pageErrors).toHaveLength(0);
  });

  test("5. undo/redo latency", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Zoom out
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(100);
      const domVisible = await page.evaluate(
        () => document.querySelectorAll('[data-testid="node"]').length,
      );
      if (domVisible >= TARGET_NODES * 0.8) break;
    }

    // Select a node using DOM events
    await page.evaluate(async () => {
      function waitForFrame(): Promise<void> {
        return new Promise((resolve) => requestAnimationFrame(() => resolve()));
      }

      const node = document.querySelector('[data-testid="node"]');
      if (!node) return;

      const rect = node.getBoundingClientRect();
      const cx = rect.x + rect.width / 2;
      const cy = rect.y + rect.height / 2;

      node.dispatchEvent(new MouseEvent("mousedown", {
        bubbles: true, cancelable: true,
        clientX: cx, clientY: cy, button: 0,
      }));
      node.dispatchEvent(new MouseEvent("mouseup", {
        bubbles: true, cancelable: true,
        clientX: cx, clientY: cy, button: 0,
      }));

      await waitForFrame();
      await waitForFrame();
    });

    await page.waitForTimeout(100);

    const undoTimes: number[] = [];
    const redoTimes: number[] = [];

    for (let i = 0; i < MEASURE_TRIALS; i++) {
      // Drag selected node using DOM events
      await page.evaluate(async () => {
        function waitForFrame(): Promise<void> {
          return new Promise((resolve) => requestAnimationFrame(() => resolve()));
        }

        const node = document.querySelector('[data-testid="node"]');
        if (!node) return;

        const rect = node.getBoundingClientRect();
        const cx = rect.x + rect.width / 2;
        const cy = rect.y + rect.height / 2;

        node.dispatchEvent(new MouseEvent("mousedown", {
          bubbles: true, cancelable: true,
          clientX: cx, clientY: cy, button: 0,
        }));

        for (let step = 0; step < 5; step++) {
          node.dispatchEvent(new MouseEvent("mousemove", {
            bubbles: true, cancelable: true,
            clientX: cx + step * 6, clientY: cy + step * 6,
          }));
        }

        node.dispatchEvent(new MouseEvent("mouseup", {
          bubbles: true, cancelable: true,
          clientX: cx + 30, clientY: cy + 30, button: 0,
        }));

        await waitForFrame();
        await waitForFrame();
      });

      await waitForNoRebuildOverlay(page);

      const tUndo = await timeMs(`undo trial ${i + 1}`, async () => {
        const start = performance.now();
        await page.locator('[data-testid="toolbar-undo"]').first().click();
        await waitForNoRebuildOverlay(page);
        return performance.now() - start;
      });
      undoTimes.push(tUndo);

      const tRedo = await timeMs(`redo trial ${i + 1}`, async () => {
        const start = performance.now();
        await page.locator('[data-testid="toolbar-redo"]').first().click();
        await waitForNoRebuildOverlay(page);
        return performance.now() - start;
      });
      redoTimes.push(tRedo);
    }

    const undoSummary = summarizeLatency(undoTimes);
    const redoSummary = summarizeLatency(redoTimes);

    console.log(
      `[BASELINE-3000] Undo: avg=${undoSummary.avgMs.toFixed(1)}ms p50=${undoSummary.p50Ms.toFixed(1)}ms p95=${undoSummary.p95Ms.toFixed(1)}ms`,
    );
    console.log(
      `[BASELINE-3000] Redo: avg=${redoSummary.avgMs.toFixed(1)}ms p50=${redoSummary.p50Ms.toFixed(1)}ms p95=${redoSummary.p95Ms.toFixed(1)}ms`,
    );

    attachPerfMetric(testInfo, "undo-redo-latency", {
      nodeCount: TARGET_NODES,
      undo: {
        avgMs: Math.round(undoSummary.avgMs * 100) / 100,
        p50Ms: Math.round(undoSummary.p50Ms * 100) / 100,
        p95Ms: Math.round(undoSummary.p95Ms * 100) / 100,
        maxMs: Math.round(undoSummary.maxMs * 100) / 100,
      },
      redo: {
        avgMs: Math.round(redoSummary.avgMs * 100) / 100,
        p50Ms: Math.round(redoSummary.p50Ms * 100) / 100,
        p95Ms: Math.round(redoSummary.p95Ms * 100) / 100,
        maxMs: Math.round(redoSummary.maxMs * 100) / 100,
      },
    });

    expect(pageErrors).toHaveLength(0);
  });

  test("6. pan performance", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Zoom out
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(100);
      const domVisible = await page.evaluate(
        () => document.querySelectorAll('[data-testid="node"]').length,
      );
      if (domVisible >= TARGET_NODES * 0.8) break;
    }

    const canvasArea = canvas(page);
    const box = await runEffect(() => canvasArea.boundingBox());
    if (!box) throw new Error("canvas not found");

    const panTimes: number[] = [];
    const PAN_STEPS = 20;
    const PAN_DISTANCE = 50;

    for (let i = 0; i < MEASURE_TRIALS; i++) {
      const t = await timeMs(`pan trial ${i + 1}`, async () => {
        const start = performance.now();

        // Perform pan by dragging on canvas
        const centerX = box.x + box.width / 2;
        const centerY = box.y + box.height / 2;

        await page.mouse.move(centerX, centerY);
        await page.mouse.down();

        for (let step = 0; step < PAN_STEPS; step++) {
          await page.mouse.move(
            centerX - step * PAN_DISTANCE,
            centerY - step * PAN_DISTANCE,
            { steps: 2 },
          );
        }

        await page.mouse.up();
        await waitForNoRebuildOverlay(page);

        return performance.now() - start;
      });
      panTimes.push(t);
    }

    const summary = summarizeLatency(panTimes);
    console.log(
      `[BASELINE-3000] Pan: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms max=${summary.maxMs.toFixed(1)}ms`,
    );

    attachPerfMetric(testInfo, "pan-performance", {
      nodeCount: TARGET_NODES,
      panSteps: PAN_STEPS,
      panDistance: PAN_STEPS * PAN_DISTANCE,
      avgMs: Math.round(summary.avgMs * 100) / 100,
      p50Ms: Math.round(summary.p50Ms * 100) / 100,
      p95Ms: Math.round(summary.p95Ms * 100) / 100,
      maxMs: Math.round(summary.maxMs * 100) / 100,
    });

    expect(pageErrors).toHaveLength(0);
  });

  test("7. zoom performance", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    const canvasArea = canvas(page);
    const box = await runEffect(() => canvasArea.boundingBox());
    if (!box) throw new Error("canvas not found");

    const centerX = box.x + box.width / 2;
    const centerY = box.y + box.height / 2;

    const zoomInTimes: number[] = [];
    const zoomOutTimes: number[] = [];
    const ZOOM_STEPS = 10;

    for (let i = 0; i < MEASURE_TRIALS; i++) {
      // Zoom in
      const tIn = await timeMs(`zoom-in trial ${i + 1}`, async () => {
        const start = performance.now();
        for (let step = 0; step < ZOOM_STEPS; step++) {
          await page.mouse.move(centerX, centerY);
          await page.mouse.wheel(0, -200);
          await page.waitForTimeout(30);
        }
        return performance.now() - start;
      });
      zoomInTimes.push(tIn);

      // Zoom out
      const tOut = await timeMs(`zoom-out trial ${i + 1}`, async () => {
        const start = performance.now();
        for (let step = 0; step < ZOOM_STEPS; step++) {
          await page.mouse.move(centerX, centerY);
          await page.mouse.wheel(0, 200);
          await page.waitForTimeout(30);
        }
        return performance.now() - start;
      });
      zoomOutTimes.push(tOut);
    }

    const inSummary = summarizeLatency(zoomInTimes);
    const outSummary = summarizeLatency(zoomOutTimes);

    console.log(
      `[BASELINE-3000] Zoom-in: avg=${inSummary.avgMs.toFixed(1)}ms p50=${inSummary.p50Ms.toFixed(1)}ms p95=${inSummary.p95Ms.toFixed(1)}ms`,
    );
    console.log(
      `[BASELINE-3000] Zoom-out: avg=${outSummary.avgMs.toFixed(1)}ms p50=${outSummary.p50Ms.toFixed(1)}ms p95=${outSummary.p95Ms.toFixed(1)}ms`,
    );

    attachPerfMetric(testInfo, "zoom-performance", {
      nodeCount: TARGET_NODES,
      zoomSteps: ZOOM_STEPS,
      zoomIn: {
        avgMs: Math.round(inSummary.avgMs * 100) / 100,
        p50Ms: Math.round(inSummary.p50Ms * 100) / 100,
        p95Ms: Math.round(inSummary.p95Ms * 100) / 100,
      },
      zoomOut: {
        avgMs: Math.round(outSummary.avgMs * 100) / 100,
        p50Ms: Math.round(outSummary.p50Ms * 100) / 100,
        p95Ms: Math.round(outSummary.p95Ms * 100) / 100,
      },
    });

    expect(pageErrors).toHaveLength(0);
  });

  test("8. memory usage snapshot", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const doc = build3000NodeDoc();
    const loaded = await loadDocument(page, doc);
    expect(loaded).toBe(true);
    await waitForNoRebuildOverlay(page);

    // Zoom out to show all nodes
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(100);
      const domVisible = await page.evaluate(
        () => document.querySelectorAll('[data-testid="node"]').length,
      );
      if (domVisible >= TARGET_NODES * 0.8) break;
    }

    // Force GC if available
    await page.evaluate(() => {
      if ("gc" in window && typeof (window as { gc?: () => void }).gc === "function") {
        (window as { gc?: () => void }).gc!();
      }
    });
    await page.waitForTimeout(500);

    const memorySnapshot = await page.evaluate(() => {
      const metrics: {
        domNodes: number;
        domEdges: number;
        totalDivs: number;
        memoryMB: { usedJSHeapSize?: number; totalJSHeapSize?: number; jsHeapSizeLimit?: number } | null;
      } = {
        domNodes: document.querySelectorAll('[data-testid="node"]').length,
        domEdges: document.querySelectorAll('[data-node-kind="edge"]').length,
        totalDivs: document.querySelectorAll("div").length,
        memoryMB: null,
      };

      if ("memory" in performance) {
        const mem = (performance as { memory?: { usedJSHeapSize: number; totalJSHeapSize: number; jsHeapSizeLimit: number } }).memory;
        if (mem) {
          metrics.memoryMB = {
            usedJSHeapSize: Math.round(mem.usedJSHeapSize / 1024 / 1024),
            totalJSHeapSize: Math.round(mem.totalJSHeapSize / 1024 / 1024),
            jsHeapSizeLimit: Math.round(mem.jsHeapSizeLimit / 1024 / 1024),
          };
        }
      }

      return metrics;
    });

    console.log(`[BASELINE-3000] Memory snapshot:`);
    console.log(`  DOM: ${memorySnapshot.domNodes} nodes, ${memorySnapshot.domEdges} edges, ${memorySnapshot.totalDivs} divs`);
    console.log(`  Heap: ${JSON.stringify(memorySnapshot.memoryMB)}`);

    attachPerfMetric(testInfo, "memory-snapshot", {
      targetNodes: TARGET_NODES,
      domNodes: memorySnapshot.domNodes,
      domEdges: memorySnapshot.domEdges,
      totalDivs: memorySnapshot.totalDivs,
      memoryMB: memorySnapshot.memoryMB,
    });

    expect(pageErrors).toHaveLength(0);
  });

  test("9. load latency after fresh page", async ({ page }, testInfo) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);

    const doc = build3000NodeDoc();

    // Measure load time from fresh page load
    const loadTimes: number[] = [];
    for (let i = 0; i < MEASURE_TRIALS; i++) {
      const t = await timeMs(`load trial ${i + 1}`, async () => {
        // Fresh page
        await freshStart(page);
        const start = performance.now();
        const loaded = await loadDocument(page, doc);
        expect(loaded).toBe(true);
        await waitForNoRebuildOverlay(page);
        return performance.now() - start;
      });
      loadTimes.push(t);
    }

    const loadSummary = summarizeLatency(loadTimes);

    console.log(
      `[BASELINE-3000] Load: avg=${loadSummary.avgMs.toFixed(1)}ms p50=${loadSummary.p50Ms.toFixed(1)}ms p95=${loadSummary.p95Ms.toFixed(1)}ms`,
    );

    attachPerfMetric(testInfo, "load-latency", {
      nodeCount: TARGET_NODES,
      edgeCount: EDGE_COUNT,
      load: {
        avgMs: Math.round(loadSummary.avgMs * 100) / 100,
        p50Ms: Math.round(loadSummary.p50Ms * 100) / 100,
        p95Ms: Math.round(loadSummary.p95Ms * 100) / 100,
        maxMs: Math.round(loadSummary.maxMs * 100) / 100,
      },
    });

    expect(pageErrors).toHaveLength(0);
  });
});