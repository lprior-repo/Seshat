import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  page.on('pageerror', error => console.log('BROWSER ERROR:', error.message));
  
  await page.goto('http://127.0.0.1:8084');
  await page.waitForLoadState('networkidle');
  await page.waitForFunction(() => window.__seshatE2eReady === true, {}, { timeout: 10000 });
  await page.evaluate(() => window.__seshatResetDocument());
  
  const canvas = page.locator('[data-testid="canvas-root"]');
  const box = await canvas.boundingBox();
  
  console.log("Creating node 1...");
  await page.getByTestId("tool-text").click();
  await page.mouse.click(box.x + 140, box.y + 220);
  
  console.log("Creating node 2...");
  await page.getByTestId("tool-text").click();
  await page.mouse.click(box.x + 400, box.y + 220);
  
  console.log("Clicking edge tool...");
  await page.getByTestId("tool-edge").click();
  
  const nodes = page.getByTestId("node");
  console.log("Node count:", await nodes.count());
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  console.log("First:", first);
  console.log("Second:", second);
  
  console.log("Creating edge...");
  await page.mouse.click(first.x + first.width / 2, first.y + first.height / 2);
  await page.mouse.click(second.x + second.width / 2, second.y + second.height / 2);
  
  await page.waitForTimeout(500);
  
  console.log("Edges UI count:", await page.getByTestId("counter-edges").textContent());
  
  await browser.close();
})();
