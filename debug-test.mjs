import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  
  await page.goto('http://127.0.0.1:8084');
  await page.waitForLoadState('networkidle');
  
  console.log("Waiting for window.__seshatE2eReady...");
  await page.waitForFunction(() => window.__seshatE2eReady === true, {}, { timeout: 10000 });
  
  console.log("Creating nodes...");
  await page.evaluate(() => window.__seshatE2eHooks.createNode(140, 220));
  await page.waitForTimeout(500);
  await page.evaluate(() => window.__seshatE2eHooks.createNode(400, 220));
  await page.waitForTimeout(500);
  
  console.log("Clicking tool edge...");
  await page.getByTestId("tool-edge").click();
  
  const nodes = page.getByTestId("node");
  console.log("Nodes count:", await nodes.count());
  
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  console.log("First:", first);
  console.log("Second:", second);
  
  console.log("Clicking centers...");
  await page.mouse.click(first.x + first.width / 2, first.y + first.height / 2);
  await page.mouse.click(second.x + second.width / 2, second.y + second.height / 2);
  
  await page.waitForTimeout(1000);
  
  const edgesCount = await page.evaluate(() => window.__seshatE2eHooks.edgeCount());
  console.log("Edges count via hook:", edgesCount);
  const selectedCount = await page.evaluate(() => window.__seshatE2eHooks.selectedCount());
  console.log("Selected count via hook:", selectedCount);
  
  await browser.close();
})();
