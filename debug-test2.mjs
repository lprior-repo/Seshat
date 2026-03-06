import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  
  await page.goto('http://127.0.0.1:8084');
  await page.waitForLoadState('networkidle');
  
  console.log("Waiting for app to load...");
  await page.waitForTimeout(5000);
  
  const hooks = await page.evaluate(() => Object.keys(window).filter(k => k.includes("seshat")));
  console.log("Hooks available:", hooks);
  
  await browser.close();
})();
