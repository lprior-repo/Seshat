import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "diagram_tool/e2e",
  outputDir: "/tmp/seshat-playwright/test-results",
  fullyParallel: true,
  timeout: 45_000,
  reporter: [["list"], ["html", { open: "never", outputFolder: "/tmp/seshat-playwright/report" }]],
  use: {
    baseURL: "http://127.0.0.1:8084",
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
      use: {
        browserName: "chromium",
      },
    },
    {
      name: "baseline",
      retries: 2,
      workers: 12,
      grep: /@baseline/,
      use: {
        browserName: "chromium",
      },
    }
  ],
  webServer: {
    command:
      "cd diagram_tool && dx serve --platform web --port 8084 --open false --watch false --hot-reload false --interactive false",
    url: "http://127.0.0.1:8084",
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
