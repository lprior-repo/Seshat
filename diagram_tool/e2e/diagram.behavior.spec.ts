import { expect, test } from "@playwright/test";
import {
  canvas,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  runEffectsSequential,
  runEffect,
  trapPageErrors,
  waitForUiReady,
} from "./helpers";

test.describe("diagram editor hardening", () => {
  test.describe.configure({ mode: "parallel" });

  test("loads with core panels and controls", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

    await expect(page.getByRole("button", { name: "Auto-Arrange" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Validate" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Properties" })).toBeVisible();
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("survives rapid panel toggles", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

    const icons = page.getByRole("button", { name: "Icons", exact: true });
    const props = page.getByRole("button", { name: "Props", exact: true });
    const mini = page.getByRole("button", { name: "Mini", exact: true });
    const valid = page.getByRole("button", { name: "Valid", exact: true });

    for (let i = 0; i < 5; i += 1) {
      await runEffect(() => icons.click());
      await runEffect(() => props.click());
      await runEffect(() => mini.click());
      await runEffect(() => valid.click());
    }

    await expect(canvas(page)).toBeVisible();
    await expect(page.getByRole("button", { name: "Export JSON" })).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("survives validate storm while toggling panels", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

    const validate = page.getByRole("button", { name: "Validate", exact: true });
    const valid = page.getByRole("button", { name: "Valid", exact: true });
    const props = page.getByRole("button", { name: "Props", exact: true });

    for (let i = 0; i < 12; i += 1) {
      await runEffect(() => validate.click());
      await runEffect(() => valid.click());
      await runEffect(() => props.click());
      await runEffect(() => valid.click());
      await runEffect(() => props.click());
    }

    await expect(page.getByRole("button", { name: "Auto-Arrange" })).toBeVisible();
    await expectNodeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("handles aggressive zoom and theme flips", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

    const zoomIn = page.getByRole("button", { name: "+", exact: true }).first();
    const zoomOut = page.getByRole("button", { name: "-", exact: true });
    const theme = page.getByRole("combobox").first();

    for (let i = 0; i < 5; i += 1) {
      await runEffect(() => zoomIn.click());
      await runEffect(() => zoomOut.click());
    }

    await runEffect(() => theme.selectOption({ label: "Light theme" }));
    await runEffect(() => theme.selectOption({ label: "Dark theme" }));
    await runEffect(() => theme.selectOption({ label: "System theme" }));

    await expect(canvas(page)).toBeVisible();
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("survives keyboard shortcut fuzzing", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

    const keys = ["v", "h", "l", "r", "t", "Escape", "Delete", "Backspace", "0"];
    for (let i = 0; i < 6; i += 1) {
      for (const key of keys) {
        await runEffect(() => page.keyboard.press(key));
      }
      await runEffect(() => page.keyboard.press("+"));
      await runEffect(() => page.keyboard.press("-"));
    }

    await expect(page.getByRole("button", { name: "Select", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Pan", exact: true })).toBeVisible();
    await expectSelectedCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("survives wheel and space-pan stress", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

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
    await expect(page.getByRole("button", { name: "Validate" })).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });

  test("keeps pan controls responsive after stress", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await runEffectsSequential([() => page.goto("/"), () => waitForUiReady(page)]);

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

    await expect(page.getByRole("button", { name: "Pan", exact: true })).toBeVisible();
    await expect(canvas(page)).toBeVisible();
    expect(pageErrors).toHaveLength(0);
  });
});
