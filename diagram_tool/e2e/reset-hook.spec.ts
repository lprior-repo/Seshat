import { expect, test } from "@playwright/test";
import { freshStart, nodeCount, edgeCount, selectedCount } from "./helpers";

test.describe("e2e reset hook @baseline", () => {
  test("freshStart creates clean state", async ({ page }) => {
    await freshStart(page);
    
    // Verify clean state
    expect(await nodeCount(page)).toBe(0);
    expect(await edgeCount(page)).toBe(0);
    expect(await selectedCount(page)).toBe(0);
  });

  test("resetDocument clears added content", async ({ page }) => {
    await freshStart(page);
    
    // Create a node via double-click
    await page.getByTestId("tool-select").click();
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    await page.mouse.dblclick(box!.x + 200, box!.y + 200);
    await page.waitForTimeout(500);
    
    // Verify node was created
    expect(await nodeCount(page)).toBeGreaterThan(0);
    
    // Reset the document
    const { resetDocument, waitForCleanState } = await import("./helpers");
    await resetDocument(page);
    await waitForCleanState(page);
    
    // Verify clean state again
    expect(await nodeCount(page)).toBe(0);
    expect(await edgeCount(page)).toBe(0);
    expect(await selectedCount(page)).toBe(0);
  });
});
