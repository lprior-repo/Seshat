import { expect, type Locator, type Page } from "@playwright/test";
import { Effect } from "effect";

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
  await expect(page.getByTestId("node-count")).toHaveText(/\d+ nodes/, {
    timeout: 30_000,
  });
  await waitForNoRebuildOverlay(page);
}

export function canvas(page: Page): Locator {
  return page.getByTestId("canvas-container");
}

export async function expectNodeCount(page: Page, count: number) {
  await expect(page.getByTestId("node-count")).toHaveText(`${count} nodes`);
}

export async function expectEdgeCount(page: Page, count: number) {
  await expect(page.getByTestId("edge-count")).toHaveText(`${count} edges`);
}

export async function expectSelectedCount(page: Page, count: number) {
  await expect(page.getByTestId("selected-count")).toHaveText(`${count} selected`);
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
      ].join("\n");
      document.head.append(style);
    }),
  );
}

export async function waitForNoRebuildOverlay(page: Page) {
  const rebuilding = page.getByRole("heading", {
    name: "Your app is being rebuilt.",
  });
  await expect.poll(async () => rebuilding.count(), { timeout: 60_000 }).toBe(0);
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

export async function createTextNode(
  page: Page,
  canvas: Locator,
  x: number,
  y: number,
) {
  await runEffectsSequential([
    () => waitForNoRebuildOverlay(page),
    () => page.getByRole("button", { name: "Text", exact: true }).click(),
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
    await runEffect(() => page.getByRole("button", { name: "Icons", exact: true }).click());
  }

  const propertiesPanel = page.getByRole("heading", { name: "Properties" });
  if (await runEffect(() => propertiesPanel.isVisible().catch(() => false))) {
    await runEffect(() => page.getByRole("button", { name: "Props", exact: true }).click());
  }
}

export async function nodeCenters(
  canvas: Locator,
): Promise<Array<{ x: number; y: number }>> {
  const boxes = await canvas
    .getByTestId("diagram-node")
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
      .getByTestId("diagram-node")
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
  const text = await runEffect(() => page.getByTestId("selected-count").innerText());
  const match = text.match(/(\d+)\s+selected/);
  const digits = match ? match[1] : "0";
  const parsed = Number.parseInt(digits, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export async function nodeCount(page: Page): Promise<number> {
  const text = await runEffect(() => page.getByTestId("node-count").innerText());
  const match = text.match(/(\d+)\s+nodes/);
  const digits = match ? match[1] : "0";
  const parsed = Number.parseInt(digits, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export async function edgeCount(page: Page): Promise<number> {
  const text = await runEffect(() => page.getByTestId("edge-count").innerText());
  const match = text.match(/(\d+)\s+edges/);
  const digits = match ? match[1] : "0";
  const parsed = Number.parseInt(digits, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export async function zoomPercent(page: Page): Promise<number> {
  const text = await runEffect(() => page.getByTestId("zoom-percent").innerText());
  const match = text.match(/(\d+)%/);
  const digits = match ? match[1] : "100";
  const parsed = Number.parseInt(digits, 10);
  return Number.isNaN(parsed) ? 100 : parsed;
}
