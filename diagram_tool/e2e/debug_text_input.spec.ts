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

  // Use viewport center - canvas element may be taller than viewport
  const viewport = page.viewportSize() ?? { width: 1280, height: 900 };
  const toolbarHeight = 56;
  const targetX = viewport.width / 2;
  const targetY = toolbarHeight + (viewport.height - toolbarHeight) / 2;
  console.log("Viewport:", JSON.stringify(viewport), "Double-clicking at:", targetX, targetY);
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
  expect(labelBox!.y).toBeLessThan(viewport.height); // label is in viewport

  // Click on node BODY (above label) to select, then double-click body to edit
  const nodeEl2 = page.locator('[data-testid="node"]').first();
  const nodeBox2 = await nodeEl2.boundingBox();
  console.log("Node body box:", JSON.stringify(nodeBox2));
  const nx = nodeBox2 ? nodeBox2.x + nodeBox2.width / 2 : labelBox!.x + labelBox!.width / 2;
  const ny = nodeBox2 ? nodeBox2.y + nodeBox2.height / 2 : labelBox!.y - 20;
  console.log("Clicking node body at:", nx, ny);
  await page.mouse.click(nx, ny);
  await page.waitForTimeout(300);

  console.log("Double-clicking node body at:", nx, ny);
  await page.mouse.dblclick(nx, ny);
  await page.waitForTimeout(600);

  await page.screenshot({ path: "/tmp/after_dblclick.png" });

  const inlineEdit = page.locator('[data-testid="inline-edit-input"]');
  await expect(inlineEdit).toBeVisible({ timeout: 3000 });
  console.log("Inline edit visible ✓");

  // InlineEdit uses use_effect to focus via JS on mount - no manual focus needed
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
