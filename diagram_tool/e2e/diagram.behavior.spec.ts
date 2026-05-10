import { expect, test } from "@playwright/test";
import {
  canvas,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  runEffect,
  runEffectsSequential,
  trapPageErrors,
} from "./helpers";

test.describe("diagram editor hardening", () => {
  test.describe.configure({ mode: "parallel" });

  test("loads with core panels and controls @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    await expect(page.getByTestId("toolbar-validate")).toHaveCount(1);
    await expect(page.getByTestId("panel-props-toggle")).toHaveCount(1);
    await expect(page.getByRole("heading", { name: "Properties" })).toBeVisible();
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("survives rapid panel toggles @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const icons = page.getByTestId("panel-icons-toggle");
    const props = page.getByTestId("panel-props-toggle");
    const mini = page.getByTestId("panel-mini-toggle");
    const valid = page.getByTestId("panel-valid-toggle");

    for (let i = 0; i < 5; i += 1) {
      await runEffect(() => icons.click());
      await runEffect(() => props.click());
      await runEffect(() => mini.click());
      await runEffect(() => valid.click());
    }

    await expect(page.getByText("Components")).toHaveCount(0);
    await expect(page.getByRole("heading", { name: "Properties" })).toHaveCount(0);
    await expect(page.getByTestId("minimap-viewport")).toHaveAttribute("data-visible", "false");
    await expect(page.getByTestId("validation-status")).toHaveAttribute("data-visible", "true");
    await expect(canvas(page)).toBeVisible();
    await expect(page.getByTestId("toolbar-export")).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("survives validate storm while toggling panels @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const validate = page.getByTestId("toolbar-validate");
    const valid = page.getByTestId("panel-valid-toggle");
    const props = page.getByTestId("panel-props-toggle");

    for (let i = 0; i < 12; i += 1) {
      await runEffect(() => validate.click());
      await runEffect(() => valid.click());
      await runEffect(() => props.click());
      await runEffect(() => valid.click());
      await runEffect(() => props.click());
    }

    await expect(page.getByRole("heading", { name: "Properties" })).toBeVisible();
    await expect(page.getByTestId("validation-status")).toHaveAttribute("data-visible", "false");
    await expect(page.getByTestId("toolbar-validate")).toHaveCount(1);
    await expectNodeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("handles aggressive zoom and theme flips @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const zoomIn = page.getByTestId("zoom-in");
    const zoomOut = page.getByTestId("zoom-out");
    const theme = page.getByTestId("theme-toggle-btn");

    for (let i = 0; i < 5; i += 1) {
      await runEffect(() => zoomIn.click());
      await runEffect(() => zoomOut.click());
    }

    await runEffect(() => theme.click());
    await expect(theme).toContainText("Light");
    await runEffect(() => theme.click());
    await expect(theme).toContainText("Dark");
    await runEffect(() => theme.click());
    await expect(theme).toContainText("White");

    await expect(canvas(page)).toBeVisible();
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("survives keyboard shortcut fuzzing @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const keys = ["v", "h", "l", "r", "t", "Escape", "Delete", "Backspace", "0"];
    for (let i = 0; i < 6; i += 1) {
      for (const key of keys) {
        await runEffect(() => page.keyboard.press(key));
      }
      await runEffect(() => page.keyboard.press("+"));
      await runEffect(() => page.keyboard.press("-"));
    }

    await expect(page.getByTestId("tool-select")).toBeVisible();
    await expect(page.getByTestId("tool-pan")).toBeVisible();
    await expectSelectedCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("survives wheel and space-pan stress @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    const box = await runEffect(() => canvasArea.boundingBox());
    if (!box) {
      throw new Error("canvas bounding box not available");
    }

    await runEffect(() => page.mouse.move(box.x + 240, box.y + 180));
    for (let i = 0; i < 12; i += 1) {
      await runEffect(() => page.mouse.wheel(0, -180));
      await runEffect(() => page.mouse.wheel(0, 180));
    }

    await runEffect(() => page.keyboard.down("Shift"));
    for (let i = 0; i < 4; i += 1) {
      await runEffect(() => page.mouse.wheel(0, 120));
      await runEffect(() => page.mouse.wheel(0, -120));
    }
    await runEffect(() => page.keyboard.up("Shift"));

    await runEffectsSequential([
      () => page.keyboard.down(" "),
      () => page.mouse.move(box.x + 280, box.y + 220),
      () => page.mouse.down(),
      () => page.mouse.move(box.x + 220, box.y + 220),
      () => page.mouse.move(box.x + 330, box.y + 220),
      () => page.mouse.up(),
      () => page.keyboard.up(" "),
    ]);

    await expect(canvas(page)).toBeVisible();
    await expect(page.getByTestId("toolbar-validate")).toHaveCount(1);
    expect(pageErrors).toHaveLength(0);
  });

  test("keeps pan controls responsive after stress @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const canvasArea = canvas(page);
    const canvasBox = await runEffect(() => canvasArea.boundingBox());
    if (!canvasBox) {
      throw new Error("canvas bounding box missing");
    }

    await runEffect(() => page.keyboard.down(" "));
    await runEffect(() => page.mouse.move(canvasBox.x + 340, canvasBox.y + 240));
    await runEffect(() => page.mouse.down());
    await runEffect(() => page.mouse.move(canvasBox.x + 240, canvasBox.y + 240));
    await runEffect(() => page.mouse.move(canvasBox.x + 360, canvasBox.y + 240));
    await runEffect(() => page.mouse.up());
    await runEffect(() => page.keyboard.up(" "));

    await expect(page.getByTestId("tool-pan")).toBeVisible();
    await expect(canvas(page)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
