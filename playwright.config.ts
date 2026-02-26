import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "diagram_tool/e2e",
  fullyParallel: true,
  workers: 4,
  timeout: 45_000,
  retries: 1,
  use: {
    baseURL: "http://127.0.0.1:8081",
    headless: true,
  },
  webServer: {
    command:
      "sh -c 'fuser -k 8081/tcp || true; dx serve --platform web --port 8081 --watch false --hot-reload false'",
    url: "http://127.0.0.1:8081",
    reuseExistingServer: true,
    timeout: 300_000,
  },
});
