import { expect, test } from "@playwright/test";
import {
  canvas,
  expectEdgeCount,
  expectNodeCount,
  expectSelectedCount,
  freshStart,
  loadDocument,
  runEffect,
  runEffectsSequential,
  trapPageErrors,
  waitForNoRebuildOverlay,
} from "./helpers";

function threeNodeDocument(): Record<string, unknown> {
  return {
    version: 2,
    revision: 1,
    document: {
      nodes: {
        alpha: {
          kind: "text",
          icon: "",
          label: "Alpha",
          x: 180,
          y: 180,
          width: 120,
          height: 48,
          fontSize: null,
          font_weight: null,
          locked: false,
          parent: null,
          dag_rank: null,
          tags: [],
          metadata: {},
          z_index: 0,
          style: "box",
          collapsed: null,
        },
        beta: {
          kind: "text",
          icon: "",
          label: "Beta",
          x: 360,
          y: 180,
          width: 120,
          height: 48,
          fontSize: null,
          font_weight: null,
          locked: false,
          parent: null,
          dag_rank: null,
          tags: [],
          metadata: {},
          z_index: 0,
          style: "box",
          collapsed: null,
        },
        gamma: {
          kind: "text",
          icon: "",
          label: "Gamma",
          x: 540,
          y: 180,
          width: 120,
          height: 48,
          fontSize: null,
          font_weight: null,
          locked: false,
          parent: null,
          dag_rank: null,
          tags: [],
          metadata: {},
          z_index: 0,
          style: "box",
          collapsed: null,
        },
      },
      edges: {},
    },
    editor_state: {
      camera_x: 0,
      camera_y: 0,
      zoom: 1,
      snap_to_grid: true,
      grid_size: 20,
      selected_items: [],
      show_grid: true,
      minimap_visible: false,
    },
  };
}

test.describe("diagram editor hardening", () => {
  test.describe.configure({ mode: "parallel" });

  test("loads with core panels and controls @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    await expect(page.getByTestId("toolbar-validate")).toHaveCount(1);
    await expect(page.getByTestId("panel-props-toggle")).toHaveCount(1);
    await expect(page.getByTestId("panel-icons-toggle")).toContainText("Icons");
    await expect(page.getByTestId("panel-mini-toggle")).toContainText("Mini");
    await expect(page.getByTestId("panel-valid-toggle")).toContainText("Valid");
    await expect(page.getByTestId("panel-props-toggle")).toContainText("Props");
    await expect(page.getByTestId("toolbar-validate")).toContainText("Check");
    await expect(page.getByRole("heading", { name: "Properties" })).toHaveCount(0);
    await expectEdgeCount(page, 0);
    expect(pageErrors).toHaveLength(0);
  });

  test("selection font controls preserve multi-selection under repeated clicks @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);
    expect(await loadDocument(page, threeNodeDocument())).toBe(true);
    await waitForNoRebuildOverlay(page);
    await expectNodeCount(page, 3);

    await page.keyboard.press("ControlOrMeta+A");
    await expectSelectedCount(page, 3);
    const increase = page.getByTestId("selection-font-increase");
    await expect(increase).toHaveAttribute("aria-label", "Increase selected font size");
    await runEffect(() => increase.dblclick());
    await runEffect(() => increase.click());

    await expectNodeCount(page, 3);
    await expectSelectedCount(page, 3);
    expect(pageErrors).toHaveLength(0);
  });

  test("grid and minimap toggles update the rendered canvas affordances @baseline", async ({ page }) => {
    const pageErrors = trapPageErrors(page);
    await freshStart(page);

    const minimap = page.getByTestId("minimap-viewport");
    await expect(minimap).toHaveAttribute("data-visible", "true");
    await expect.poll(async () => (await minimap.boundingBox())?.width ?? 0).toBeGreaterThan(80);
    await expect.poll(async () => (await minimap.boundingBox())?.height ?? 0).toBeGreaterThan(50);

    await expect(page.locator("#canvas-grid-dot-pattern")).toHaveCount(1);
    await page.getByTestId("tool-grid").click();
    await expect(page.locator("#canvas-grid-dot-pattern")).toHaveCount(0);
    await page.getByTestId("tool-grid").click();
    await expect(page.locator("#canvas-grid-dot-pattern")).toHaveCount(1);
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
    await expect(page.getByRole("heading", { name: "Properties" })).toBeVisible();
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

    await expect(page.getByRole("heading", { name: "Properties" })).toHaveCount(0);
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
