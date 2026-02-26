import { expect, test } from "@playwright/test";
import {
  clearCanvasOverlays,
  createTextNode,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram panel persistence and resiliency", () => {
  test("panel toggles preserve canvas interactivity", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() =>
      expect(page.locator(".canvas-container")).toBeVisible({ timeout: 30_000 }),
    );
    await runEffect(() => waitForUiReady(page));

    const icons = page.getByRole("button", { name: "Icons", exact: true });
    const props = page.getByRole("button", { name: "Props", exact: true });
    const mini = page.getByRole("button", { name: "Mini", exact: true });
    const valid = page.getByRole("button", { name: "Valid", exact: true });

    for (let i = 0; i < 3; i += 1) {
      await runEffect(() => icons.click());
      await runEffect(() => props.click());
      await runEffect(() => mini.click());
      await runEffect(() => valid.click());
    }

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 780, 320));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    const textNode = canvas.getByText("Text", { exact: true }).first();
    const before = await runEffect(() => textNode.boundingBox());
    if (!before) {
      throw new Error("text node bounds missing before drag");
    }

    await runEffect(() => page.mouse.move(before.x + 8, before.y + 8));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(before.x + 52, before.y + 40));
    await runEffect(() => page.mouse.up());

    const after = await runEffect(() => textNode.boundingBox());
    if (!after) {
      throw new Error("text node bounds missing after drag");
    }

    expect(after.x).toBeGreaterThan(before.x + 14);
    expect(after.y).toBeGreaterThan(before.y + 14);
    expect(pageErrors).toHaveLength(0);
  });

  test("validation panel badge update path stays stable", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    await expect(page.getByText("Validation", { exact: true })).toBeVisible();

    const badge = page.locator(
      "xpath=//span[normalize-space()='Validation']/following-sibling::span[1]",
    );
    await expect(badge).toHaveText("Valid");

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 560, 220));
    await runEffect(() => createTextNode(page, canvas, 780, 320));
    await expect(page.getByText(/2 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Edge", exact: true }).click());
    const textNodes = canvas.getByText("Text", { exact: true });
    const first = await runEffect(() => textNodes.first().boundingBox());
    const second = await runEffect(() => textNodes.nth(1).boundingBox());
    if (!first || !second) {
      throw new Error("text node bounds missing for edge creation");
    }
    await runEffect(() => page.mouse.move(first.x + first.width / 2, first.y + first.height / 2));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await runEffect(() => page.mouse.move(second.x + second.width / 2, second.y + second.height / 2));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.up());
    await expect(page.getByText(/1 edges/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Validate", exact: true }).click());
    if (
      !(await runEffect(() =>
        page.getByText("Validation", { exact: true }).isVisible().catch(() => false),
      ))
    ) {
      await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    }
    await expect(page.getByText("Validation", { exact: true })).toBeVisible();
    await expect(badge).toHaveText("Valid");

    await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    await runEffect(() => page.getByRole("button", { name: "Valid", exact: true }).click());
    await expect(badge).toHaveText("Valid");
    expect(pageErrors).toHaveLength(0);
  });

  test("export buttons survive populated canvas without runtime errors", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));
    await runEffect(() => clearCanvasOverlays(page));

    const canvas = page.locator(".canvas-container");
    await runEffect(() => createTextNode(page, canvas, 520, 210));
    await runEffect(() => createTextNode(page, canvas, 740, 290));
    await runEffect(() => createTextNode(page, canvas, 900, 360));
    await expect(page.getByText(/3 nodes/)).toBeVisible();

    await runEffect(() => page.getByRole("button", { name: "Export JSON", exact: true }).click());
    await runEffect(() => page.getByRole("button", { name: "Export SVG", exact: true }).click());
    await runEffect(() => page.getByRole("button", { name: "Export PNG", exact: true }).click());

    await expect(page.locator(".canvas-container")).toBeVisible();
    await expect(page.getByText(/3 nodes/)).toBeVisible();
    await runEffect(() => page.waitForTimeout(200));
    expect(pageErrors).toHaveLength(0);
  });

  test("icon sidebar search and load-more remain sane", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffect(() => page.goto("/"));
    await runEffect(() => waitForUiReady(page));

    await expect(page.getByRole("heading", { name: "Diagram Icons" })).toBeVisible();
    const iconGridItems = page.locator(".icon-item");
    await expect(iconGridItems.first()).toBeVisible();

    const search = page.getByPlaceholder("Search icons...");
    await runEffect(() => search.fill("aws"));
    await expect(iconGridItems.first()).toBeVisible();

    await expect(
      page.getByRole("button", { name: "Load more", exact: true }),
    ).toHaveCount(0);

    await runEffect(() => search.fill(""));
    const initialVisibleCount = await runEffect(() => iconGridItems.count());

    const loadMore = page.getByRole("button", { name: "Load more", exact: true }).first();
    await expect(loadMore).toBeVisible();
    await runEffect(() => loadMore.click());
    await runEffect(() => page.waitForTimeout(250));

    await expect(iconGridItems.first()).toBeVisible();
    const afterLoadMoreCount = await runEffect(() => iconGridItems.count());
    expect(afterLoadMoreCount).toBeGreaterThanOrEqual(initialVisibleCount);

    await runEffect(() => search.fill("zzzzzz-no-match"));
    await expect(iconGridItems).toHaveCount(0);

    await runEffect(() => search.fill(""));
    await expect(iconGridItems.first()).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
