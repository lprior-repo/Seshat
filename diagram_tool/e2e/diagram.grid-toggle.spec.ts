import { expect, test, type Locator, type Page } from "@playwright/test";
import {
  canvas,
  clearCanvasOverlays,
  freshStart,
} from "./helpers";

async function setupFreshPage(page: Page): Promise<Locator> {
  await freshStart(page);
  await clearCanvasOverlays(page);
  return canvas(page);
}

test.describe("grid visibility toggle", () => {
  test("grid toggle button exists in toolbar @baseline", async ({ page }) => {
    await setupFreshPage(page);
    
    // The grid toggle should exist in the toolbar
    const gridToggle = page.getByTestId("grid-toggle");
    await expect(gridToggle).toBeVisible();
  });

  test("grid is visible by default @baseline", async ({ page }) => {
    await setupFreshPage(page);
    
    // The grid should be visible by default (show_grid defaults to true)
    const gridToggle = page.getByTestId("grid-toggle");
    await expect(gridToggle).toHaveAttribute("data-checked", "true");
  });

  test("clicking grid toggle hides grid @behavior", async ({ page }) => {
    await setupFreshPage(page);
    
    const gridToggle = page.getByTestId("grid-toggle");
    
    // Click to hide grid
    await gridToggle.click();
    
    // Toggle should now show unchecked state
    await expect(gridToggle).toHaveAttribute("data-checked", "false");
  });

  test("clicking grid toggle twice shows grid again @behavior", async ({ page }) => {
    await setupFreshPage(page);
    
    const gridToggle = page.getByTestId("grid-toggle");
    
    // Click to hide grid
    await gridToggle.click();
    await expect(gridToggle).toHaveAttribute("data-checked", "false");
    
    // Click again to show grid
    await gridToggle.click();
    await expect(gridToggle).toHaveAttribute("data-checked", "true");
  });
});
