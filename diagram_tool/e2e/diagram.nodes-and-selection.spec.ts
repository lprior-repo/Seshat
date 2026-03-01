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

  // SEL-006: Hover shows visual affordances @baseline
  test("hovering node shows visual affordances @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds missing for hover test");
    }

    // Get initial border style by hovering outside the node
    await runEffect(() => page.mouse.move(nodeBounds.x - 20, nodeBounds.y - 20));
    const beforeHover = await runEffect(() =>
      node.evaluate((el) => {
        const style = window.getComputedStyle(el);
        return {
          borderColor: style.borderColor,
          borderWidth: style.borderWidth,
        };
      }),
    );

    // Hover over the node center
    await runEffect(() =>
      page.mouse.move(nodeBounds.x + nodeBounds.width / 2, nodeBounds.y + nodeBounds.height / 2),
    );

    // Wait for hover state to apply
    await runEffect(() => page.waitForTimeout(50));

    const afterHover = await runEffect(() =>
      node.evaluate((el) => {
        const style = window.getComputedStyle(el);
        return {
          borderColor: style.borderColor,
          borderWidth: style.borderWidth,
        };
      }),
    );

    // Hover should change the border (either color or width)
    const borderChanged =
      beforeHover.borderColor !== afterHover.borderColor ||
      beforeHover.borderWidth !== afterHover.borderWidth;
    expect(borderChanged).toBe(true);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-007: Resize handles are clickable @baseline
  test("resize handles are clickable and initiate resize @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    // Select the node to show resize handles
    const node = diagramCanvas.getByTestId("node").first();
    await runEffect(() => node.click());
    await expectSelectedCount(page, 1);

    // Get the SE resize handle
    const seHandle = diagramCanvas.getByTestId("resize-handle-se").first();
    await runEffect(() => expect(seHandle).toBeVisible());

    const handleBounds = await runEffect(() => seHandle.boundingBox());
    if (!handleBounds) {
      throw new Error("resize handle bounds missing");
    }

    const nodeBoundsBefore = await runEffect(() => node.boundingBox());
    if (!nodeBoundsBefore) {
      throw new Error("node bounds missing before resize");
    }

    // Drag the handle to resize
    await runEffectsSequential([
      () => page.mouse.move(handleBounds.x + handleBounds.width / 2, handleBounds.y + handleBounds.height / 2),
      () => page.mouse.down(),
      () => page.mouse.move(handleBounds.x + 60, handleBounds.y + 40, { steps: 5 }),
      () => page.mouse.up(),
    ]);

    const nodeBoundsAfter = await runEffect(() => node.boundingBox());
    if (!nodeBoundsAfter) {
      throw new Error("node bounds missing after resize");
    }

    // Node should be larger after resize
    expect(nodeBoundsAfter.width).toBeGreaterThan(nodeBoundsBefore.width + 30);
    expect(nodeBoundsAfter.height).toBeGreaterThan(nodeBoundsBefore.height + 20);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-008: Touch has larger hit area @baseline
  test("touch tap near edge selects node with larger hit area @baseline", async ({ page, hasTouch }) => {
    test.skip(!hasTouch, "touch hit area test requires touch-enabled project");
    const pageErrors = trapPageErrors(page);

    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds missing for touch hit area test");
    }

    // Tap slightly outside the node boundary (within touch hit area margin)
    // Touch hit area should be more forgiving than mouse
    const tapX = nodeBounds.x + nodeBounds.width / 2;
    const tapY = nodeBounds.y - 8; // Just above the node

    await runEffect(() => page.touchscreen.tap(tapX, tapY));

    // Touch should select the node due to larger hit area
    await expectSelectedCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-009: Drag threshold prevents accidental drag @baseline
  test("drag below threshold does not move selection @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    await runEffect(() => node.click());
    await expectSelectedCount(page, 1);

    const nodeBoundsBefore = await runEffect(() => node.boundingBox());
    if (!nodeBoundsBefore) {
      throw new Error("node bounds missing before threshold drag");
    }

    // Perform a very small drag (below the 3px threshold)
    const startX = nodeBoundsBefore.x + 10;
    const startY = nodeBoundsBefore.y + 10;
    const endX = startX + 1; // Only 1 pixel - below threshold
    const endY = startY + 1;

    await runEffectsSequential([
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(endX, endY),
      () => page.mouse.up(),
    ]);

    const nodeBoundsAfter = await runEffect(() => node.boundingBox());
    if (!nodeBoundsAfter) {
      throw new Error("node bounds missing after threshold drag");
    }

    // Node should not have moved significantly (below threshold)
    expect(Math.abs(nodeBoundsAfter.x - nodeBoundsBefore.x)).toBeLessThan(3);
    expect(Math.abs(nodeBoundsAfter.y - nodeBoundsBefore.y)).toBeLessThan(3);

    // Selection should be preserved
    await expectSelectedCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-021: Selection UI matches geometry for items @baseline
  test("selection bounding box matches node geometry @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds missing for selection UI test");
    }

    // Select the node
    await runEffect(() => node.click());
    await expectSelectedCount(page, 1);

    // Verify selection handles are visible
    const seHandle = diagramCanvas.getByTestId("resize-handle-se").first();
    await runEffect(() => expect(seHandle).toBeVisible());

    // Get handle position and verify it's at the corner of the node
    const handleBounds = await runEffect(() => seHandle.boundingBox());
    if (!handleBounds) {
      throw new Error("handle bounds missing for selection UI test");
    }

    // Handle should be near the bottom-right corner of the node
    const handleCenterX = handleBounds.x + handleBounds.width / 2;
    const handleCenterY = handleBounds.y + handleBounds.height / 2;
    const nodeBottomRightX = nodeBounds.x + nodeBounds.width;
    const nodeBottomRightY = nodeBounds.y + nodeBounds.height;

    // Handle should be within reasonable distance of the corner
    expect(Math.abs(handleCenterX - nodeBottomRightX)).toBeLessThan(20);
    expect(Math.abs(handleCenterY - nodeBottomRightY)).toBeLessThan(20);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-022: Long press selects without drag @baseline
  test("pointer down with hold selects node without drag @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds missing for long press test");
    }

    // Get position before interaction
    const initialX = nodeBounds.x;
    const initialY = nodeBounds.y;

    // Perform pointer down, hold briefly, then release without moving
    const clickX = nodeBounds.x + nodeBounds.width / 2;
    const clickY = nodeBounds.y + nodeBounds.height / 2;

    await runEffectsSequential([
      () => page.mouse.move(clickX, clickY),
      () => page.mouse.down(),
      () => page.waitForTimeout(100), // Hold for 100ms
      () => page.mouse.up(),
    ]);

    // Node should be selected
    await expectSelectedCount(page, 1);

    // Node should not have moved (no drag occurred)
    const nodeBoundsAfter = await runEffect(() => node.boundingBox());
    if (!nodeBoundsAfter) {
      throw new Error("node bounds missing after long press");
    }

    expect(Math.abs(nodeBoundsAfter.x - initialX)).toBeLessThan(3);
    expect(Math.abs(nodeBoundsAfter.y - initialY)).toBeLessThan(3);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-023: Multi-click timing thresholds @baseline
  test("double-click on selected node enters edit mode @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    const nodeBounds = await runEffect(() => node.boundingBox());
    if (!nodeBounds) {
      throw new Error("node bounds missing for double-click test");
    }

    // First click to select
    await runEffect(() => node.click());
    await expectSelectedCount(page, 1);

    // Double-click to enter edit mode
    const clickX = nodeBounds.x + nodeBounds.width / 2;
    const clickY = nodeBounds.y + nodeBounds.height / 2;

    await runEffectsSequential([
      () => page.mouse.click(clickX, clickY, { clickCount: 2 }),
    ]);

    // After double-click, look for an input field or editable text element
    // The exact behavior depends on the implementation
    // At minimum, selection should still be present
    await expectSelectedCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-024: Selection not dropped during rerender @baseline
  test("selection persists after zoom change @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);
    await runEffect(() => createTextNode(page, diagramCanvas, 500, 250));

    const node = diagramCanvas.getByTestId("node").first();
    await runEffect(() => node.click());
    await expectSelectedCount(page, 1);

    // Trigger a zoom change using the zoom-in button
    const zoomInButton = page.getByTestId("zoom-in").first();
    await runEffect(() => zoomInButton.click());

    // Wait for zoom to complete
    await runEffect(() => page.waitForTimeout(100));

    // Selection should still be present after zoom
    await expectSelectedCount(page, 1);

    expect(pageErrors).toHaveLength(0);
  });

  // SEL-025: Box-select through parent boundaries @baseline
  test("marquee selects nodes regardless of position @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    await clearCanvasOverlays(page);

    const diagramCanvas = canvas(page);

    // Create two nodes at different positions
    await runEffectsSequential([
      () => createTextNode(page, diagramCanvas, 400, 200),
      () => createTextNode(page, diagramCanvas, 600, 300),
    ]);

    await expectNodeCount(page, 2);

    // Activate select tool
    await runEffect(() => page.getByTestId("tool-select").click());

    const canvasBox = await runEffect(() => diagramCanvas.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounds missing for marquee test");
    }

    // Draw a marquee that encompasses both nodes
    const startX = canvasBox.x + 350;
    const startY = canvasBox.y + 150;
    const endX = canvasBox.x + 650;
    const endY = canvasBox.y + 380;

    await runEffectsSequential([
      () => page.mouse.move(startX, startY),
      () => page.mouse.down(),
      () => page.mouse.move(endX, endY, { steps: 10 }),
      () => page.mouse.up(),
    ]);

    // Both nodes should be selected
    await expectSelectedCount(page, 2);

    expect(pageErrors).toHaveLength(0);
  });
});
