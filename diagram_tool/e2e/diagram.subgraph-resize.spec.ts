import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

type Box = { x: number; y: number; width: number; height: number };

async function requireBox(target: Locator): Promise<Box> {
  const box = await runEffect(() => target.boundingBox());
  if (!box) {
    throw new Error("expected bounding box");
  }
  return box;
}

async function nodeFrameByLabel(page: Page, label: string, index = 0): Promise<Box> {
  const frame = await runEffect(() =>
    page.evaluate(
      ({ targetLabel, targetIndex }) => {
        const labels = Array.from(document.querySelectorAll("span"))
          .filter((el) => (el.textContent ?? "").trim() === targetLabel)
          .sort(
            (a, b) =>
              a.getBoundingClientRect().x - b.getBoundingClientRect().x,
          );
        const label = labels[targetIndex];
        if (!label) {
          return null;
        }

        let current: HTMLElement | null = label as HTMLElement;
        while (current) {
          const style = current.getAttribute("style") ?? "";
          if (style.includes("position: absolute") && style.includes("border-radius: 10px")) {
            const rect = current.getBoundingClientRect();
            return {
              x: rect.x,
              y: rect.y,
              width: rect.width,
              height: rect.height,
            };
          }
          current = current.parentElement;
        }

        return null;
      },
      { targetLabel: label, targetIndex: index },
    ),
  );

  if (!frame) {
    throw new Error(`missing frame for label: ${label}`);
  }
  return frame;
}

async function pickSouthEastHandle(canvas: Locator): Promise<Box> {
  const handles = canvas.locator('button[style*="nwse-resize"], button[style*="nesw-resize"]');
  const count = await runEffect(() => handles.count());
  if (count < 1) {
    throw new Error("no resize handles found");
  }

  const boxes: Box[] = [];
  for (let i = 0; i < count; i += 1) {
    const next = await runEffect(() => handles.nth(i).boundingBox());
    if (next) {
      boxes.push(next);
    }
  }

  const best = boxes.sort((a, b) => (a.x + a.y) - (b.x + b.y)).pop();
  if (!best) {
    throw new Error("no visible resize handle boxes");
  }
  return best;
}

async function center(box: Box) {
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

async function selectedCount(page: Page): Promise<number> {
  const text = await runEffect(() =>
    page.evaluate(() => {
      const match = document.body.innerText.match(/(\d+) selected/);
      return match ? match[1] : "0";
    }),
  );
  const parsed = Number.parseInt(text, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

async function setupSubgraphWithNodes(page: Page) {
  await runEffect(() => page.goto("/"));
  await runEffect(() => waitForUiReady(page));
  await runEffect(() => clearCanvasOverlays(page));
  const canvas = page.locator(".canvas-container");

  await runEffect(() => createTextNode(page, canvas, 620, 250));
  await runEffect(() => createTextNode(page, canvas, 760, 330));
  await expect(page.getByText(/2 nodes/)).toBeVisible();

  await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
  const canvasBox = await requireBox(canvas);
  const sx = canvasBox.x + 560;
  const sy = canvasBox.y + 210;
  const ex = canvasBox.x + 860;
  const ey = canvasBox.y + 420;
  await runEffect(() => page.mouse.move(sx, sy));
  await runEffect(() => page.mouse.down());
  await runEffect(() => page.mouse.move(ex, ey, { steps: 8 }));
  await runEffect(() => page.mouse.up());

  await expect(page.getByText(/3 nodes/)).toBeVisible();
  return canvas;
}

test.describe("subgraph proportional resize", () => {
  test("resizing subgraph and contained nodes keeps proportions coherent", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithNodes(page);

    await runEffect(() => page.keyboard.press("Control+a"));
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(3);

    const subBefore = await nodeFrameByLabel(page, "Subgraph");
    const nodeBefore = await nodeFrameByLabel(page, "Text", 0);

    const relXBefore = (nodeBefore.x - subBefore.x) / subBefore.width;
    const relYBefore = (nodeBefore.y - subBefore.y) / subBefore.height;

    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await runEffect(() => page.mouse.move(seCenter.x, seCenter.y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(seCenter.x + 140, seCenter.y + 110, { steps: 8 }));
    await runEffect(() => page.mouse.up());

    const subAfter = await nodeFrameByLabel(page, "Subgraph");
    const nodeAfter = await nodeFrameByLabel(page, "Text", 0);

    expect(subAfter.width).toBeGreaterThan(subBefore.width);
    expect(subAfter.height).toBeGreaterThan(subBefore.height);

    const relXAfter = (nodeAfter.x - subAfter.x) / subAfter.width;
    const relYAfter = (nodeAfter.y - subAfter.y) / subAfter.height;

    expect(Math.abs(relXAfter - relXBefore)).toBeLessThanOrEqual(0.2);
    expect(Math.abs(relYAfter - relYBefore)).toBeLessThanOrEqual(0.2);
    expect(pageErrors).toHaveLength(0);
  });

  test("nested subgraph resize preserves inner/outer proportionality", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    const canvas = await setupSubgraphWithNodes(page);

    await runEffect(() => page.getByRole("button", { name: "Subgraph", exact: true }).click());
    const canvasBox = await requireBox(canvas);
    await runEffect(() => page.mouse.move(canvasBox.x + 640, canvasBox.y + 250));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(canvasBox.x + 800, canvasBox.y + 360, { steps: 8 }));
    await runEffect(() => page.mouse.up());

    await expect(page.getByText(/4 nodes/)).toBeVisible();
    await runEffect(() => page.keyboard.press("Control+a"));
    expect(await selectedCount(page)).toBeGreaterThanOrEqual(4);

    const outerBefore = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerBefore = await nodeFrameByLabel(page, "Subgraph", 1);
    const ratioWBefore = innerBefore.width / outerBefore.width;
    const ratioHBefore = innerBefore.height / outerBefore.height;

    const seHandle = await pickSouthEastHandle(canvas);
    const seCenter = await center(seHandle);
    await runEffect(() => page.mouse.move(seCenter.x, seCenter.y));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(seCenter.x + 160, seCenter.y + 120, { steps: 8 }));
    await runEffect(() => page.mouse.up());

    const outerAfter = await nodeFrameByLabel(page, "Subgraph", 0);
    const innerAfter = await nodeFrameByLabel(page, "Subgraph", 1);
    const ratioWAfter = innerAfter.width / outerAfter.width;
    const ratioHAfter = innerAfter.height / outerAfter.height;

    expect(Math.abs(ratioWAfter - ratioWBefore)).toBeLessThanOrEqual(0.2);
    expect(Math.abs(ratioHAfter - ratioHBefore)).toBeLessThanOrEqual(0.2);
    expect(pageErrors).toHaveLength(0);
  });
});
