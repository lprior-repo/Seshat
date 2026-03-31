// @perf-latency
// JS-side profiling to identify exactly where the 110ms is spent.
//
// Uses the Performance User Timing API (performance.mark/measure) to profile:
// 1. Event dispatch (JS event handler → Rust/WASM)
// 2. WASM execution (Rust signal write + re-render)
// 3. Dioxus reconciliation (VDOM diffing + mutation generation)
// 4. DOM mutation application (real DOM updates)
// 5. Browser style recalculation + layout
//
// Run with:  npx playwright test --project=perf-latency --grep "js-profile" --reporter=list

import { expect, test } from "@playwright/test";
import {
  canvas,
  freshStart,
  loadDocument,
  waitForNoRebuildOverlay,
  trapPageErrors,
} from "./helpers";

test.describe("js-profile — detailed JS-side breakdown", () => {
  test.describe.configure({ mode: "serial" });

  test("2000 nodes — timing breakdown of select/deselect", async ({ page }) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    // Load 2000-node document
    const cols = Math.ceil(Math.sqrt(2000 * 1.2));
    const nodesObj: Record<string, unknown> = {};
    for (let i = 0; i < 2000; i++) {
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

    // Zoom out to show all 2000 nodes
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(300);
      const domVisible = await page.evaluate(() => document.querySelectorAll('[data-testid="node"]').length);
      if (domVisible >= 1900) break;
    }

    // Run profiling with detailed User Timing marks
    const result = await page.evaluate(async () => {
      function waitForFrame(): Promise<void> {
        return new Promise((resolve) => requestAnimationFrame(() => resolve()));
      }

      async function measureInteraction(
        action: () => Promise<void>,
        label: string,
      ): Promise<void> {
        performance.mark(`${label}-start`);
        performance.mark(`${label}-dispatch-start`);
        await action();
        performance.mark(`${label}-dispatch-end`);
        performance.mark(`${label}-rAF1`);
        await waitForFrame();
        performance.mark(`${label}-rAF1-end`);
        performance.mark(`${label}-rAF2`);
        await waitForFrame();
        performance.mark(`${label}-rAF2-end`);
        performance.mark(`${label}-layout-start`);
        const _ = document.body.offsetHeight; // force layout
        performance.mark(`${label}-layout-end`);
        performance.mark(`${label}-end`);

        performance.measure(`${label}-total`, `${label}-start`, `${label}-end`);
        performance.measure(`${label}-event-dispatch`, `${label}-dispatch-start`, `${label}-dispatch-end`);
        performance.measure(`${label}-rAF1-wait`, `${label}-dispatch-end`, `${label}-rAF1`);
        performance.measure(`${label}-rAF1-work`, `${label}-rAF1`, `${label}-rAF1-end`);
        performance.measure(`${label}-rAF2-wait`, `${label}-rAF1-end`, `${label}-rAF2`);
        performance.measure(`${label}-rAF2-work`, `${label}-rAF2`, `${label}-rAF2-end`);
        performance.measure(`${label}-layout`, `${label}-layout-start`, `${label}-layout-end`);
        performance.measure(`${label}-after-action`, `${label}-rAF2-end`, `${label}-end`);
      }

      const node = document.querySelector('[data-testid="node"]');
      if (!node) return { error: "no nodes" };

      const rect = node.getBoundingClientRect();
      const cx = rect.x + rect.width / 2;
      const cy = rect.y + rect.height / 2;
      const canvas = document.querySelector('[data-testid="canvas-root"]');
      if (!canvas) return { error: "no canvas" };

      // Measure SELECT (mousedown + mouseup on node)
      const selectProfile = await measureInteraction(async () => {
        node.dispatchEvent(new MouseEvent("mousedown", {
          bubbles: true, cancelable: true, clientX: cx, clientY: cy, button: 0,
        }));
        node.dispatchEvent(new MouseEvent("mouseup", {
          bubbles: true, cancelable: true, clientX: cx, clientY: cy, button: 0,
        }));
        // Wait for Dioxus to process and browser to settle
        await waitForFrame();
        await waitForFrame();
      }, "select");

      // Measure DESELECT (mousedown + mouseup on empty canvas)
      const canvasRect = canvas.getBoundingClientRect();
      const deselectProfile = await measureInteraction(async () => {
        canvas.dispatchEvent(new MouseEvent("mousedown", {
          bubbles: true, cancelable: true,
          clientX: canvasRect.x + 10, clientY: canvasRect.y + 10, button: 0,
        }));
        canvas.dispatchEvent(new MouseEvent("mouseup", {
          bubbles: true, cancelable: true,
          clientX: canvasRect.x + 10, clientY: canvasRect.y + 10, button: 0,
        }));
        await waitForFrame();
        await waitForFrame();
      }, "deselect");

      // Get all performance entries
      const entries = performance.getEntriesByType("measure");
      const timing: Record<string, number> = {};
      for (const e of entries) {
        timing[e.name] = Math.round(e.duration);
      }
      performance.clearMeasures();

      // Count DOM elements
      const domNodes = document.querySelectorAll('[data-testid="node"]').length;
      const totalDivs = document.querySelectorAll("div").length;

      return { timing, domNodes, totalDivs };
    });

    console.log("[JS-PROFILE] Timing breakdown (2000 nodes, 2000 visible):");
    for (const [name, ms] of Object.entries(result.timing)) {
      console.log(`  ${name}: ${ms}ms`);
    }
    console.log(`[JS-PROFILE] DOM: ${result.domNodes} nodes, ${result.totalDivs} divs`);

    expect(pageErrors).toHaveLength(0);
  });

  test("500 nodes — timing breakdown for comparison", async ({ page }) => {
    test.setTimeout(180_000);
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    // Load 500-node document
    const cols = Math.ceil(Math.sqrt(500 * 1.2));
    const nodesObj: Record<string, unknown> = {};
    for (let i = 0; i < 500; i++) {
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

    // Zoom out to show all 500 nodes
    for (let i = 0; i < 20; i++) {
      await page.keyboard.press("_");
      await page.waitForTimeout(300);
      const domVisible = await page.evaluate(() => document.querySelectorAll('[data-testid="node"]').length);
      if (domVisible >= 450) break;
    }

    const result = await page.evaluate(async () => {
      function waitForFrame(): Promise<void> {
        return new Promise((resolve) => requestAnimationFrame(() => resolve()));
      }

      async function measureInteraction(
        action: () => Promise<void>,
        label: string,
      ): Promise<void> {
        performance.mark(`${label}-start`);
        performance.mark(`${label}-dispatch-start`);
        await action();
        performance.mark(`${label}-dispatch-end`);
        performance.mark(`${label}-rAF1`);
        await waitForFrame();
        performance.mark(`${label}-rAF1-end`);
        performance.mark(`${label}-rAF2`);
        await waitForFrame();
        performance.mark(`${label}-rAF2-end`);
        performance.mark(`${label}-layout-start`);
        const _ = document.body.offsetHeight;
        performance.mark(`${label}-layout-end`);
        performance.mark(`${label}-end`);

        performance.measure(`${label}-total`, `${label}-start`, `${label}-end`);
        performance.measure(`${label}-event-dispatch`, `${label}-dispatch-start`, `${label}-dispatch-end`);
        performance.measure(`${label}-rAF1-wait`, `${label}-dispatch-end`, `${label}-rAF1`);
        performance.measure(`${label}-rAF1-work`, `${label}-rAF1`, `${label}-rAF1-end`);
        performance.measure(`${label}-rAF2-wait`, `${label}-rAF1-end`, `${label}-rAF2`);
        performance.measure(`${label}-rAF2-work`, `${label}-rAF2`, `${label}-rAF2-end`);
        performance.measure(`${label}-layout`, `${label}-layout-start`, `${label}-layout-end`);
        performance.measure(`${label}-after-action`, `${label}-rAF2-end`, `${label}-end`);
      }

      const node = document.querySelector('[data-testid="node"]');
      if (!node) return { error: "no nodes" };
      const rect = node.getBoundingClientRect();
      const cx = rect.x + rect.width / 2;
      const cy = rect.y + rect.height / 2;
      const canvas = document.querySelector('[data-testid="canvas-root"]');
      if (!canvas) return { error: "no canvas" };

      const selectProfile = await measureInteraction(async () => {
        node.dispatchEvent(new MouseEvent("mousedown", {
          bubbles: true, cancelable: true, clientX: cx, clientY: cy, button: 0,
        }));
        node.dispatchEvent(new MouseEvent("mouseup", {
          bubbles: true, cancelable: true, clientX: cx, clientY: cy, button: 0,
        }));
        await waitForFrame();
        await waitForFrame();
      }, "select");

      const canvasRect = canvas.getBoundingClientRect();
      const deselectProfile = await measureInteraction(async () => {
        canvas.dispatchEvent(new MouseEvent("mousedown", {
          bubbles: true, cancelable: true,
          clientX: canvasRect.x + 10, clientY: canvasRect.y + 10, button: 0,
        }));
        canvas.dispatchEvent(new MouseEvent("mouseup", {
          bubbles: true, cancelable: true,
          clientX: canvasRect.x + 10, clientY: canvasRect.y + 10, button: 0,
        }));
        await waitForFrame();
        await waitForFrame();
      }, "deselect");

      const entries = performance.getEntriesByType("measure");
      const timing: Record<string, number> = {};
      for (const e of entries) {
        timing[e.name] = Math.round(e.duration);
      }
      performance.clearMeasures();
      const domNodes = document.querySelectorAll('[data-testid="node"]').length;
      const totalDivs = document.querySelectorAll("div").length;
      return { timing, domNodes, totalDivs };
    });

    console.log("[JS-PROFILE] Timing breakdown (500 nodes, 500 visible):");
    for (const [name, ms] of Object.entries(result.timing)) {
      console.log(`  ${name}: ${ms}ms`);
    }
    console.log(`[JS-PROFILE] DOM: ${result.domNodes} nodes, ${result.totalDivs} divs`);

    expect(pageErrors).toHaveLength(0);
  });
});
