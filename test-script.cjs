const { chromium } = require('playwright');
const assert = require('assert');

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  page.on('pageerror', error => console.log('BROWSER ERROR:', error.message));
  
  console.log("Navigating...");
  await page.goto('http://127.0.0.1:8084');
  console.log("Waiting for network idle...");
  await page.waitForLoadState('networkidle');
  
  console.log("Waiting for window.__seshatE2eReady...");
  await page.waitForFunction('window.__seshatE2eReady === true', {}, { timeout: 10000 });
  
  console.log("Running reset hook...");
  await page.evaluate('window.__seshatE2eHooks.reset()');
  await page.evaluate('window.__seshatE2eHooks.clearOverlays()');
  
  console.log("Creating nodes...");
  
  const canvas = page.getByTestId("canvas-root");
  const canvasBox = await canvas.boundingBox();
  
  // Use dispatchEvent to click tool to avoid auto-scroll side effects
  await page.locator('[data-testid="tool-text"]').first().dispatchEvent('click');
  await page.waitForTimeout(100);
  
  // Target position is offset to mimic test conditions
  const firstTargetX = canvasBox.x + 360;
  const firstTargetY = canvasBox.y + 230;
  
  await page.mouse.click(firstTargetX, firstTargetY);
  await page.waitForTimeout(500);
  
  // Scroll document down 
  console.log("Scrolling document down...");
  await page.evaluate(() => window.scrollTo(0, 320));
  await page.waitForTimeout(500);
  
  const scrolledBox = await canvas.boundingBox();
  
  // Click again using offset that compensates for new bounding box, simulating absolute screen click 
  await page.locator('[data-testid="tool-text"]').first().dispatchEvent('click');
  await page.waitForTimeout(100);
  
  const secondTargetX = scrolledBox.x + 360;
  const secondTargetY = scrolledBox.y + 230;
  await page.mouse.click(secondTargetX, secondTargetY);
  await page.waitForTimeout(500);
  
  const nodes = page.locator('[data-testid="node"]');
  console.log("Node count:", await nodes.count());
  
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  console.log("First node:", first);
  console.log("Second node:", second);
  
  const driftY = Math.abs(second.y - first.y);
  console.log("Y drift between node 1 and 2:", driftY);
  
  await browser.close();
})();
