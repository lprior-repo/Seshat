import { expect, test, type Page } from "@playwright/test";

import {
  canvas,
  ensureDeterministicUi,
  resetDocument,
  waitForCleanState,
  waitForE2eReady,
  waitForNoRebuildOverlay,
} from "./helpers";

type ViewportCase = {
  readonly name: string;
  readonly width: number;
  readonly height: number;
};

const VIEWPORTS: readonly ViewportCase[] = [
  { name: "desktop", width: 1280, height: 800 },
  { name: "mobile", width: 390, height: 844 },
  { name: "narrow", width: 320, height: 720 },
];

type LayoutAudit = {
  readonly bodyMargin: string;
  readonly iconItemCount: number;
  readonly iconCardWidths: readonly number[];
  readonly noRebuildText: boolean;
  readonly noXOverflow: boolean;
  readonly searchHasName: boolean;
  readonly themeInsideToolbar: boolean;
  readonly toolbarFits: boolean;
  readonly unnamedButtons: readonly string[];
};

async function auditLayout(page: Page): Promise<LayoutAudit> {
  return page.evaluate(() => {
    const isVisible = (element: Element): boolean =>
      element instanceof HTMLElement && element.offsetParent !== null;
    const toolbar = document.querySelector('[data-testid="toolbar-root"]');
    const search = document.querySelector('input[placeholder="Search icons..."]');
    const theme = document.querySelector('[data-testid="theme-toggle-btn"]');
    const toolbarChildren = toolbar
      ? Array.from(toolbar.querySelectorAll('button,[data-testid]')).filter(isVisible)
      : [];
    const toolbarBounds = toolbarChildren.map((element) => element.getBoundingClientRect());
    const maxToolbarRight = Math.max(0, ...toolbarBounds.map((bounds) => bounds.right));
    const minToolbarLeft = Math.min(0, ...toolbarBounds.map((bounds) => bounds.left));
    const themeRect = theme?.getBoundingClientRect();
    const toolbarRect = toolbar?.getBoundingClientRect();
    const buttons = Array.from(document.querySelectorAll("button")).filter(isVisible);
    const iconCardWidths = Array.from(document.querySelectorAll('[data-testid="icon-item"]'))
      .filter(isVisible)
      .slice(0, 12)
      .map((element) => Math.round(element.getBoundingClientRect().width));

    return {
      bodyMargin: getComputedStyle(document.body).margin,
      iconItemCount: Array.from(document.querySelectorAll('[data-testid="icon-item"]')).filter(isVisible).length,
      iconCardWidths,
      noRebuildText:
        !document.body.innerText.includes("Your app is being rebuilt") &&
        !document.body.innerText.includes("We're building your app now"),
      noXOverflow: document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1,
      searchHasName: Boolean(search?.getAttribute("aria-label") || search?.getAttribute("aria-labelledby")),
      themeInsideToolbar: Boolean(
        themeRect &&
          toolbarRect &&
          themeRect.top >= toolbarRect.top - 1 &&
          themeRect.bottom <= toolbarRect.bottom + 1 &&
          themeRect.right <= innerWidth + 1,
      ),
      toolbarFits: maxToolbarRight <= innerWidth + 1 && minToolbarLeft >= -1,
      unnamedButtons: buttons
        .map((button) =>
          button.getAttribute("data-testid") ??
          button.getAttribute("aria-label") ??
          button.getAttribute("title") ??
          button.innerText.trim(),
        )
        .filter((name, index) => {
          const button = buttons[index];
          return !(button?.getAttribute("aria-label") || button?.getAttribute("title") || button?.innerText.trim());
        }),
    };
  });
}

async function freshLayoutStart(page: Page): Promise<void> {
  await page.context().clearCookies();
  await page.addInitScript(() => {
    localStorage.setItem("diagram_tool.theme_mode", "dark");
  });
  await page.goto("/Seshat/", { waitUntil: "domcontentloaded" });
  await waitForNoRebuildOverlay(page);
  await ensureDeterministicUi(page);
  await expect(canvas(page)).toBeVisible({ timeout: 30_000 });
  await waitForE2eReady(page);
  await resetDocument(page);
  await waitForCleanState(page);
}

test.describe("UI polish regressions", () => {
  for (const viewport of VIEWPORTS) {
    test(`keeps the shell crisp at ${viewport.name} width @baseline`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await freshLayoutStart(page);

      const audit = await auditLayout(page);

      expect(audit.noRebuildText).toBe(true);
      expect(audit.bodyMargin).toBe("0px");
      expect(audit.noXOverflow).toBe(true);
      expect(audit.toolbarFits).toBe(true);
      expect(audit.searchHasName).toBe(true);
      expect(audit.unnamedButtons).toEqual([]);
      expect(audit.themeInsideToolbar).toBe(true);
      expect(audit.iconCardWidths.length).toBeGreaterThan(0);
      expect(audit.iconCardWidths.every((width) => width >= 70)).toBe(true);
      await expect(page).toHaveScreenshot(`seshat-shell-${viewport.name}.png`, {
        animations: "disabled",
        caret: "hide",
        maxDiffPixelRatio: 0.02,
      });
    });
  }

  test("exposes hidden actions in the mobile toolbar menu @baseline", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await freshLayoutStart(page);

    await page.getByTestId("toolbar-more").click();
    await expect(page.getByTestId("toolbar-more-menu")).toBeVisible();
    await expect(page.getByTestId("mobile-tool-subgraph")).toBeVisible();
    await expect(page.getByTestId("mobile-tool-grid")).toBeVisible();
    await expect(page.getByTestId("mobile-toolbar-undo")).toBeVisible();
    await expect(page.getByTestId("mobile-zoom-reset")).toBeVisible();
    await expect(page.getByTestId("mobile-style-arrow-type")).toBeVisible();
    await expect(page.getByTestId("mobile-toolbar-export")).toBeVisible();
    await expect(page.getByTestId("mobile-toolbar-import")).toBeVisible();

    const menuRight = await page.getByTestId("toolbar-more-menu").evaluate((element) => {
      const bounds = element.getBoundingClientRect();
      return bounds.right;
    });
    expect(menuRight).toBeLessThanOrEqual(391);
  });

  test("caps icon search DOM rendering for large result sets @baseline", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await freshLayoutStart(page);

    await page.getByRole("textbox", { name: "Search icons" }).fill("a");
    await expect.poll(async () => page.getByTestId("icon-item").count()).toBeGreaterThan(0);
    await expect.poll(async () => page.getByTestId("icon-item").count()).toBeLessThanOrEqual(96);
    await expect(page.getByText("Showing first 96 matches")).toBeVisible();
  });

  test("adds a consistent icon page on repeated load more clicks @baseline", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await freshLayoutStart(page);

    const loadMore = page.getByRole("button", { name: "Load more" });
    let visibleIcons = await page.getByTestId("icon-item").count();

    for (let clickIndex = 0; clickIndex < 4; clickIndex += 1) {
      await expect(loadMore).toBeVisible();
      await loadMore.scrollIntoViewIfNeeded();
      await loadMore.click();

      visibleIcons += 25;
      await expect.poll(async () => page.getByTestId("icon-item").count()).toBe(visibleIcons);
    }
  });
});
