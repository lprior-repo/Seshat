import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  
  await page.goto('http://127.0.0.1:8084');
  await page.waitForLoadState('networkidle');
  await page.waitForFunction(() => window.__seshatE2eReady === true, {}, { timeout: 10000 });
  await page.evaluate(() => window.__seshatResetDocument());
  
  const canvas = page.locator('[data-testid="canvas-root"]');
  const box = await canvas.boundingBox();
  
  await page.getByTestId("tool-text").click();
  await page.mouse.click(box.x + 140, box.y + 220);
  await page.getByTestId("tool-text").click();
  await page.mouse.click(box.x + 400, box.y + 220);
  
  await page.getByTestId("tool-edge").click();
  
  const nodes = page.getByTestId("node");
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  
  // Try pointer events instead of just mouse.click since some apps use pointerdown/up
  console.log("Simulating pointer events for edge creation...");
  
  // First node
  await page.mouse.move(first.x + first.width / 2, first.y + first.height / 2);
  await page.mouse.down();
  await page.mouse.up();
  
  // Wait a bit
  await page.waitForTimeout(100);
  
  // Second node
  await page.mouse.move(second.x + second.width / 2, second.y + second.height / 2);
  await page.mouse.down();
  await page.mouse.up();
  
  await page.waitForTimeout(500);
  
  console.log("Edges UI count:", await page.getByTestId("counter-edges").textContent());
  
  await browser.close();
})();
