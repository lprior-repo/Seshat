import { expect, test, type Page } from "@playwright/test";

import { runEffect } from "./helpers";
import {
  attachPerfMetric,
  createPageErrorGuard,
  summarizeFrames,
  summarizeLatency,
} from "./perf.helpers";

const OPERATION_P95_MS = 160;
const OPERATION_MAX_MS = 260;
const JANK_CUTOFF_MS = 50;
const JANK_RATIO_MAX = 0.08;
const FRAME_P95_MAX_MS = 34;
const FRAME_MAX_MS = 220;
const APP_URL = process.env.PLAYWRIGHT_TEST_BASE_URL ?? "http://127.0.0.1:8081/";

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
  await runEffect(() => page.goto(APP_URL, { waitUntil: "domcontentloaded" }));
  await runEffect(() =>
    page.waitForSelector(".canvas-container", {
      state: "attached",
      timeout: 10_000,
    }),
  );
  await runEffect(() =>
    page.getByRole("button", { name: "Validate", exact: true }).waitFor({
      state: "visible",
      timeout: 10_000,
    }),
  );
}

async function sampleButtonLatency(
  page: Page,
  labels: readonly string[],
  iterations: number,
): Promise<number[]> {
  return runEffect(() =>
    page.evaluate(
      ({ labels: requestedLabels, iterations: repeats }) => {
        const byExactLabel = (label: string): HTMLButtonElement => {
          const node = Array.from(document.querySelectorAll("button")).find(
            (candidate) => candidate.textContent?.trim() === label,
          );
          if (!(node instanceof HTMLButtonElement)) {
            throw new Error(`button not found: ${label}`);
          }
          return node;
        };

        const buttons = requestedLabels.map(byExactLabel);
        const frame = () =>
          new Promise<number>((resolve) => {
            requestAnimationFrame((ts) => resolve(ts));
          });

        return (async () => {
          const samples: number[] = [];
          await frame();
          for (let i = 0; i < repeats; i += 1) {
            for (const button of buttons) {
              const start = performance.now();
              button.click();
              await frame();
              samples.push(performance.now() - start);
            }
          }
          return samples;
        })();
      },
      { labels, iterations },
    ),
  );
}

async function sampleFrameJank(
  page: Page,
  totalFrames: number,
): Promise<number[]> {
  return runEffect(() =>
    page.evaluate(({ totalFrames: frames }) => {
      const byExactLabel = (label: string): HTMLButtonElement => {
        const node = Array.from(document.querySelectorAll("button")).find(
          (candidate) => candidate.textContent?.trim() === label,
        );
        if (!(node instanceof HTMLButtonElement)) {
          throw new Error(`button not found: ${label}`);
        }
        return node;
      };

      const propsButton = byExactLabel("Props");
      const validButton = byExactLabel("Valid");
      const validateButton = byExactLabel("Validate");
      const deltas: number[] = [];

      return (async () => {
        let previous = await new Promise<number>((resolve) => {
          requestAnimationFrame((ts) => resolve(ts));
        });

        for (let frameIndex = 0; frameIndex < frames; frameIndex += 1) {
          if (frameIndex % 2 === 0) {
            propsButton.click();
          }
          if (frameIndex % 3 === 0) {
            validButton.click();
          }
          if (frameIndex % 6 === 0) {
            validateButton.click();
          }

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

test.describe("diagram editor performance hardening", () => {
  test("keeps operation latency distributions within CI budget", async ({ page }, testInfo) => {
    const pageErrors = createPageErrorGuard(page);
    await bootPerformancePage(page);

    const panelSamples = await sampleButtonLatency(page, ["Props", "Valid"], 16);
    const panelSummary = summarizeLatency(panelSamples);
    attachPerfMetric(testInfo, "panel-toggle-latency-ms", panelSummary);

    expect(panelSummary.p95Ms).toBeLessThanOrEqual(OPERATION_P95_MS);
    expect(panelSummary.maxMs).toBeLessThanOrEqual(OPERATION_MAX_MS);

    const validateZoomSamples = await sampleButtonLatency(page, ["Validate", "+", "-"], 10);
    const validateZoomSummary = summarizeLatency(validateZoomSamples);
    attachPerfMetric(testInfo, "validate-zoom-latency-ms", validateZoomSummary);

    expect(validateZoomSummary.p95Ms).toBeLessThanOrEqual(OPERATION_P95_MS);
    expect(validateZoomSummary.maxMs).toBeLessThanOrEqual(OPERATION_MAX_MS);
    pageErrors.assertNone();
  });

  test("keeps RAF jank ratio low during deterministic interaction burst", async ({ page }, testInfo) => {
    const pageErrors = createPageErrorGuard(page);
    await bootPerformancePage(page);

    const frameDurations = await sampleFrameJank(page, 150);
    const frameSummary = summarizeFrames(frameDurations, JANK_CUTOFF_MS);
    attachPerfMetric(testInfo, "raf-jank", {
      ...frameSummary,
      jankCutoffMs: JANK_CUTOFF_MS,
    });

    expect(frameSummary.jankRatio).toBeLessThanOrEqual(JANK_RATIO_MAX);
    expect(frameSummary.p95FrameMs).toBeLessThanOrEqual(FRAME_P95_MAX_MS);
    expect(frameSummary.maxFrameMs).toBeLessThanOrEqual(FRAME_MAX_MS);
    pageErrors.assertNone();
  });
});
