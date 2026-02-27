import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "diagram_tool/e2e",
  fullyParallel: true,
  workers: 4,
  timeout: 45_000,
  retries: 1,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: "http://127.0.0.1:8081",
    headless: true,
    timezoneId: "UTC",
    locale: "en-US",
    trace: "retain-on-failure",
    video: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "baseline-chromium",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1600, height: 980 },
      },
      testIgnore: ["**/specs-redqueen/**"],
    },
    {
      name: "baseline-iphone-13",
      use: {
        ...devices["iPhone 13"],
      },
      testIgnore: ["**/specs-redqueen/**"],
    },
    {
      name: "baseline-pixel-7",
      use: {
        ...devices["Pixel 7"],
      },
      testIgnore: ["**/specs-redqueen/**"],
    },
    {
      name: "rq-wave1",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1600, height: 980 },
      },
      testMatch: ["**/specs-redqueen/**/*.wave1.spec.ts"],
    },
    {
      name: "rq-wave3",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1600, height: 980 },
      },
      timeout: 75_000,
      testMatch: ["**/specs-redqueen/**/*.wave3.spec.ts"],
    },
  ],
  webServer: {
    command: "dx serve --platform web --port 8081 --watch false --hot-reload false",
    url: "http://127.0.0.1:8081",
    reuseExistingServer: true,
    timeout: 300_000,
  },
});
