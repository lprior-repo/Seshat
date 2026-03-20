import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "diagram_tool/e2e",
  timeout: 60000,
  use: {
    baseURL: "http://localhost:3333",
    headless: true,
    viewport: { width: 1600, height: 900 },
  },
  projects: [{ name: "debug", use: { browserName: "chromium" } }],
});
