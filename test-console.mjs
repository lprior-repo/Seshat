import { chromium } from 'playwright';
import fs from 'fs';

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();

  page.on('console', msg => {
    console.log(`[CONSOLE]:`, msg.text());
  });

  await page.goto('http://127.0.0.1:8081');
  await page.waitForTimeout(3000);
  
  const payload = fs.readFileSync('diagram_tool/e2e/scenes/scene_mixed_selection_v1.json', 'utf8');
  await page.evaluate((jsonPayload) => {
    window.__SESHAT_E2E_IMPORT_JSON = jsonPayload;
    const btn = document.querySelector('[data-testid="toolbar-open"]');
    if (btn) btn.click();
  }, payload);
  await page.waitForTimeout(1000);

  const canvas = page.getByTestId("canvas-root");
  const box = await canvas.boundingBox();
  
  await page.getByTestId("node").first().click();
  await page.getByTestId("toolbar-delete").click();

  await page.waitForTimeout(500);

  const validateBtn = page.getByTestId("toolbar-validate");
  await validateBtn.click();
  await page.waitForTimeout(500);

  const panelHtml = await page.evaluate(() => document.querySelector('[data-testid="validation-panel"]')?.innerHTML);
  console.log('Validation panel HTML:', panelHtml);
  
  const statusText = await page.evaluate(() => document.querySelector('[data-testid="validation-status"]')?.textContent);
  console.log('Validation status:', statusText);

  await browser.close();
})();
