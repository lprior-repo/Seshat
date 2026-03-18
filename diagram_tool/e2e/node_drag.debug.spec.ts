import { expect, test } from "@playwright/test";
import {
  canvas,
  createTextNode,
  expectNodeCount,
  freshStart,
  runEffect,
  trapPageErrors,
} from "./helpers";

test("node can be dragged @baseline", async ({ page }) => {
  const pageErrors = trapPageErrors(page);
  await freshStart(page);

  const canvasArea = canvas(page);
  await createTextNode(page, canvasArea, 200, 200);
  await expectNodeCount(page, 1);

  // Get the initial position of the node
  const node = page.getByTestId("node").first();
  const initialBox = await node.boundingBox();
  expect(initialBox).toBeDefined();

  if (!initialBox) throw new Error("no box");

  // Wait for React/Dioxus to update
  await page.waitForTimeout(500);

  // Get the new position of the node
  const preDragBox = await node.boundingBox();
  expect(preDragBox).toBeDefined();
  
  if (!preDragBox) throw new Error("no preDrag box");

  console.log(`Pre-drag: ${preDragBox.x}, ${preDragBox.y}`);

  // Drag the node by 100px down and right using mouse
  await page.mouse.move(preDragBox.x + 10, preDragBox.y + 10);
  await page.mouse.down();
  await page.mouse.move(preDragBox.x + 60, preDragBox.y + 60, { steps: 5 });
  await page.waitForTimeout(100);
  
  const midDragBox = await node.boundingBox();
  console.log(`Mid-drag (mouse still down): ${midDragBox?.x}, ${midDragBox?.y}`);

  await page.mouse.move(preDragBox.x + 110, preDragBox.y + 110, { steps: 5 });
  await page.waitForTimeout(100);

  const midDragBox2 = await node.boundingBox();
  console.log(`Mid-drag 2 (mouse still down): ${midDragBox2?.x}, ${midDragBox2?.y}`);

  await page.mouse.up();

  // Wait for React/Dioxus to update
  await page.waitForTimeout(500);

  // Get the new position of the node
  const newBox = await node.boundingBox();
  expect(newBox).toBeDefined();
  
  if (!newBox) throw new Error("no new box");

  console.log(`New: ${newBox.x}, ${newBox.y}`);

  expect(newBox.x).toBeGreaterThan(preDragBox.x + 50);
  expect(newBox.y).toBeGreaterThan(preDragBox.y + 50);
  expect(pageErrors).toHaveLength(0);
});
