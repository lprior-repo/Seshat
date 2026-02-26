import { expect, type Locator, type Page } from "@playwright/test";
import { Effect } from "effect";

export async function runEffect<A>(thunk: () => Promise<A>): Promise<A> {
  return Effect.runPromise(
    Effect.tryPromise({
      try: thunk,
      catch: (error) =>
        new Error(error instanceof Error ? error.message : String(error)),
    }),
  );
}

export async function waitForUiReady(page: Page) {
  await expect(page.locator(".canvas-container")).toBeVisible();
  await expect(page.getByText(/0 nodes/)).toBeVisible({ timeout: 30_000 });
  await waitForNoRebuildOverlay(page);
}

export async function waitForNoRebuildOverlay(page: Page) {
  const rebuilding = page.getByRole("heading", {
    name: "Your app is being rebuilt.",
  });
  await expect(
    rebuilding,
  ).toHaveCount(0, { timeout: 60_000 });

  for (let i = 0; i < 3; i += 1) {
    await page.waitForTimeout(300);
    await expect(rebuilding).toHaveCount(0, { timeout: 5_000 });
  }
}

export function trapPageErrors(page: Page) {
  const errors: string[] = [];
  page.on("pageerror", (err) => {
    errors.push(err.message);
  });
  return errors;
}

export async function createTextNode(
  page: Page,
  canvas: Locator,
  x: number,
  y: number,
) {
  await waitForNoRebuildOverlay(page);
  await page.getByRole("button", { name: "Text", exact: true }).click();
  const box = await canvas.boundingBox();
  if (!box) {
    throw new Error("canvas bounding box not available");
  }
  await page.mouse.click(box.x + x, box.y + y);
}

export async function clearCanvasOverlays(page: Page) {
  await waitForNoRebuildOverlay(page);
  const iconsPanel = page.getByRole("heading", { name: "Diagram Icons" });
  if (await iconsPanel.isVisible().catch(() => false)) {
    await page.getByRole("button", { name: "Icons", exact: true }).click();
  }

  const propertiesPanel = page.getByRole("heading", { name: "Properties" });
  if (await propertiesPanel.isVisible().catch(() => false)) {
    await page.getByRole("button", { name: "Props", exact: true }).click();
  }
}

export async function nodeCenters(
  canvas: Locator,
): Promise<Array<{ x: number; y: number }>> {
  const boxes = await canvas
    .locator('div[style*="position: absolute"][style*="border-radius: 10px"]')
    .evaluateAll((elements) =>
      elements
        .map((element) => {
          const rect = element.getBoundingClientRect();
          return {
            x: rect.x + rect.width / 2,
            y: rect.y + rect.height / 2,
          };
        })
        .sort((a, b) => a.x - b.x),
    );
  return boxes;
}

export async function nodeFrameByLabel(
  page: Page,
  label: string,
  index = 0,
): Promise<{ x: number; y: number; width: number; height: number }> {
  const frame = await runEffect(() =>
    page.evaluate(
      ({ targetLabel, targetIndex }) => {
        const labels = Array.from(document.querySelectorAll("span"))
          .filter((el) => (el.textContent ?? "").trim() === targetLabel)
          .sort((a, b) => a.getBoundingClientRect().x - b.getBoundingClientRect().x);
        const labelEl = labels[targetIndex];
        if (!labelEl) {
          return null;
        }

        let current: HTMLElement | null = labelEl as HTMLElement;
        while (current) {
          const style = current.getAttribute("style") ?? "";
          if (style.includes("position: absolute") && style.includes("border-radius: 10px")) {
            const rect = current.getBoundingClientRect();
            return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
          }
          current = current.parentElement;
        }

        return null;
      },
      { targetLabel: label, targetIndex: index },
    ),
  );

  if (!frame) {
    throw new Error(`missing frame for label: ${label}`);
  }
  return frame;
}

export async function selectedCount(page: Page): Promise<number> {
  const text = await runEffect(() =>
    page.evaluate(() => {
      const match = document.body.innerText.match(/(\d+) selected/);
      return match ? match[1] : "0";
    }),
  );
  const parsed = Number.parseInt(text, 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}
