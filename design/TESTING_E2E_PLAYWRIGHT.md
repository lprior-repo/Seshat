# End-to-End Testing (`playwright`)

While `proptest` perfectly verifies our pure Rust core, it cannot prove that the Dioxus UI renders correctly or that the browser's DOM events are firing as expected. For this, we use **Playwright**.

## Architecture
Playwright runs a headless Chromium browser, boots up the compiled Dioxus Web application (`dx serve`), and interacts with it exactly like a human user would.

It ensures that:
1. The `wasm32-unknown-unknown` boundary is intact and no forbidden libraries crashed the load.
2. Tailwind CSS classes are actively applied.
3. DOM manipulation bypasses (via `document::eval`) trigger the correct Rust channels.

## The Test Suite (`playwright.config.ts`)
Our E2E tests live in the `/tests/` or `/playwright/` folder.

```javascript
import { test, expect } from '@playwright/test';

test('human can drag a node', async ({ page }) => {
  // 1. Load the app
  await page.goto('http://localhost:3333');

  // 2. Identify the node
  const node = page.locator('[data-node-id="node-1"]');
  const startBox = await node.boundingBox();

  // 3. Simulate exact human mouse events
  await page.mouse.move(startBox.x + 10, startBox.y + 10);
  await page.mouse.down();
  await page.mouse.move(startBox.x + 100, startBox.y + 100);
  await page.mouse.up();

  // 4. Assert visual update
  const endBox = await node.boundingBox();
  expect(endBox.x).toBeGreaterThan(startBox.x + 80);
});
```

## Contract Guarantee
The Playwright suite is the final arbiter of truth for human UI interactability. If `moon run :ci-source` passes the Rust checks, but Playwright fails, the plane cannot land. The task is incomplete.

Because we guarantee an 8ms frame budget, we also utilize Playwright's Chrome DevTools Protocol (CDP) bindings to occasionally assert that heavy drag operations do not block the main thread for more than 16ms during execution.