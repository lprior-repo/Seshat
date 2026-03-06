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
  
  await page.waitForTimeout(500);
  
  await page.getByTestId("tool-edge").click();
  
  const nodes = page.locator('[data-testid="node"]');
  console.log("Nodes:", await nodes.count());
  
  const first = await nodes.nth(0).boundingBox();
  const second = await nodes.nth(1).boundingBox();
  
  console.log("Creating edge...");
  const firstX = first.x + first.width / 2;
  const firstY = first.y + first.height / 2;
  const secondX = second.x + second.width / 2;
  const secondY = second.y + second.height / 2;
  
  console.log(firstX, firstY, secondX, secondY);
  
  // Directly call the rust function from window pointerdown
  await page.evaluate(({x, y}) => {
     window.dispatchEvent(new PointerEvent('pointerdown', { clientX: x, clientY: y, button: 0, buttons: 1, bubbles: true, cancelable: true }));
     window.dispatchEvent(new PointerEvent('pointerup', { clientX: x, clientY: y, button: 0, buttons: 0, bubbles: true, cancelable: true }));
  }, {x: firstX, y: firstY});
  
  await page.waitForTimeout(100);
  
  await page.evaluate(({x, y}) => {
     window.dispatchEvent(new PointerEvent('pointerdown', { clientX: x, clientY: y, button: 0, buttons: 1, bubbles: true, cancelable: true }));
     window.dispatchEvent(new PointerEvent('pointerup', { clientX: x, clientY: y, button: 0, buttons: 0, bubbles: true, cancelable: true }));
  }, {x: secondX, y: secondY});
  
  await page.waitForTimeout(500);
  
  console.log("Edges UI count:", await page.getByTestId("counter-edges").textContent());
  
  await browser.close();
})();
