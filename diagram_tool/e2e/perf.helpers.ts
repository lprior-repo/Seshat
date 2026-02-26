import { expect, type Page, type TestInfo } from "@playwright/test";

export type LatencySummary = {
  readonly samples: number[];
  readonly minMs: number;
  readonly p50Ms: number;
  readonly p95Ms: number;
  readonly maxMs: number;
  readonly avgMs: number;
};

export type FrameSummary = {
  readonly totalFrames: number;
  readonly jankFrames: number;
  readonly jankRatio: number;
  readonly p95FrameMs: number;
  readonly maxFrameMs: number;
  readonly avgFrameMs: number;
};

const byAscending = (a: number, b: number) => a - b;

function percentile(sorted: readonly number[], q: number): number {
  if (sorted.length === 0) {
    return 0;
  }
  const idx = Math.max(0, Math.min(sorted.length - 1, Math.ceil(sorted.length * q) - 1));
  return sorted[idx] ?? 0;
}

export function summarizeLatency(samples: readonly number[]): LatencySummary {
  const sorted = [...samples].sort(byAscending);
  const total = sorted.reduce((acc, value) => acc + value, 0);
  return {
    samples: sorted,
    minMs: sorted[0] ?? 0,
    p50Ms: percentile(sorted, 0.5),
    p95Ms: percentile(sorted, 0.95),
    maxMs: sorted[sorted.length - 1] ?? 0,
    avgMs: sorted.length > 0 ? total / sorted.length : 0,
  };
}

export function summarizeFrames(
  frameDurations: readonly number[],
  jankCutoffMs: number,
): FrameSummary {
  const sorted = [...frameDurations].sort(byAscending);
  const jankFrames = sorted.filter((value) => value > jankCutoffMs).length;
  const total = sorted.reduce((acc, value) => acc + value, 0);
  return {
    totalFrames: sorted.length,
    jankFrames,
    jankRatio: sorted.length > 0 ? jankFrames / sorted.length : 0,
    p95FrameMs: percentile(sorted, 0.95),
    maxFrameMs: sorted[sorted.length - 1] ?? 0,
    avgFrameMs: sorted.length > 0 ? total / sorted.length : 0,
  };
}

export function createPageErrorGuard(page: Page) {
  const errors: string[] = [];
  page.on("pageerror", (error) => {
    errors.push(error.message);
  });
  return {
    errors,
    assertNone() {
      expect(errors).toHaveLength(0);
    },
  };
}

export function attachPerfMetric(
  testInfo: TestInfo,
  metricName: string,
  value: unknown,
): void {
  const payload = JSON.stringify(value, null, 2);
  testInfo.annotations.push({
    type: `perf:${metricName}`,
    description: payload,
  });
  console.log(`[perf] ${metricName}: ${payload}`);
}
