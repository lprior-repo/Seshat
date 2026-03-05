import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { freshStart, runEffect, runEffectsSequential, waitForUiReady, waitForE2eReady, resetDocument, waitForCleanState, waitForNoRebuildOverlay, nodeCount } from "./helpers";
import {
  attachPerfMetric,
  createPageErrorGuard,
  summarizeFrames,
  summarizeLatency,
} from "./perf.helpers";

const JANK_CUTOFF_MS = 50;
const JANK_RATIO_MAX = 0.50; // allow some jank in baseline
const FRAME_MAX_MS = 1000;

async function bootPerformancePage(page: Page): Promise<void> {
  await runEffect(() =>
    page.addInitScript(() => {
      const originalFetch = window.fetch.bind(window);
      window.fetch = (input: RequestInfo | URL, init?: RequestInit) => {
        const url =
          typeof input === "string"
            ? input
            : input instanceof URL
              ? input.toString()
              : input.url;

        if (url.includes("/_dioxus")) {
          return Promise.resolve(
            new Response("{}", {
              status: 200,
              headers: { "content-type": "application/json" },
            }),
          );
        }

        return originalFetch(input, init);
      };
    }),
  );
  await freshStart(page);
}

async function loadTestScene(page: Page, sceneName: string) {
  const filePath = resolve(__dirname, "scenes", `${sceneName}.json`);
  const payload = readFileSync(filePath, "utf8");

  await runEffectsSequential([
    () => waitForUiReady(page),
    () => page.evaluate((jsonPayload) => {
      (window as any).__SESHAT_E2E_IMPORT_JSON = jsonPayload;
    }, payload),
    () => expect(page.getByTestId("toolbar-open")).toBeEnabled({ timeout: 15_000 }),
    () => page.getByTestId("toolbar-open").click(),
    () => waitForNoRebuildOverlay(page),
  ]);

  await expect.poll(() => nodeCount(page), { timeout: 30_000 }).toBe(3000);
}

async function sampleFrameJank(page: Page, totalFrames: number): Promise<number[]> {
  return runEffect(() =>
    page.evaluate(({ totalFrames: frames }) => {
      const deltas: number[] = [];
      const canvas = document.querySelector('[data-testid="canvas-root"]');
      if (!canvas) throw new Error("Canvas not found");

      return (async () => {
        let previous = await new Promise<number>((resolve) => {
          requestAnimationFrame((ts) => resolve(ts));
        });

        for (let frameIndex = 0; frameIndex < frames; frameIndex += 1) {
          // Simulate panning
          canvas.dispatchEvent(new WheelEvent('wheel', {
            deltaX: 10,
            deltaY: 10,
            bubbles: true,
            cancelable: true
          }));

          const now = await new Promise<number>((resolve) => {
            requestAnimationFrame((ts) => resolve(ts));
          });
          deltas.push(now - previous);
          previous = now;
        }
        return deltas;
      })();
    }, { totalFrames }),
  );
}

test.describe("diagram editor 3000 node baseline", () => {
  test("measure FPS during panning with 3000 nodes", async ({ page }, testInfo) => {
    // Extend timeout for heavy test
    test.setTimeout(120_000);
    const pageErrors = createPageErrorGuard(page);
    
    await bootPerformancePage(page);
    await loadTestScene(page, "perf-3000");

    // Let the renderer settle
    await page.waitForTimeout(1000);

    // Measure memory before
    const memBefore = await page.evaluate(() => (performance as any).memory?.usedJSHeapSize || 0);

    // Pan around and measure jank
    const frameDurations = await sampleFrameJank(page, 100);
    
    // Measure memory after
    const memAfter = await page.evaluate(() => (performance as any).memory?.usedJSHeapSize || 0);
    const memDeltaMB = (memAfter - memBefore) / (1024 * 1024);

    const frameSummary = summarizeFrames(frameDurations, JANK_CUTOFF_MS);
    attachPerfMetric(testInfo, "3000-nodes-raf-jank", {
      ...frameSummary,
      jankCutoffMs: JANK_CUTOFF_MS,
      memoryDeltaMB: memDeltaMB
    });

    console.log(`Average Frame MS: ${frameSummary.avgFrameMs.toFixed(2)}ms`);
    console.log(`Jank Ratio: ${frameSummary.jankRatio.toFixed(2)}`);
    console.log(`Memory Delta: ${memDeltaMB.toFixed(2)} MB`);

    expect(frameSummary.jankRatio).toBeLessThanOrEqual(JANK_RATIO_MAX);
    expect(frameSummary.maxFrameMs).toBeLessThanOrEqual(FRAME_MAX_MS);
    expect(memDeltaMB).toBeLessThan(500); // Should not leak 500MB
    
    pageErrors.assertNone();
  });
});
