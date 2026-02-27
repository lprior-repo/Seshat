import { expect, test } from "@playwright/test";
import { Buffer } from "node:buffer";
import {
  canvas,
  chooseFilesWithFileChooser,
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  expectSelectedCount,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram nodes and selection", () => {
  test("creates, selects, and drags text nodes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);

    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 560, 220),
      () => createTextNode(page, diagramCanvas, 780, 320),
    ]);

    await expectNodeCount(page, 2);

    const textNodes = diagramCanvas.getByText("Text", { exact: true });
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    await runEffectsSequential([
      () => page.keyboard.down("Shift"),
      () => textNodes.nth(1).click(),
      () => page.keyboard.up("Shift"),
    ]);
    await expectSelectedCount(page, 2);

    const nodeBoundsBefore = await runEffect(() => textNodes.first().boundingBox());
    if (!nodeBoundsBefore) {
      throw new Error("text node bounds missing before drag");
    }

    await runEffectsSequential([
      () => page.mouse.move(nodeBoundsBefore.x + 6, nodeBoundsBefore.y + 6),
      () => page.mouse.down(),
      () => page.mouse.move(nodeBoundsBefore.x + 56, nodeBoundsBefore.y + 46),
      () => page.mouse.up(),
    ]);

    const nodeBoundsAfter = await runEffect(() => textNodes.first().boundingBox());
    if (!nodeBoundsAfter) {
      throw new Error("text node bounds missing after drag");
    }

    expect(nodeBoundsAfter.x).toBeGreaterThan(nodeBoundsBefore.x + 20);
    expect(nodeBoundsAfter.y).toBeGreaterThan(nodeBoundsBefore.y + 20);
    expect(pageErrors).toHaveLength(0);
  });

  test("control-or-meta click toggles additive selection parity @baseline", async ({ page }) => {
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 520, 220),
      () => createTextNode(page, diagramCanvas, 760, 220),
    ]);

    const textNodes = diagramCanvas.getByText("Text", { exact: true });
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    await runEffect(() =>
      textNodes.nth(1).click({ modifiers: ["ControlOrMeta"] }),
    );
    await expectSelectedCount(page, 2);

    await runEffect(() =>
      textNodes.nth(1).click({ modifiers: ["ControlOrMeta"] }),
    );
    await expectSelectedCount(page, 1);
  });

  test("no-op marquee does not clear existing selection @baseline", async ({ page }) => {
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 560, 220));
    const node = diagramCanvas.getByText("Text", { exact: true }).first();
    await runEffect(() => node.click());
    await expectSelectedCount(page, 1);

    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounds missing for marquee no-op test");
    }

    const startX = canvasBox.x + 40;
    const startY = canvasBox.y + 40;
    await runEffectsSequential([
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.up(),
    ]);

    await expectSelectedCount(page, 1);
  });

  test("marquee drag direction deterministically switches contain vs intersect", async ({ page }) => {
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 560, 220));
    const node = diagramCanvas.getByText("Text", { exact: true }).first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("text node bounds missing for marquee direction test");
    }

    const top = nodeBounds.y + 4;
    const bottom = nodeBounds.y + nodeBounds.height - 4;
    const leftEdge = nodeBounds.x + 12;
    const rightEdge = nodeBounds.x + 48;

    await runEffectsSequential([
      () => page.mouse.move(leftEdge, top),
      () => page.mouse.down(),
      () => page.mouse.move(rightEdge, bottom, { steps: 5 }),
      () => page.mouse.up(),
    ]);
    await expectSelectedCount(page, 0);

    await runEffectsSequential([
      () => page.mouse.move(rightEdge, bottom),
      () => page.mouse.down(),
      () => page.mouse.move(leftEdge, top, { steps: 5 }),
      () => page.mouse.up(),
    ]);
    await expectSelectedCount(page, 1);
  });

  test("selection survives failed import attempt @baseline", async ({ page }) => {
    await runEffectsSequential([
      () => page.goto("/"),
      () => waitForUiReady(page),
      () => clearCanvasOverlays(page),
    ]);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 520, 220),
      () => createTextNode(page, diagramCanvas, 760, 220),
    ]);

    const textNodes = diagramCanvas.getByText("Text", { exact: true });
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    await runEffect(() =>
      chooseFilesWithFileChooser(page, () => page.getByTestId("toolbar-open").click(), [
        {
          name: "broken-selection.json",
          mimeType: "application/json",
          buffer: Buffer.from("{not valid json"),
        },
      ]),
    );

    await expect(page.getByText("Load failed", { exact: true })).toBeVisible();
    await expectSelectedCount(page, 1);
  });
});
