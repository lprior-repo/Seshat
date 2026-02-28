import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "diagram_tool/e2e",
  outputDir: "/tmp/seshat-playwright/test-results",
  fullyParallel: true,
  timeout: 45_000,
  reporter: [["list"], ["html", { open: "never", outputFolder: "/tmp/seshat-playwright/report" }]],
  use: {
    baseURL: "http://127.0.0.1:8082",
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
      retries: 2,
      workers: 12,
      grep: /@baseline/,
      grepInvert: /@rq/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "baseline",
      retries: 2,
      workers: 12,
      grep: /@baseline/,
      grepInvert: /@rq/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-wave1",
      retries: 1,
      workers: 6,
      grep: /(?=.*@rq)(?=.*@rq-wave1)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-wave2",
      retries: 2,
      workers: 6,
      grep: /(?=.*@rq)(?=.*@rq-wave2)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-wave3",
      retries: 1,
      workers: 4,
      grep: /(?=.*@rq)(?=.*@rq-wave3)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-seeded",
      retries: 1,
      workers: 6,
      grep: /(?=.*@rq)(?=.*@seeded)/,
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "redqueen-stress",
      retries: 1,
      workers: 4,
      grep: /(?=.*@rq)(?=.*@stress)/,
      use: {
        browserName: "chromium",
      },
    },
  ],
  webServer: {
    command:
      "moon run :serve-e2e",
    url: "http://127.0.0.1:8082",
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
