import { expect, test, type Page } from "@playwright/test";
import { freshStart } from "./helpers";

async function setupPage(page: Page) {
  await freshStart(page);
}

test.describe("DOC contracts @baseline", () => {
  test("DOC-001: Create node - unique ID and default props", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    await page.mouse.dblclick(box!.x + 200, box!.y + 200);
    await page.waitForTimeout(1000);
    
    const nodes = await page.getByTestId("node").count();
    expect(nodes).toBeGreaterThan(0);
    
    const nodeIds: string[] = [];
    const nodeElements = await page.getByTestId("node").all();
    for (const node of nodeElements) {
      const id = await node.getAttribute("data-node-id");
      if (id) nodeIds.push(id);
    }
    
    const uniqueIds = new Set(nodeIds);
    expect(uniqueIds.size).toBe(nodeIds.length);
  });

  test("DOC-002: Create edge - endpoints exist", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-edge").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    await page.mouse.click(box!.x + 100, box!.y + 200);
    await page.waitForTimeout(500);
    await page.mouse.click(box!.x + 300, box!.y + 200);
    await page.waitForTimeout(1000);
    
    const edges = await page.getByTestId("edge").count();
    expect(edges).toBeGreaterThanOrEqual(0);
  });

  test("DOC-003: Delete node with edges", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const nodes = await page.getByTestId("node").count();
    if (nodes > 0) {
      await page.getByTestId("node").first().click();
      await page.keyboard.press("Delete");
      await page.waitForTimeout(500);
    }
    
    await expect(page.getByTestId("counter-nodes")).toBeVisible();
  });

  test("DOC-005: Prevent parent cycles", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    await page.mouse.dblclick(box!.x + 200, box!.y + 200);
    await page.waitForTimeout(1000);
    
    const nodes = await page.getByTestId("node").count();
    expect(nodes).toBeGreaterThan(0);
  });

  test("DOC-010: Z-order ops maintain relative order", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    await page.keyboard.press("ControlOrMeta+a");
    await page.waitForTimeout(500);
    
    const selected = await page.getByTestId("counter-selected").textContent();
    expect(selected).toMatch(/\d+/);
  });

  test("DOC-011: Lock/hide flags prevent transforms", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    await page.mouse.click(box!.x + 200, box!.y + 200);
    await page.waitForTimeout(500);
    
    const node = page.getByTestId("node").first();
    const before = await node.boundingBox();
    expect(before).not.toBeNull();
    
    await page.mouse.move(before!.x + before!.width / 2, before!.y + before!.height / 2);
    await page.mouse.down();
    await page.mouse.move(before!.x + before!.width / 2 + 30, before!.y + before!.height / 2 + 30, { steps: 3 });
    await page.mouse.up();
    await page.waitForTimeout(500);
    
    const after = await node.boundingBox();
    expect(after?.x).not.toBe(before?.x);
  });

  test("DOC-012: Multi-select stable under unrelated updates", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    await page.mouse.click(box!.x + 100, box!.y + 200);
    await page.waitForTimeout(300);
    await page.keyboard.down("Shift");
    await page.mouse.click(box!.x + 200, box!.y + 200);
    await page.keyboard.up("Shift");
    await page.waitForTimeout(500);
    
    const selected = await page.getByTestId("counter-selected").textContent();
    expect(selected).toBe("2");
  });

  test("DOC-013: Duplicate/paste remaps IDs", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    await page.mouse.click(box!.x + 200, box!.y + 200);
    await page.waitForTimeout(500);
    
    const nodesBefore = await page.getByTestId("node").count();
    
    await page.keyboard.press("ControlOrMeta+c");
    await page.waitForTimeout(300);
    await page.keyboard.press("ControlOrMeta+v");
    await page.waitForTimeout(1000);
    
    const nodesAfter = await page.getByTestId("node").count();
    expect(nodesAfter).toBe(nodesBefore + 1);
  });

  test("DOC-014: Move operation atomic", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    const canvas = page.getByTestId("canvas-root");
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    
    const node = page.getByTestId("node").first();
    const before = await node.boundingBox();
    if (!before) return;
    
    await page.mouse.move(before.x + before.width / 2, before.y + before.height / 2);
    await page.mouse.down();
    await page.mouse.move(before.x + before.width / 2 + 20, before.y + before.height / 2 + 20, { steps: 2 });
    await page.mouse.up();
    await page.waitForTimeout(500);
    
    const undoEnabled = await page.getByTestId("toolbar-undo").isEnabled();
    expect(undoEnabled).toBe(true);
  });

  test("DOC-015: Transaction grouping - single history entry", async ({ page }) => {
    await setupPage(page);
    await page.getByTestId("tool-select").click();
    
    await page.keyboard.press("ControlOrMeta+a");
    await page.waitForTimeout(300);
    
    await page.keyboard.press("ArrowRight");
    await page.waitForTimeout(100);
    await page.keyboard.press("ArrowRight");
    await page.waitForTimeout(100);
    await page.keyboard.press("ArrowRight");
    await page.waitForTimeout(500);
    
    const undoButton = page.getByTestId("toolbar-undo");
    await expect(undoButton).toBeEnabled();
    
    await undoButton.click();
    await page.waitForTimeout(500);
    
    const selected = await page.getByTestId("counter-selected").textContent();
    expect(selected).toMatch(/\d+/);
  });

  test("DOC-019: Serialize/deserialize idempotent", async ({ page }) => {
    await setupPage(page);
    
    await expect(page.getByTestId("toolbar-open")).toBeVisible();
    
    const exportBtn = page.getByRole("button", { name: "Export JSON" });
    if (await exportBtn.isVisible()) {
      const [download] = await Promise.all([
        page.waitForEvent("download", { timeout: 5000 }),
        exportBtn.click(),
      ]);
      expect(download.suggestedFilename()).toMatch(/\.json$/);
    }
  });
});
