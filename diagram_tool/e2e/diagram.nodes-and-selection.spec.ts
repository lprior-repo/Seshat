import { expect, test } from "@playwright/test";
import { Buffer } from "node:buffer";
import {
  canvas,
  chooseFilesWithFileChooser,
  clearCanvasOverlays,
  createTextNode,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
} from "./helpers";

test.describe("diagram nodes and selection", () => {
  test("creates, selects, and drags text nodes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);

    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 560, 220),
      () => createTextNode(page, diagramCanvas, 780, 320),
    ]);

    await expectNodeCount(page, 2);

    const textNodes = diagramCanvas.getByTestId("node");
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
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 520, 220),
      () => createTextNode(page, diagramCanvas, 760, 220),
    ]);

    const textNodes = diagramCanvas.getByTestId("node");
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
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 560, 220));
    const node = diagramCanvas.getByTestId("node").first();
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
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 560, 220));
    const node = diagramCanvas.getByTestId("node").first();
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
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 520, 220),
      () => createTextNode(page, diagramCanvas, 760, 220),
    ]);

    const textNodes = diagramCanvas.getByTestId("node");
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

  // SEL-001: Click node selects @baseline
  test("click on node selects that node @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 400, 200),
      () => createTextNode(page, diagramCanvas, 600, 200),
    ]);

    await expectNodeCount(page, 2);

    // Click first node - should select it
    const textNodes = diagramCanvas.getByTestId("node");
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    // Click second node - should select only that one (replace selection)
    await runEffect(() => textNodes.nth(1).click());
    await expectSelectedCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-002: Click empty clears selection @baseline
  test("click on empty canvas clears selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const textNodes = diagramCanvas.getByTestId("node");
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    // Activate select tool to ensure click on empty clears selection
    await runEffect(() => page.getByTestId("tool-select").click());

    // Click on empty area of canvas
    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounds missing for empty click test");
    }

    // Click far from the node
    await runEffect(() => page.mouse.click(canvasBox.x + 50, canvasBox.y + 50));
    await expectSelectedCount(page, 0);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-003: Shift-click adds to selection @baseline
  test("shift-click adds unselected node to existing selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 350, 200),
      () => createTextNode(page, diagramCanvas, 550, 200),
      () => createTextNode(page, diagramCanvas, 450, 350),
    ]);

    await expectNodeCount(page, 3);

    const textNodes = diagramCanvas.getByTestId("node");

    // Select first node
    await runEffect(() => textNodes.first().click());
    await expectSelectedCount(page, 1);

    // Shift-click second node to add to selection
    await runEffectsSequential([
      () => page.keyboard.down("Shift"),
      () => textNodes.nth(1).click(),
      () => page.keyboard.up("Shift"),
    ]);
    await expectSelectedCount(page, 2);

    // Shift-click third node to add to selection
    await runEffectsSequential([
      () => page.keyboard.down("Shift"),
      () => textNodes.nth(2).click(),
      () => page.keyboard.up("Shift"),
    ]);
    await expectSelectedCount(page, 3);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-004: Marquee select contains nodes @baseline
  test("marquee drag selects nodes within rectangle @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 400, 200),
      () => createTextNode(page, diagramCanvas, 500, 200),
    ]);

    await expectNodeCount(page, 2);

    // Activate select tool
    await runEffect(() => page.getByTestId("tool-select").click());

    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounds missing for marquee test");
    }

    // Marquee from right-to-left (intersect mode) to select both nodes
    const startX = canvasBox.x + 600;
    const startY = canvasBox.y + 150;
    const endX = canvasBox.x + 350;
    const endY = canvasBox.y + 280;

    await runEffectsSequential([
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(endX, endY, { steps: 10 }),
      () => page.mouse.up(),
    ]);

    await expectSelectedCount(page, 2);
    expect(pageErrors).toHaveLength(0);
  });

  // SEL-005: Marquee direction switches selection mode @baseline
  test("marquee left-to-right requires full containment @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 450, 220));

    const node = diagramCanvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds missing for containment test");
    }

    // Draw left-to-right marquee that only partially overlaps the node
    // This should NOT select (containment mode requires full inclusion)
    const partialLeft = nodeBounds.x + nodeBounds.width - 30;
    const partialRight = nodeBounds.x + nodeBounds.width + 20;
    const partialTop = nodeBounds.y + 10;
    const partialBottom = nodeBounds.y + nodeBounds.height - 10;

    await runEffectsSequential([
      () => page.mouse.move(partialLeft, partialTop),
      () => page.mouse.down(),
      () => page.mouse.move(partialRight, partialBottom, { steps: 5 }),
      () => page.mouse.up(),
    ]);
    await expectSelectedCount(page, 0);

    // Now draw a left-to-right marquee that fully contains the node
    const fullLeft = nodeBounds.x - 10;
    const fullRight = nodeBounds.x + nodeBounds.width + 10;
    const fullTop = nodeBounds.y - 10;
    const fullBottom = nodeBounds.y + nodeBounds.height + 10;

    await runEffectsSequential([
      () => page.mouse.move(fullLeft, fullTop),
      () => page.mouse.down(),
      () => page.mouse.move(fullRight, fullBottom, { steps: 5 }),
      () => page.mouse.up(),
    ]);
    await expectSelectedCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });
});
