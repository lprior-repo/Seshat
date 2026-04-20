import { expect, test } from "@playwright/test";
import {
  freshStart,
  runEffect,
  trapPageErrors,
  waitForNoRebuildOverlay,
} from "./helpers";

const THEME_BTN = '[data-testid="theme-toggle-btn"]';
const STORAGE_KEY = "diagram_tool.theme_mode";

const CYCLE = ["System", "Light", "Dark", "White"] as const;

async function getThemeLabel(page: import("@playwright/test").Page): Promise<string> {
  return runEffect(() =>
    page.evaluate((sel) => {
      const el = document.querySelector(sel);
      return el?.textContent?.trim() ?? "";
    }, THEME_BTN),
  );
}

async function clickThemeToggle(page: import("@playwright/test").Page) {
  await runEffect(() => page.locator(THEME_BTN).click());
  await waitForNoRebuildOverlay(page);
}

test.describe("theme toggle @baseline", () => {
  test.describe.configure({ mode: "parallel" });

  test("theme toggle button renders on canvas", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    await expect(page.locator(THEME_BTN)).toBeVisible({ timeout: 10_000 });

    const label = await getThemeLabel(page);
    expect(CYCLE).toContain(label as typeof CYCLE[number]);
    expect(pageErrors).toHaveLength(0);
  });

  test("clicking toggle cycles through all 4 modes", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const firstLabel = await getThemeLabel(page);
    const firstIdx = CYCLE.indexOf(firstLabel as typeof CYCLE[number]);
    expect(firstIdx).toBeGreaterThanOrEqual(0);

    for (let i = 1; i <= 4; i += 1) {
      await clickThemeToggle(page);
      const expected = CYCLE[(firstIdx + i) % CYCLE.length];
      await expect
        .poll(() => getThemeLabel(page), { timeout: 5_000 })
        .toBe(expected);
    }

    await expect
      .poll(() => getThemeLabel(page), { timeout: 5_000 })
      .toBe(firstLabel);
    expect(pageErrors).toHaveLength(0);
  });

  test("persisted theme is loaded on page reload", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    while ((await getThemeLabel(page)) !== "Dark") {
      await clickThemeToggle(page);
    }

    await expect
      .poll(() => getThemeLabel(page), { timeout: 5_000 })
      .toBe("Dark");

    await page.reload({ waitUntil: "domcontentloaded" });
    await waitForNoRebuildOverlay(page);
    await import("./helpers").then(m => m.waitForE2eReady(page));

    await expect
      .poll(() => getThemeLabel(page), { timeout: 10_000 })
      .toBe("Dark");

    const stored = await runEffect(() =>
      page.evaluate((key) => localStorage.getItem(key), STORAGE_KEY),
    );
    expect(stored).toBe("dark");
    expect(pageErrors).toHaveLength(0);
  });

  test("toggle shows correct mode after cycling to each variant", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const firstLabel = await getThemeLabel(page);
    const firstIdx = CYCLE.indexOf(firstLabel as typeof CYCLE[number]);
    expect(firstIdx).toBeGreaterThanOrEqual(0);

    const expectedSequence = CYCLE.map((_, i) => CYCLE[(firstIdx + i) % CYCLE.length]);

    for (let i = 0; i < expectedSequence.length; i += 1) {
      if (i > 0) {
        await clickThemeToggle(page);
      }
      await expect
        .poll(() => getThemeLabel(page), { timeout: 5_000 })
        .toBe(expectedSequence[i]);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("localStorage is updated on each toggle click", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const firstLabel = await getThemeLabel(page);
    const firstIdx = CYCLE.indexOf(firstLabel as typeof CYCLE[number]);
    expect(firstIdx).toBeGreaterThanOrEqual(0);

    for (let i = 1; i <= 4; i += 1) {
      await clickThemeToggle(page);
      const expectedMode = CYCLE[(firstIdx + i) % CYCLE.length]!.toLowerCase();
      await expect
        .poll(
          () =>
            runEffect(() =>
              page.evaluate((key) => localStorage.getItem(key), STORAGE_KEY),
            ),
          { timeout: 5_000 },
        )
        .toBe(expectedMode);
    }

    expect(pageErrors).toHaveLength(0);
  });

  test("survives rapid toggle clicks", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    for (let i = 0; i < 20; i += 1) {
      await runEffect(() => page.locator(THEME_BTN).dispatchEvent("click"));
    }

    await waitForNoRebuildOverlay(page);

    const label = await getThemeLabel(page);
    expect(CYCLE).toContain(label as typeof CYCLE[number]);

    await expect(page.locator(THEME_BTN)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
