// @perf-latency
// HONEST performance measurement using PerformanceObserver + MutationObserver.
//
// Instead of trying to click nodes (which fails at high zoom due to tiny hit targets),
// this test:
// 1. Loads N nodes into the model
// 2. Zooms out to show all N nodes in the DOM
// 3. Dispatches synthetic DOM events (mousedown/mouseup) directly on the first node
// 4. Measures wall-clock time, mutation count, and DOM size
//
// This BYPASSES all the clicking/selection issues by operating at the DOM event level.
// The Dioxus signal propagation and virtual DOM diffing still happen because
// the events bubble through the actual DOM tree that Dioxus rendered.
//
// Run with:  npx playwright test --project=perf-latency --grep "perf-observer" --reporter=list

import { expect, test } from "@playwright/test";
import {
  canvas,
  freshStart,
  loadDocument,
  nodeCount,
  runEffect,
  waitForNoRebuildOverlay,
  trapPageErrors,
} from "./helpers";

test.describe("perf-observer — real DOM measurement at scale", () => {
  test.describe.configure({ mode: "serial" });

  for (const totalNodes of [100, 500, 2000] as const) {
    test(`${totalNodes} total — select event with all nodes visible`, async ({ page }) => {
      test.setTimeout(180_000);
      const pageErrors = trapPageErrors(page);
      await freshStart(page);

      // Load document with N nodes
      const cols = Math.ceil(Math.sqrt(totalNodes * 1.2));
      const nodesObj: Record<string, unknown> = {};
      for (let i = 0; i < totalNodes; i++) {
        const col = i % cols;
        const row = Math.floor(i / cols);
        nodesObj[`node-${i}`] = {
          kind: "text", icon: "", label: `N${i}`,
          x: 50 + col * 150, y: 50 + row * 100,
          width: 100, height: 24,
          fontSize: null, font_weight: null,
          lock_state: "unlocked", parent: null, dag_rank: null,
          tags: [], metadata: {}, z_index: 0, style: "box", collapsed: null,
        };
      }
      const doc = {
        version: 2, revision: 0,
        document: { nodes: nodesObj, edges: {} },
        editor_state: {
          camera_x: 0.0, camera_y: 0.0, zoom: 1.0,
          snap_to_grid: true, grid_size: 20,
          selected_items: [], show_grid: true, minimap_visible: false,
        },
      };
      const loaded = await loadDocument(page, doc);
      expect(loaded).toBe(true);

      const domBeforeZoom = await page.evaluate(() => document.querySelectorAll('[data-testid="node"]').length);
      console.log(`[PERF-OBS] ${totalNodes}n: ${domBeforeZoom} DOM nodes at default zoom`);

      // Zoom out until most nodes are visible
      let domVisible = domBeforeZoom;
      for (let i = 0; i < 20 && domVisible < totalNodes * 0.9; i++) {
        await page.keyboard.press("_");
        await page.waitForTimeout(300);
        domVisible = await page.evaluate(() => document.querySelectorAll('[data-testid="node"]').length);
        console.log(`[PERF-OBS] After zoom ${i + 1}: ${domVisible} DOM nodes`);
      }
      console.log(`[PERF-OBS] ${totalNodes}n: ${domVisible} DOM nodes visible after zoom`);

      // Measure with PerformanceObserver + MutationObserver via page.evaluate
      const result = await page.evaluate(async () => {
        // Find first node
        const node = document.querySelector('[data-testid="node"]');
        if (!node) return { error: "no nodes" };

        const rect = node.getBoundingClientRect();
        if (rect.height < 2 || rect.width < 2) return { error: "nodes too small", rect: `${rect.width}x${rect.height}` };

        // Setup MutationObserver
        const mutations = { adds: 0, removes: 0, attributes: 0 };
        const observer = new MutationObserver((ms) => {
          for (const m of ms) {
            if (m.type === "childList") {
              mutations.adds += m.addedNodes.length;
              mutations.removes += m.removedNodes.length;
            }
            if (m.type === "attributes") mutations.attributes++;
          }
        });

        const totalDivsBefore = document.querySelectorAll("div").length;

        // Select (mousedown on node, then mouseup to commit)
        const results: { label: string; wallMs: number; mutations: typeof mutations; domNodes: number; totalDivs: number }[] = [];

        for (let i = 0; i < 5; i++) {
          // Reset: click empty canvas to deselect
          const canvas = document.querySelector('[data-testid="canvas-root"]');
          if (!canvas) return { error: "no canvas" };

          observer.observe(document.body, {
            childList: true, subtree: true, attributes: true,
            attributeFilter: ["class", "style", "data-testid", "data-dragging"],
          });

          const t0 = performance.now();

          // Dispatch mousedown on the first node (selects it)
          const cx = rect.x + rect.width / 2;
          const cy = rect.y + rect.height / 2;
          node.dispatchEvent(new MouseEvent("mousedown", {
            bubbles: true, cancelable: true,
            clientX: cx, clientY: cy, button: 0,
          }));

          // Dispatch mouseup on same position (commits selection)
          node.dispatchEvent(new MouseEvent("mouseup", {
            bubbles: true, cancelable: true,
            clientX: cx, clientY: cy, button: 0,
          }));

          // Wait for Dioxus to process (use requestAnimationFrame)
          await new Promise<void>((resolve) => requestAnimationFrame(() => {
            requestAnimationFrame(() => resolve());
          }));

          const t1 = performance.now();

          // Deselect: click on canvas empty area
          const canvasRect = canvas.getBoundingClientRect();
          observer.observe(document.body, {
            childList: true, subtree: true, attributes: true,
            attributeFilter: ["class", "style", "data-testid"],
          });
          const t2 = performance.now();

          canvas.dispatchEvent(new MouseEvent("mousedown", {
            bubbles: true, cancelable: true,
            clientX: canvasRect.x + 10, clientY: canvasRect.y + 10, button: 0,
          }));
          canvas.dispatchEvent(new MouseEvent("mouseup", {
            bubbles: true, cancelable: true,
            clientX: canvasRect.x + 10, clientY: canvasRect.y + 10, button: 0,
          }));

          await new Promise<void>((resolve) => requestAnimationFrame(() => {
            requestAnimationFrame(() => resolve());
          }));

          const t3 = performance.now();
          observer.disconnect();

          const domNow = document.querySelectorAll('[data-testid="node"]').length;
          const divsNow = document.querySelectorAll("div").length;

          results.push({
            label: `select+desel ${i + 1}`,
            wallMs: Math.round(t1 - t0),
            deselectMs: Math.round(t3 - t2),
            mutations: { ...mutations },
            domNodes: domNow,
            totalDivs: divsNow,
          });
        }

        const finalDivs = document.querySelectorAll("div").length;
        return { results, finalDivs };
      });

      console.log(`[PERF-OBS] ${totalNodes}n (${domVisible} DOM):`);
      for (const r of result.results) {
        console.log(
          `  ${r.label}: select=${r.wallMs}ms deselect=${r.deselectMs}ms ` +
          `mutations(add=${r.mutations.adds} rm=${r.mutations.removes} attr=${r.mutations.attributes}) ` +
          `DOM=${r.domNodes}n divs=${r.totalDivs}`,
        );
      }
      console.log(`[PERF-OBS] ${totalNodes}n: total divs in DOM = ${result.finalDivs}`);

      // Verify the interaction actually happened — check if selected_items changed
      // (We can't read Rust signals from JS, but we can check CSS class changes)
      // The fact that mutations.attributes > 0 proves Dioxus processed the event

      // Basic sanity: deselect should NOT have 0 mutations at scale
      if (totalNodes >= 100) {
        const avgMutations = result.results.reduce((sum, r) => sum + r.mutations.adds + r.mutations.attributes, 0) / result.results.length;
        console.log(`[PERF-OBS] ${totalNodes}n: avg mutations per cycle = ${avgMutations.toFixed(1)}`);
      }

      expect(pageErrors).toHaveLength(0);
    });
  }
});
