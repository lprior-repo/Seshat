const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
  page.on('pageerror', error => console.log('BROWSER ERROR:', error.message));
  page.on('requestfailed', request => console.log('REQUEST FAILED:', request.url(), request.failure().errorText));

  console.log('Navigating to http://localhost:8081...');
  try {
    await page.goto('http://localhost:8081');
    console.log('Page loaded!');
    await page.waitForTimeout(5000);
  } catch (e) {
    console.error('Failed to navigate:', e);
  }
  
  await browser.close();
})();
