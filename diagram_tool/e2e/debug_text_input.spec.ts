import { test, expect } from "@playwright/test";

async function waitForApp(page: any) {
  for (let i = 0; i < 40; i++) {
    try {
      await page.goto("http://localhost:3333", { waitUntil: "commit", timeout: 5000 });
      if (!(await page.content()).includes('Dioxus Build')) {
        await page.waitForSelector('[data-testid="canvas-root"]', { timeout: 5000 });
        await page.waitForTimeout(1000);
        return true;
      }
      console.log("Building...", i+1);
      await page.waitForTimeout(4000);
    } catch { await page.waitForTimeout(3000); }
  }
  return false;
}

test("inline edit stores text on Enter @baseline", async ({ page }) => {
  if (!await waitForApp(page)) throw new Error("App never loaded");

  // Reset camera to origin so nodes land near screen center
  await page.locator('[data-testid="zoom-reset"]').first().click();
  await page.waitForTimeout(300);

  // Select tool → double-click on empty canvas creates a node
  await page.locator('[data-testid="tool-select"]').first().click();
  await page.waitForTimeout(200);

  const canvasEl = page.locator('[data-testid="canvas-root"]').first();
  const canvasBox = await canvasEl.boundingBox();
  console.log("Canvas box:", JSON.stringify(canvasBox));
  if (!canvasBox) throw new Error("No canvas box");

  const targetX = canvasBox.x + canvasBox.width / 2;
  const targetY = canvasBox.y + canvasBox.height / 2;
  console.log("Double-clicking canvas at:", targetX, targetY);
  await page.mouse.dblclick(targetX, targetY);
  await page.waitForTimeout(600);

  const nodeCount = await page.locator('[data-testid="node"]').count();
  console.log("Nodes after dblclick:", nodeCount);
  expect(nodeCount).toBeGreaterThan(0);

  // Get node and label position
  const labelEl = page.locator('[data-testid="node-label"]').first();
  await labelEl.waitFor({ state: "visible", timeout: 5000 });
  const labelBox = await labelEl.boundingBox();
  const originalLabel = await labelEl.textContent();
  console.log("Label box:", JSON.stringify(labelBox));
  console.log("Original label:", JSON.stringify(originalLabel));

  expect(labelBox).not.toBeNull();
  expect(labelBox!.y).toBeLessThan(canvasBox.y + canvasBox.height); // label is in viewport

  // Double-click the label to start inline edit
  const lx = labelBox!.x + labelBox!.width / 2;
  const ly = labelBox!.y + labelBox!.height / 2;
  console.log("Double-clicking label at:", lx, ly);
  await page.mouse.dblclick(lx, ly);
  await page.waitForTimeout(400);

  const inlineEdit = page.locator('[data-testid="inline-edit-input"]');
  await expect(inlineEdit).toBeVisible({ timeout: 3000 });
  console.log("Inline edit visible ✓");

  // Should be auto-focused - type without clicking
  await page.keyboard.press('Control+a');
  await page.keyboard.type("TestLabel", { delay: 60 });

  const val = await inlineEdit.inputValue().catch(() => "GONE");
  console.log("Value while typing:", JSON.stringify(val));
  expect(val).toBe("TestLabel");

  // Commit with Enter
  await page.keyboard.press("Enter");
  await page.waitForTimeout(800);

  // Verify label updated in DOM
  const newLabel = await page.locator('[data-testid="node-label"]').first().textContent().catch(() => "gone");
  console.log("New label:", JSON.stringify(newLabel));
  expect(newLabel).toBe("TestLabel");
});
