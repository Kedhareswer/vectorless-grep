import { expect, test } from "@playwright/test";

test("renders vectorless shell", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Vectorless" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Upload File" })).toBeVisible();
  await expect(
    page.getByPlaceholder("Ask a question about the document structure or content..."),
  ).toBeVisible();
});
