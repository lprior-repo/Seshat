//! Empirical interaction latency benchmarks for Seshat diagram tool.
//!
//! Measures wall-clock time for user interactions: node creation, drag, selection,
//! undo/redo, edge creation, multi-select, and batch drag.
//!
//! Run with:  npx playwright test --project=perf-latency --reporter=list
//!
//! @perf-latency

import { expect, test } from "@playwright/test";
import {
  canvas,
  createTextNode,
  edgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  nodeCount,
  runEffect,
  runEffectsSequential,
  selectedCount,
  waitForNoRebuildOverlay,
  trapPageErrors,
} from "./helpers";
import { summarizeLatency } from "./perf.helpers";

// ---------------------------------------------------------------------------
// Timing helpers
// ---------------------------------------------------------------------------

async function timeMs(label: string, fn: () => Promise<void>): Promise<number> {
  const start = performance.now();
  await fn();
  const elapsed = performance.now() - start;
  console.log(`[PERF] ${label}: ${elapsed.toFixed(1)} ms`);
  return elapsed;
}

/** Measure DOM update time after an action by polling until node/edge count stabilizes. */
async function timeUntilStable(
  label: string,
  fn: () => Promise<void>,
  pollFn: () => Promise<number>,
  expectedValue: number,
): Promise<number> {
  const start = performance.now();
  await fn();
  // Poll until the expected value appears (with timeout baked into expect.poll)
  await expect.poll(pollFn, { timeout: 30_000 }).toBe(expectedValue);
  const elapsed = performance.now() - start;
  console.log(`[PERF] ${label}: ${elapsed.toFixed(1)} ms (until DOM stable)`);
  return elapsed;
}

// ---------------------------------------------------------------------------
// Direct node creation helper (avoids createTextNode's dispatchEvent issues)
// ---------------------------------------------------------------------------

/**
 * Create a text node using keyboard shortcut 't' + canvas click.
 * More robust than createTextNode for rapid loops because:
 * - Keyboard shortcut always works regardless of toolbar DOM changes
 * - Handles rebuild overlay between tool activation and canvas click
 * - Waits for overlay after the click before returning
 *
 * IMPORTANT: The offset (x, y) must land on EMPTY canvas, not on an existing node.
 * The node's mousedown handler stops propagation, preventing the canvas-root
 * Text tool handler from creating a new node. Nodes must be spaced >= 200px apart.
 */
async function createNodeDirect(
  page: Parameters<typeof freshStart>[0],
  canvasArea: ReturnType<typeof canvas>,
  x: number,
  y: number,
): Promise<void> {
  // Ensure rebuild overlay is gone before we start
  await waitForNoRebuildOverlay(page);
  // Activate text tool via keyboard (idempotent, always works)
  await page.keyboard.press("t");
  await page.waitForTimeout(50);
  // Ensure overlay didn't appear during tool switch
  await waitForNoRebuildOverlay(page);
  const box = await runEffect(() => canvasArea.boundingBox());
  if (!box) throw new Error("canvas not found");
  await page.mouse.click(box.x + x, box.y + y);
  // Wait for Dioxus to process the click and for rebuild overlay to clear
  await waitForNoRebuildOverlay(page);
  await page.waitForTimeout(100);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("interaction latency", () => {
  test.describe.configure({ mode: "serial" });

  test("1. node creation — click on canvas", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    const creationTimes: number[] = [];

    // Create 5 nodes and measure each (200px apart to avoid overlapping)
    for (let i = 0; i < 5; i++) {
      const t = await timeUntilStable(
        `create node ${i + 1}`,
        () => createNodeDirect(page, canvasArea, 100 + i * 200, 200),
        () => nodeCount(page),
        i + 1,
      );
      creationTimes.push(t);
    }

    const summary = summarizeLatency(creationTimes);
    console.log(
      `[PERF] node creation summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms max=${summary.maxMs.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("2. single node drag — 50px", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    await createTextNode(page, canvasArea, 300, 300);
    await expectNodeCount(page, 1);

    const dragTimes: number[] = [];

    for (let i = 0; i < 5; i++) {
      const node = page.getByTestId("node").first();
      const box = await runEffect(() => node.boundingBox());
      if (!box) throw new Error("node not visible");

      const cx = box.x + box.width / 2;
      const cy = box.y + box.height / 2;

      const t = await timeMs(`drag ${i + 1}`, async () => {
        await page.mouse.move(cx, cy);
        await page.mouse.down();
        await page.mouse.move(cx + 50, cy + 50, { steps: 10 });
        await page.mouse.up();
        // Wait for Dioxus to commit the drag
        await waitForNoRebuildOverlay(page);
        // Extra frame for DOM to update
        await page.waitForTimeout(50);
      });
      dragTimes.push(t);
    }

    const summary = summarizeLatency(dragTimes);
    console.log(
      `[PERF] single drag summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms max=${summary.maxMs.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("3. undo/redo cycle — single node drag", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    await createTextNode(page, canvasArea, 300, 300);
    await expectNodeCount(page, 1);

    const undoTimes: number[] = [];
    const redoTimes: number[] = [];

    for (let i = 0; i < 5; i++) {
      // Drag
      const node = page.getByTestId("node").first();
      const box = await runEffect(() => node.boundingBox());
      if (!box) throw new Error("node not visible");
      await runEffectsSequential([
        () => page.mouse.move(box.x + box.width / 2, box.y + box.height / 2),
        () => page.mouse.down(),
        () => page.mouse.move(box.x + box.width / 2 + 40, box.y + box.height / 2 + 40, { steps: 8 }),
        () => page.mouse.up(),
        () => waitForNoRebuildOverlay(page),
      ]);

      const tUndo = await timeMs(`undo ${i + 1}`, async () => {
        await page.locator('[data-testid="toolbar-undo"]').first().click();
        await waitForNoRebuildOverlay(page);
        await page.waitForTimeout(30);
      });
      undoTimes.push(tUndo);

      const tRedo = await timeMs(`redo ${i + 1}`, async () => {
        await page.locator('[data-testid="toolbar-redo"]').first().click();
        await waitForNoRebuildOverlay(page);
        await page.waitForTimeout(30);
      });
      redoTimes.push(tRedo);
    }

    const undoSummary = summarizeLatency(undoTimes);
    const redoSummary = summarizeLatency(redoTimes);
    console.log(
      `[PERF] undo summary: avg=${undoSummary.avgMs.toFixed(1)}ms p50=${undoSummary.p50Ms.toFixed(1)}ms p95=${undoSummary.p95Ms.toFixed(1)}ms`,
    );
    console.log(
      `[PERF] redo summary: avg=${redoSummary.avgMs.toFixed(1)}ms p50=${redoSummary.p50Ms.toFixed(1)}ms p95=${redoSummary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("4. click-to-select single node", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    await createTextNode(page, canvasArea, 300, 300);
    await expectNodeCount(page, 1);

    const selectTimes: number[] = [];

    for (let i = 0; i < 10; i++) {
      // Click to select, then click empty to deselect, repeat
      const node = page.getByTestId("node").first();
      const box = await runEffect(() => node.boundingBox());
      if (!box) throw new Error("node not visible");

      const t = await timeMs(`select cycle ${i + 1}`, async () => {
        await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
        await expect.poll(() => selectedCount(page)).toBe(1);
        // Deselect by clicking empty area
        await page.mouse.click(box.x + box.width + 100, box.y);
        await expect.poll(() => selectedCount(page)).toBe(0);
      });
      selectTimes.push(t);
    }

    const summary = summarizeLatency(selectTimes);
    console.log(
      `[PERF] select cycle (select+deselect) summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("5. multi-select — shift-click 5 nodes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);

    // Create 5 nodes using robust helper (200px apart to avoid overlapping)
    for (let i = 0; i < 5; i++) {
      await createNodeDirect(page, canvasArea, 100 + i * 200, 200);
    }
    await expectNodeCount(page, 5);

    const multiSelectTimes: number[] = [];

    for (let i = 0; i < 3; i++) {
      // First deselect — switch to select tool and click empty canvas area
      await page.keyboard.press("v");
      await page.waitForTimeout(30);
      const deselectBox = await runEffect(() => canvasArea.boundingBox());
      if (!deselectBox) throw new Error("canvas not found");
      // Click top-left corner of canvas (empty area, well away from nodes)
      await page.mouse.click(deselectBox.x + 20, deselectBox.y + 20);
      await expect.poll(() => selectedCount(page)).toBe(0);

      const t = await timeMs(`multi-select ${i + 1}`, async () => {
        await page.keyboard.down("Shift");
        for (let j = 0; j < 5; j++) {
          const node = page.getByTestId("node").nth(j);
          const box = await runEffect(() => node.boundingBox());
          if (!box) throw new Error(`node ${j} not visible`);
          await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
        }
        await page.keyboard.up("Shift");
        await expect.poll(() => selectedCount(page)).toBe(5);
      });
      multiSelectTimes.push(t);
    }

    const summary = summarizeLatency(multiSelectTimes);
    console.log(
      `[PERF] multi-select (shift-click 5 nodes) summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("6. batch drag — move 5 selected nodes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);

    // Create 5 nodes in a column using robust helper (200px apart vertically)
    for (let i = 0; i < 5; i++) {
      await createNodeDirect(page, canvasArea, 300, 100 + i * 200);
    }
    await expectNodeCount(page, 5);

    // Select all 5 via shift-click
    await page.keyboard.press("v");
    await page.waitForTimeout(30);
    // Deselect any previously selected node (last created node is auto-selected)
    const deselectBox = await runEffect(() => canvasArea.boundingBox());
    if (!deselectBox) throw new Error("canvas not found");
    await page.mouse.click(deselectBox.x + 20, deselectBox.y + 20);
    await expect.poll(() => selectedCount(page)).toBe(0);
    // Now shift-click all 5 nodes
    await page.keyboard.down("Shift");
    for (let j = 0; j < 5; j++) {
      const node = page.getByTestId("node").nth(j);
      const box = await runEffect(() => node.boundingBox());
      if (!box) throw new Error(`node ${j} not visible`);
      await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
    }
    await page.keyboard.up("Shift");
    await expect.poll(() => selectedCount(page)).toBe(5);

    const batchDragTimes: number[] = [];

    for (let i = 0; i < 5; i++) {
      const firstNode = page.getByTestId("node").first();
      const box = await runEffect(() => firstNode.boundingBox());
      if (!box) throw new Error("first node not visible");

      const cx = box.x + box.width / 2;
      const cy = box.y + box.height / 2;

      const t = await timeMs(`batch drag ${i + 1}`, async () => {
        await page.mouse.move(cx, cy);
        await page.mouse.down();
        await page.mouse.move(cx + 30, cy + 20, { steps: 10 });
        await page.mouse.up();
        await waitForNoRebuildOverlay(page);
        await page.waitForTimeout(50);
      });
      batchDragTimes.push(t);
    }

    const summary = summarizeLatency(batchDragTimes);
    console.log(
      `[PERF] batch drag (5 selected nodes) summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("7. edge creation — connect 2 nodes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);

    // Create 2 nodes: left and right (300px apart)
    await createNodeDirect(page, canvasArea, 150, 300);
    await expectNodeCount(page, 1);
    await createNodeDirect(page, canvasArea, 550, 300);
    await expectNodeCount(page, 2);

    const edgeTimes: number[] = [];

    for (let i = 0; i < 3; i++) {
      // Delete any existing edges first (select + delete)
      if (i > 0) {
        // Switch to select mode
        await page.keyboard.press("v");
        await page.waitForTimeout(30);
        // Click on the edge to select it
        const edge = page.getByTestId("edge").first();
        if ((await edge.count()) > 0) {
          await edge.click();
          await page.keyboard.press("Delete");
          await waitForNoRebuildOverlay(page);
          await expect.poll(() => edgeCount(page)).toBe(0);
        }
      }

      // Switch to edge tool (keyboard shortcut is 'l', not 'e')
      await page.keyboard.press("l");
      await page.waitForTimeout(50);

      // Get node centers for connection
      const nodes = page.getByTestId("node");
      const sourceBox = await runEffect(() => nodes.nth(0).boundingBox());
      const targetBox = await runEffect(() => nodes.nth(1).boundingBox());
      if (!sourceBox || !targetBox) throw new Error("nodes not visible");

      const sourceCx = sourceBox.x + sourceBox.width / 2;
      const sourceCy = sourceBox.y + sourceBox.height / 2;
      const targetCx = targetBox.x + targetBox.width / 2;
      const targetCy = targetBox.y + targetBox.height / 2;

      const t = await timeMs(`edge creation ${i + 1}`, async () => {
        // Mouse down on source node center
        await page.mouse.move(sourceCx, sourceCy);
        await page.mouse.down();
        // Drag to target node center
        await page.mouse.move(targetCx, targetCy, { steps: 10 });
        await page.mouse.up();
        // Wait for edge to be created
        await waitForNoRebuildOverlay(page);
        await expect.poll(() => edgeCount(page), { timeout: 10_000 }).toBe(1);
      });
      edgeTimes.push(t);

      // Switch back to select mode for next iteration
      await page.keyboard.press("v");
      await page.waitForTimeout(30);
    }

    const summary = summarizeLatency(edgeTimes);
    console.log(
      `[PERF] edge creation summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms max=${summary.maxMs.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("8. rapid zoom — 5 clicks via keyboard", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const zoomTimes: number[] = [];

    for (let i = 0; i < 5; i++) {
      const t = await timeMs(`zoom in ${i + 1}`, async () => {
        await page.keyboard.press("=");
        await page.waitForTimeout(50);
      });
      zoomTimes.push(t);
    }

    const summary = summarizeLatency(zoomTimes);
    console.log(
      `[PERF] zoom click summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("9. rapid keyboard shortcut storm (v, h, Escape, Delete)", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const keys = ["v", "h", "Escape", "Delete", "Backspace", "z", "y", "a", "0"];
    const stormTimes: number[] = [];

    for (let i = 0; i < 5; i++) {
      const t = await timeMs(`shortcut storm ${i + 1}`, async () => {
        for (const key of keys) {
          await page.keyboard.press(key);
        }
        await page.waitForTimeout(20);
      });
      stormTimes.push(t);
    }

    const summary = summarizeLatency(stormTimes);
    console.log(
      `[PERF] keyboard storm (9 keys) summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("10. theme toggle cycle — 4 modes × 3 rounds", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const toggleBtn = page.getByTestId("theme-toggle-btn");
    const cycleTimes: number[] = [];

    for (let round = 0; round < 3; round++) {
      const t = await timeMs(`theme cycle ${round + 1}`, async () => {
        // 4 clicks = full cycle
        for (let i = 0; i < 4; i++) {
          await toggleBtn.click();
          await page.waitForTimeout(30);
        }
      });
      cycleTimes.push(t);
    }

    const summary = summarizeLatency(cycleTimes);
    console.log(
      `[PERF] theme cycle (4 modes) summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });

  test("11. wheel zoom — 5 steps in, 5 steps out", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    const box = await runEffect(() => canvasArea.boundingBox());
    if (!box) throw new Error("canvas not found");

    const wheelTimes: number[] = [];

    // Zoom in 5 steps
    const tIn = await timeMs("wheel zoom in 5×", async () => {
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      for (let i = 0; i < 5; i++) {
        await page.mouse.wheel(0, -200);
        await page.waitForTimeout(30);
      }
    });
    wheelTimes.push(tIn);

    // Zoom out 5 steps
    const tOut = await timeMs("wheel zoom out 5×", async () => {
      for (let i = 0; i < 5; i++) {
        await page.mouse.wheel(0, 200);
        await page.waitForTimeout(30);
      }
    });
    wheelTimes.push(tOut);

    const summary = summarizeLatency(wheelTimes);
    console.log(
      `[PERF] wheel zoom (5 in + 5 out) summary: avg=${summary.avgMs.toFixed(1)}ms p50=${summary.p50Ms.toFixed(1)}ms p95=${summary.p95Ms.toFixed(1)}ms`,
    );
    expect(pageErrors).toHaveLength(0);
  });
});
