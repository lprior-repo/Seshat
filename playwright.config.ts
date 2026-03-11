import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "diagram_tool/e2e",
  fullyParallel: true,
  timeout: 45_000,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: "http://127.0.0.1:8081",
    headless: true,
    viewport: { width: 1600, height: 980 },
    timezoneId: "UTC",
    locale: "en-US",
    trace: "retain-on-failure",
    video: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "e2e-smoke",
      retries: 0,
      workers: 4,
      grep: /@baseline/,
      grepInvert: /@rq/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "baseline",
      retries: 1,
      workers: 4,
      grep: /@baseline/,
      grepInvert: /@rq/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-wave1",
      retries: 0,
      workers: 2,
      grep: /(?=.*@rq)(?=.*@rq-wave1)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-wave2",
      retries: 1,
      workers: 2,
      grep: /(?=.*@rq)(?=.*@rq-wave2)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-wave3",
      retries: 0,
      workers: 1,
      grep: /(?=.*@rq)(?=.*@rq-wave3)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-seeded",
      retries: 0,
      workers: 2,
      grep: /(?=.*@rq)(?=.*@seeded)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-stress",
      retries: 0,
      workers: 1,
      grep: /(?=.*@rq)(?=.*@stress)/,
      use: {
        browserName: "chromium",
      },
    },
  ],
  webServer: {
    command: "moon run :serve-e2e",
    url: "http://127.0.0.1:8081",
    reuseExistingServer: true,
    timeout: 300_000,
  },
});
