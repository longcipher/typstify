
import { test, expect } from '@playwright/test';

test('verify search bar and theme toggle', async ({ page }) => {
  await page.goto('http://localhost:5173');

  // Check search bar visibility and width
  const searchInput = page.locator('#search-input');
  await expect(searchInput).toBeVisible();
  
  // Check if search input is within the viewport
  const box = await searchInput.boundingBox();
  const viewport = page.viewportSize();
  if (box && viewport) {
      expect(box.x + box.width).toBeLessThan(viewport.width);
  }

  // Check theme toggle existence
  const themeToggle = page.locator('#theme-toggle');
  await expect(themeToggle).toBeVisible();

  // Check initial theme (should be light or system default, let's assume light for now or check attribute)
  // We set data-theme attribute on html element
  const html = page.locator('html');
  
  // Click toggle
  await themeToggle.click();
  
  // Check if theme changed
  await expect(html).toHaveAttribute('data-theme', 'dark');
  
  // Check background color in dark mode
  const body = page.locator('body');
  await expect(body).toHaveCSS('background-color', 'rgb(25, 25, 25)'); // #191919

  // Click toggle again
  await themeToggle.click();
  await expect(html).toHaveAttribute('data-theme', 'light');
  await expect(body).toHaveCSS('background-color', 'rgb(255, 255, 255)'); // #ffffff
});
