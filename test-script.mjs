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
  await page.evaluate('window.__seshatE2eHooks.createNode(140, 220)');
  await page.waitForTimeout(100);
  await page.evaluate('window.__seshatE2eHooks.createNode(400, 220)');
  await page.waitForTimeout(100);
  
  console.log("Clicking edge tool...");
  await page.getByTestId("tool-edge").click();
  
  const nodes = page.getByTestId("node");
  console.log("Node count:", await nodes.count());
  
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  console.log("First box:", first);
  console.log("Second box:", second);
  
  console.log("Clicking first node...");
  await page.mouse.click(first.x + first.width / 2, first.y + first.height / 2);
  
  console.log("Clicking second node...");
  await page.mouse.click(second.x + second.width / 2, second.y + second.height / 2);
  
  await page.waitForTimeout(500);
  
  const edgesCounter = await page.getByTestId("counter-edges").textContent();
  console.log("Edges count from UI:", edgesCounter);
  
  await browser.close();
})();
