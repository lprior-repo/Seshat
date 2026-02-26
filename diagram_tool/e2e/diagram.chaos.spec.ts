import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  nodeCenters,
  runEffect,
  trapPageErrors,
  waitForNoRebuildOverlay,
  waitForUiReady,
} from "./helpers";

type Counters = {
  nodes: number;
  edges: number;
  selected: number;
};

class Lcg {
  private readonly state: { value: number };

  constructor(seed: number) {
    this.state = { value: seed >>> 0 };
  }

  next(): number {
    this.state.value = (this.state.value * 1664525 + 1013904223) >>> 0;
    return this.state.value / 4294967296;
  }

  pick(maxExclusive: number): number {
    return Math.floor(this.next() * Math.max(1, maxExclusive));
  }
}

async function readCount(page: Page, label: "nodes" | "edges" | "selected"): Promise<number> {
  const text = await runEffect(() =>
    page.evaluate((targetLabel) => {
      const pattern = new RegExp(`(\\d+) ${targetLabel}`);
      const body = document.body.innerText;
      const matched = body.match(pattern);
      return matched ? matched[0] : `0 ${targetLabel}`;
    }, label),
  );
  const parsed = Number.parseInt(text, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

async function readCounters(page: Page): Promise<Counters> {
  const [nodes, edges, selected] = await Promise.all([
    readCount(page, "nodes"),
    readCount(page, "edges"),
    readCount(page, "selected"),
  ]);
  return { nodes, edges, selected };
}

async function dragRandomNode(page: Page, canvas: Locator, rng: Lcg) {
  const nodes = canvas.getByText("Text", { exact: true });
  const count = await runEffect(() => nodes.count());
  if (count < 1) {
    return;
  }
  const index = rng.pick(count);
  const target = nodes.nth(index);
  const box = await runEffect(() => target.boundingBox());
  if (!box) {
    return;
  }

  const dx = (rng.next() - 0.5) * 140;
  const dy = (rng.next() - 0.5) * 100;
  await runEffect(() => page.mouse.move(box.x + box.width / 2, box.y + box.height / 2));
  await runEffect(() => page.mouse.down());
  await runEffect(() =>
    page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 5 }),
  );
  await runEffect(() => page.mouse.up());
}

async function createTextNodeSafe(page: Page, canvas: Locator, x: number, y: number) {
  await clickFirstVisibleButton(page, "Text");
  const box = await runEffect(() => canvas.boundingBox().catch(() => null));
  if (!box) {
    return;
  }
  await runEffect(() => page.mouse.click(box.x + x, box.y + y));
}

async function recoverFromRebuildOverlay(page: Page) {
  const rebuilding = page.getByRole("heading", { name: "Your app is being rebuilt." });
  const visible = await runEffect(() => rebuilding.isVisible().catch(() => false));
  if (!visible) {
    return;
  }

  await runEffect(() => page.reload({ waitUntil: "domcontentloaded", timeout: 5_000 }));
  await runEffect(() => waitForUiReady(page));
  await runEffect(() => clearCanvasOverlays(page));
}

async function createRandomEdge(page: Page, canvas: Locator, rng: Lcg) {
  const centers = await runEffect(() => nodeCenters(canvas));
  if (centers.length < 2) {
    return;
  }

  let a = rng.pick(centers.length);
  let b = rng.pick(centers.length);
  if (a === b) {
    b = (b + 1) % centers.length;
  }

  await clickFirstVisibleButton(page, "Edge");
  await runEffect(() => page.mouse.move(centers[a].x, centers[a].y));
  await runEffect(() => page.mouse.down());
  await runEffect(() => page.mouse.up());
  await runEffect(() => page.mouse.move(centers[b].x, centers[b].y));
  await runEffect(() => page.mouse.down());
  await runEffect(() => page.mouse.up());
  await runEffect(() => page.keyboard.press("Escape"));
}

async function clickFirstVisibleButton(page: Page, label: string) {
  const buttons = page.getByRole("button", { name: label, exact: true });
  const count = await runEffect(() => buttons.count());
  for (let i = 0; i < count; i += 1) {
    const candidate = buttons.nth(i);
    const visible = await runEffect(() => candidate.isVisible().catch(() => false));
    if (visible) {
      await runEffect(() => candidate.click({ timeout: 1_500 }));
      return;
    }
  }
}

async function randomOp(page: Page, canvas: Locator, rng: Lcg) {
  await recoverFromRebuildOverlay(page);
  const counters = await readCounters(page);
  const roll = rng.next();

  if (roll < 0.2) {
    const x = 480 + rng.pick(520);
    const y = 160 + rng.pick(380);
    await createTextNodeSafe(page, canvas, x, y);
    return;
  }

  if (roll < 0.38 && counters.nodes > 0) {
    await dragRandomNode(page, canvas, rng);
    return;
  }

  if (roll < 0.5 && counters.nodes > 1) {
    await createRandomEdge(page, canvas, rng);
    return;
  }

  if (roll < 0.62) {
    const btn = rng.next() > 0.5 ? "+" : "-";
    await clickFirstVisibleButton(page, btn);
    return;
  }

  if (roll < 0.74) {
    const panel = ["Icons", "Props", "Mini", "Valid"][rng.pick(4)];
    await clickFirstVisibleButton(page, panel);
    return;
  }

  if (roll < 0.84) {
    const historyBtn = rng.next() > 0.5 ? "Undo" : "Redo";
    await clickFirstVisibleButton(page, historyBtn);
    return;
  }

  if (roll < 0.92 && counters.selected > 0) {
    await clickFirstVisibleButton(page, "Delete");
    return;
  }

  await runEffect(() => page.keyboard.press("Escape"));
}

async function runSeededChaos(page: Page, seed: number, steps: number) {
  const rng = new Lcg(seed);
  await runEffect(() => page.goto("/"));
  await runEffect(() => waitForUiReady(page));
  await runEffect(() => clearCanvasOverlays(page));

  const canvas = page.locator(".canvas-container");
  await createTextNodeSafe(page, canvas, 540, 220);
  await createTextNodeSafe(page, canvas, 760, 260);
  await createTextNodeSafe(page, canvas, 940, 300);

  for (let i = 0; i < steps; i += 1) {
    if (i % 8 === 0) {
      await runEffect(() => waitForNoRebuildOverlay(page));
    }
    await randomOp(page, canvas, rng);
    const counters = await readCounters(page);
    expect(counters.nodes).toBeGreaterThanOrEqual(0);
    expect(counters.edges).toBeGreaterThanOrEqual(0);
    expect(counters.selected).toBeGreaterThanOrEqual(0);
    expect(counters.selected).toBeLessThanOrEqual(counters.nodes + counters.edges);
  }
}

test.describe("diagram chaos hardening", () => {
  test.describe.configure({ timeout: 60_000 });

  test("survives deterministic mixed-interaction chaos seed 1337", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runSeededChaos(page, 1337, 24);
    await expect(page.locator(".canvas-container")).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("survives deterministic mixed-interaction chaos seed 4242", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runSeededChaos(page, 4242, 24);
    await expect(page.locator(".canvas-container")).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
