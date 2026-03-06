import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  edgeCount,
  freshStart,
  nodeCenters,
  nodeCount,
  runEffectsSequential,
  runEffect,
  selectedCount,
  trapPageErrors,
  waitForNoRebuildOverlay,
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

async function readCounters(page: Page): Promise<Counters> {
  const [nodes, edges, selected] = await Promise.all([
    nodeCount(page),
    edgeCount(page),
    selectedCount(page),
  ]);
  return { nodes, edges, selected };
}

async function dragRandomNode(page: Page, canvas: Locator, rng: Lcg) {
  const nodes = canvas.getByTestId("node");
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
  await runEffectsSequential([
    () => page.mouse.move(box.x + box.width / 2, box.y + box.height / 2),
    () => page.mouse.down(),
    () => page.mouse.move(box.x + box.width / 2 + dx, box.y + box.height / 2 + dy, { steps: 5 }),
    () => page.mouse.up(),
  ]);
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

  await runEffectsSequential([
    () => page.reload({ waitUntil: "domcontentloaded", timeout: 5_000 }),
    () => freshStart(page),
    () => clearCanvasOverlays(page),
  ]);
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
  await runEffectsSequential([
    () => page.mouse.move(centers[a].x, centers[a].y),
    () => page.mouse.down(),
    () => page.mouse.up(),
    () => page.mouse.move(centers[b].x, centers[b].y),
    () => page.mouse.down(),
    () => page.mouse.up(),
    () => page.keyboard.press("Escape"),
  ]);
}

async function clickFirstVisibleButton(page: Page, label: string) {
  const testIdByLabel: Record<string, string> = {
    Select: "tool-select",
    Pan: "tool-pan",
    Edge: "tool-edge",
    Subgraph: "tool-subgraph",
    Text: "tool-text",
    Undo: "toolbar-undo",
    Redo: "toolbar-redo",
    Delete: "toolbar-delete",
    Validate: "toolbar-validate",
    Icons: "panel-icons-toggle",
    Props: "panel-props-toggle",
    Mini: "panel-mini-toggle",
    Valid: "panel-valid-toggle",
    "+": "zoom-in",
    "-": "zoom-out",
  };
  const maybeTestId = testIdByLabel[label];
  if (maybeTestId) {
    const byTestId = page.getByTestId(maybeTestId);
    const visible = await runEffect(() => byTestId.isVisible().catch(() => false));
    if (visible) {
      await runEffect(() => byTestId.click({ timeout: 1_500 }));
      return;
    }
  }

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
  await freshStart(page);
  await clearCanvasOverlays(page);

  const canvasArea = canvas(page);
  await createTextNodeSafe(page, canvasArea, 540, 220);
  await createTextNodeSafe(page, canvasArea, 760, 260);
  await createTextNodeSafe(page, canvasArea, 940, 300);

  for (let i = 0; i < steps; i += 1) {
    if (i % 8 === 0) {
      await runEffect(() => waitForNoRebuildOverlay(page));
    }
    await randomOp(page, canvasArea, rng);
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
    await expect(canvas(page)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("survives deterministic mixed-interaction chaos seed 4242", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runSeededChaos(page, 4242, 24);
    await expect(canvas(page)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
