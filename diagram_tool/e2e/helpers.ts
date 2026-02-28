import { expect, type FileChooser, type Locator, type Page } from "@playwright/test";
import { Effect } from "effect";

const SELECTOR_CANVAS = '[data-testid="canvas-root"]';
const SELECTOR_COUNTER_NODES = '[data-testid="counter-nodes"]';
const SELECTOR_COUNTER_EDGES = '[data-testid="counter-edges"]';
const SELECTOR_COUNTER_SELECTED = '[data-testid="counter-selected"]';
const SELECTOR_ZOOM_RESET = '[data-testid="zoom-reset"]';
const SELECTOR_MINIMAP_VIEWPORT = '[data-testid="minimap-viewport"]';

/** Timeout (ms) for waiting for the WASM app to expose reset hooks. */
const E2E_READY_TIMEOUT = 30_000;

/** Interval (ms) between polls when waiting for __seshatE2eReady. */
const E2E_READY_POLL_MS = 100;

export async function runEffect<A>(thunk: () => Promise<A>): Promise<A> {
  return Effect.runPromise(
    Effect.tryPromise({
      try: thunk,
      catch: (error) =>
        new Error(error instanceof Error ? error.message : String(error)),
    }),
  );
}

export async function runEffectsSequential(
  steps: ReadonlyArray<() => Promise<unknown>>,
): Promise<void> {
  await runEffect(() =>
    Effect.runPromise(
      Effect.forEach(
        steps,
        (step) =>
          Effect.tryPromise({
            try: step,
            catch: (error) =>
              new Error(error instanceof Error ? error.message : String(error)),
          }),
        {
          concurrency: 1,
          discard: true,
        },
      ),
    ),
  );
}

export async function waitForUiReady(page: Page) {
  await ensureDeterministicUi(page);
  await expect(canvas(page)).toBeVisible();
  await expect(page.locator(SELECTOR_COUNTER_NODES).first()).toBeVisible({ timeout: 30_000 });
  await waitForNoRebuildOverlay(page);
}

export function canvas(page: Page): Locator {
  return page.locator(SELECTOR_CANVAS).first();
}

export function minimapViewport(page: Page): Locator {
  return page.locator(SELECTOR_MINIMAP_VIEWPORT).first();
}

export async function chooseFilesWithFileChooser(
  page: Page,
  trigger: () => Promise<unknown>,
  files: Parameters<FileChooser["setFiles"]>[0],
) {
  const [chooser] = await Promise.all([page.waitForEvent("filechooser"), trigger()]);
  await chooser.setFiles(files);
}

export async function cancelFileChooser(
  page: Page,
  trigger: () => Promise<unknown>,
) {
  await chooseFilesWithFileChooser(page, trigger, []);
}

export async function expectNodeCount(page: Page, count: number) {
  await expect.poll(() => nodeCount(page)).toBe(count);
}

export async function expectEdgeCount(page: Page, count: number) {
  await expect.poll(() => edgeCount(page)).toBe(count);
}

export async function expectSelectedCount(page: Page, count: number) {
  await expect.poll(() => selectedCount(page)).toBe(count);
}

async function readNumericCounter(page: Page, selector: string, fallbackPattern: RegExp): Promise<number> {
  return runEffect(() =>
    page.evaluate(
      ({ selectorText, fallbackSource }) => {
        const el = document.querySelector(selectorText);
        if (!el) {
          return 0;
        }

        const dataCount = el.getAttribute("data-count")?.trim() ?? "";
        if (/^\d+$/.test(dataCount)) {
          return Number.parseInt(dataCount, 10);
        }

        const fallback = new RegExp(fallbackSource);
        const text = el.textContent ?? "";
        const match = text.match(fallback);
        const digits = match?.[1] ?? "0";
        const parsed = Number.parseInt(digits, 10);
        return Number.isNaN(parsed) ? 0 : parsed;
      },
      {
        selectorText: selector,
        fallbackSource: fallbackPattern.source,
      },
    ),
  );
}

export async function ensureDeterministicUi(page: Page) {
  await runEffect(() =>
    page.evaluate(() => {
      if (document.documentElement.dataset.seshatE2eDeterministic === "1") {
        return;
      }

      document.documentElement.dataset.seshatE2eDeterministic = "1";
      const style = document.createElement("style");
      style.setAttribute("data-seshat-e2e", "deterministic-ui");
      style.textContent = [
        "* { animation: none !important; transition: none !important; }",
        "html, body { scroll-behavior: auto !important; }",
        '[data-seshat-e2e-overlay="rebuild"] { pointer-events: none !important; opacity: 0 !important; visibility: hidden !important; }',
      ].join("\n");
      document.head.append(style);

      const neutralizeRebuildOverlay = () => {
        const heading = Array.from(document.querySelectorAll("h1, h2, h3, h4"))
          .find((node) => node.textContent?.trim() === "Your app is being rebuilt.");

        if (!heading) {
          return;
        }

        let root = heading.parentElement;
        while (root?.parentElement && root.parentElement !== document.body) {
          root = root.parentElement;
        }

        if (root instanceof HTMLElement) {
          root.dataset.seshatE2eOverlay = "rebuild";
        }
      };

      neutralizeRebuildOverlay();
      const observer = new MutationObserver(() => {
        neutralizeRebuildOverlay();
      });
      observer.observe(document.documentElement, { childList: true, subtree: true });
    }),
  );
}

export async function waitForNoRebuildOverlay(page: Page) {
  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const headings = Array.from(document.querySelectorAll("h1, h2, h3, h4"));
          return headings.some((heading) => {
            if (heading.textContent?.trim() !== "Your app is being rebuilt.") {
              return false;
            }
            const style = window.getComputedStyle(heading);
            return style.display !== "none" && style.visibility !== "hidden" && style.opacity !== "0";
          });
        }),
      { timeout: 60_000 },
    )
    .toBe(false);
}

export function trapPageErrors(page: Page) {
  const errors: string[] = [];
  page.on("pageerror", (err) => {
    errors.push(err.message);
  });
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      errors.push(msg.text());
    }
  });
  page.on("requestfailed", (request) => {
    const failure = request.failure();
    const reason = failure?.errorText ?? "request failed";
    errors.push(`${request.method()} ${request.url()} :: ${reason}`);
  });
  return errors;
}

export async function mountScrollableHarness(page: Page) {
  await runEffect(() =>
    page.evaluate(() => {
      if (document.getElementById("e2e-scroll-shell")) {
        return;
      }

      const appRoot = document.body.firstElementChild as HTMLElement | null;
      if (!appRoot) {
        throw new Error("unable to mount scroll harness: missing app root");
      }

      const shell = document.createElement("div");
      shell.id = "e2e-scroll-shell";
      shell.style.position = "fixed";
      shell.style.inset = "0";
      shell.style.overflow = "auto";

      const pad = document.createElement("div");
      pad.id = "e2e-scroll-pad";
      pad.style.position = "relative";
      pad.style.width = "2800px";
      pad.style.height = "2200px";
      pad.style.padding = "260px 320px";
      pad.style.boxSizing = "border-box";

      document.body.style.margin = "0";
      document.body.innerHTML = "";
      pad.append(appRoot);
      shell.append(pad);
      document.body.append(shell);
    }),
  );
}

export async function scrollHarnessTo(page: Page, left: number, top: number) {
  await runEffect(() =>
    page.evaluate(
      ({ x, y }) => {
        const shell = document.getElementById("e2e-scroll-shell") as HTMLElement | null;
        if (!shell) {
          throw new Error("scroll harness not mounted");
        }
        shell.scrollTo({ left: x, top: y, behavior: "auto" });
      },
      { x: left, y: top },
    ),
  );
}

export async function mountPageScrollHarness(page: Page) {
  await runEffect(() =>
    page.evaluate(() => {
      if (document.getElementById("e2e-page-scroll-shell")) {
        return;
      }

      const appRoot = document.body.firstElementChild as HTMLElement | null;
      if (!appRoot) {
        throw new Error("unable to mount page scroll harness: missing app root");
      }

      const shell = document.createElement("div");
      shell.id = "e2e-page-scroll-shell";
      shell.style.position = "relative";
      shell.style.minHeight = "3200px";
      shell.style.paddingTop = "480px";
      shell.style.paddingLeft = "180px";
      shell.style.paddingRight = "180px";
      shell.style.boxSizing = "border-box";

      const appFrame = document.createElement("div");
      appFrame.id = "e2e-page-scroll-app-frame";
      appFrame.style.height = "min(840px, calc(100vh - 120px))";
      appFrame.style.width = "min(1480px, calc(100vw - 220px))";

      document.body.style.margin = "0";
      document.body.innerHTML = "";
      appFrame.append(appRoot);
      shell.append(appFrame);
      document.body.append(shell);
    }),
  );
}

export async function scrollPageTo(page: Page, x: number, y: number) {
  await runEffect(() =>
    page.evaluate(
      ({ scrollX, scrollY }) => {
        window.scrollTo({ left: scrollX, top: scrollY, behavior: "auto" });
      },
      { scrollX: x, scrollY: y },
    ),
  );
}

/**
 * Wait for the Dioxus WASM app to signal that the e2e reset hook is ready.
 * This polls `window.__seshatE2eReady` until it becomes `true`.
 */
export async function waitForE2eReady(page: Page) {
  await expect
    .poll(
      () =>
        page.evaluate(
          () => (window as { __seshatE2eReady?: boolean }).__seshatE2eReady === true,
        ),
      { timeout: E2E_READY_TIMEOUT, intervals: [E2E_READY_POLL_MS] },
    )
    .toBe(true);
}

/**
 * Reset the Dioxus document state to defaults without a page reload.
 *
 * Calls `window.__seshatResetDocument()` which is registered by the Rust
 * `use_e2e_reset_hook`. The call returns a Promise that resolves once
 * all signals have been `.set()` back to their defaults on the Rust side.
 */
export async function resetDocument(page: Page) {
  await runEffect(() =>
    page.evaluate(async () => {
      const win = window as {
        __seshatResetDocument?: () => Promise<void>;
      };
      if (typeof win.__seshatResetDocument === "function") {
        await win.__seshatResetDocument();
      }
    }),
  );
}

/**
 * Verify the app is in a clean state: 0 nodes, 0 edges, 0 selected.
 * Useful as a post-reset sanity check.
 */
export async function waitForCleanState(page: Page) {
  await expect.poll(() => nodeCount(page), { timeout: 10_000 }).toBe(0);
  await expect.poll(() => edgeCount(page), { timeout: 5_000 }).toBe(0);
  await expect.poll(() => selectedCount(page), { timeout: 5_000 }).toBe(0);
}

/**
 * Full fresh-start sequence for a test:
 * 1. Clear browser storage (localStorage, cookies)
 * 2. Navigate to "/"
 * 3. Wait for UI ready + e2e hooks
 * 4. Reset document state (in case prior navigation preserved state)
 * 5. Verify clean state
 */
export async function freshStart(page: Page) {
  await runEffectsSequential([
    // Clear any persisted state from prior runs.
    () => page.context().clearCookies(),
    () => page.evaluate(() => {
      try { localStorage.clear(); } catch (_) { /* noop */ }
      try { sessionStorage.clear(); } catch (_) { /* noop */ }
    }),
    () => page.goto("/", { waitUntil: "load" }),
    () => waitForUiReady(page),
    () => waitForE2eReady(page),
    () => resetDocument(page),
    () => waitForCleanState(page),
  ]);
}

export async function createTextNode(
  page: Page,
  canvas: Locator,
  x: number,
  y: number,
) {
  await runEffectsSequential([
    () => waitForNoRebuildOverlay(page),
    () => page.locator('[data-testid="tool-text"]').first().click(),
    // Allow Dioxus signal propagation for tool mode switch before
    // clicking on the canvas.  Without this, clicks may land while the
    // canvas is still in Select mode.
    () => page.waitForTimeout(60),
  ]);
  const box = await runEffect(() => canvas.boundingBox());
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  await runEffect(() => page.mouse.click(box.x + x, box.y + y));
}

export async function clearCanvasOverlays(page: Page) {
  await runEffect(() => waitForNoRebuildOverlay(page));
  const iconsPanel = page.getByRole("heading", { name: "Diagram Icons" });
  if (await runEffect(() => iconsPanel.isVisible().catch(() => false))) {
    await runEffect(() =>
      page.locator('[data-testid="panel-icons-toggle"]').first().click(),
    );
  }

  const propertiesPanel = page.getByRole("heading", { name: "Properties" });
  if (await runEffect(() => propertiesPanel.isVisible().catch(() => false))) {
    await runEffect(() =>
      page.locator('[data-testid="panel-props-toggle"]').first().click(),
    );
  }
}

export async function nodeCenters(
  canvas: Locator,
): Promise<Array<{ x: number; y: number }>> {
  const boxes = await canvas
    .getByTestId("node")
    .evaluateAll((elements) =>
      elements
        .map((element) => {
          const rect = element.getBoundingClientRect();
          return {
            x: rect.x + rect.width / 2,
            y: rect.y + rect.height / 2,
          };
        })
        .sort((a, b) => a.x - b.x),
    );
  return boxes;
}

export async function nodeFrameByLabel(
  page: Page,
  label: string,
  index = 0,
): Promise<{ x: number; y: number; width: number; height: number }> {
  const frame = await runEffect(() =>
    page
      .getByTestId("node")
      .filter({ hasText: label })
      .nth(index)
      .boundingBox(),
  );

  if (!frame) {
    throw new Error(`missing frame for label: ${label}`);
  }
  return frame;
}

export async function selectedCount(page: Page): Promise<number> {
  return readNumericCounter(page, SELECTOR_COUNTER_SELECTED, /(\d+)\s+selected/);
}

export async function nodeCount(page: Page): Promise<number> {
  return readNumericCounter(page, SELECTOR_COUNTER_NODES, /(\d+)\s+nodes/);
}

export async function edgeCount(page: Page): Promise<number> {
  return readNumericCounter(page, SELECTOR_COUNTER_EDGES, /(\d+)\s+edges/);
}

export async function zoomPercent(page: Page): Promise<number> {
  return runEffect(() =>
    page.evaluate((selectorText) => {
      const el = document.querySelector(selectorText);
      if (!el) {
        return 100;
      }

      const dataZoom = el instanceof HTMLElement ? (el.dataset.zoomPercent?.trim() ?? "") : "";
      if (/^\d+$/.test(dataZoom)) {
        return Number.parseInt(dataZoom, 10);
      }

      const text = el.textContent ?? "";
      const match = text.match(/(\d+)%/);
      const digits = match?.[1] ?? "100";
      const parsed = Number.parseInt(digits, 10);
      return Number.isNaN(parsed) ? 100 : parsed;
    }, SELECTOR_ZOOM_RESET),
  );
}
