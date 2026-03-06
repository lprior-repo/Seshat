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
  
  // Actually the edge tool needs to hit the nodes themselves. Let's make sure nodes are actually rendered!
  await page.getByTestId("tool-node").click(); // contracts.spec.ts uses createNodeAt, which usually creates default nodes
  await page.mouse.click(box.x + 140, box.y + 220);
  await page.mouse.click(box.x + 400, box.y + 220);
  
  await page.waitForTimeout(500);
  
  await page.getByTestId("tool-edge").click();
  
  const nodes = page.getByTestId("node");
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  
  console.log("Creating edge with move->down->up...");
  const firstX = first.x + first.width / 2;
  const firstY = first.y + first.height / 2;
  const secondX = second.x + second.width / 2;
  const secondY = second.y + second.height / 2;
  
  // In the real app, we need to click ON the node. Let's dispatch events to the node element itself or canvas.
  await page.mouse.click(firstX, firstY);
  await page.waitForTimeout(100);
  await page.mouse.click(secondX, secondY);
  
  await page.waitForTimeout(500);
  
  console.log("Edges UI count:", await page.getByTestId("counter-edges").textContent());
  
  await browser.close();
})();
