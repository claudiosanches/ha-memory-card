import { expect, type Page, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX } from '../constants.ts';

// Desktop and mobile run in parallel against one server, so names carry the project.
async function openTab(page: Page, tab: 'devices' | 'stores' | 'platforms') {
  await page.goto(`${PREFIX}/#/manage?tab=${tab}`);
  await expect(page.getByRole('tab', { name: en.manage.tabs[tab], selected: true })).toBeVisible();
}

test('stores: add, rename, delete', async ({ page }, testInfo) => {
  const name = `Amazon ${testInfo.project.name}`;
  await openTab(page, 'stores');

  await page.getByRole('button', { name: en.stores.add }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.common.name).fill(name);
  await dialog.getByLabel(en.common.notes).fill('Prime');
  await dialog.getByRole('button', { name: en.common.save }).click();
  await expect(dialog).toBeHidden();

  const row = page.getByTestId(`store-${name}`);
  await expect(row).toContainText('Prime');

  await row.getByRole('button', { name: en.common.edit }).click();
  await dialog.getByLabel(en.common.name).fill(`${name} BR`);
  await dialog.getByRole('button', { name: en.common.save }).click();
  const renamed = page.getByTestId(`store-${name} BR`);
  await expect(renamed).toBeVisible();

  await page.reload();
  await renamed.getByRole('button', { name: en.common.delete }).click();
  await page.getByRole('dialog').getByRole('button', { name: en.common.delete }).click();
  await expect(renamed).toBeHidden();
});

test('devices: add with platforms, reject a taken name', async ({ page }, testInfo) => {
  const name = `Switch ${testInfo.project.name}`;
  await openTab(page, 'devices');

  await page.getByRole('button', { name: en.devices.add }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.common.name).fill(name);
  await dialog.getByLabel(en.devices.kind, { exact: true }).click();
  await page.getByRole('option', { name: en.deviceKind.handheld }).click();
  await dialog.getByLabel(en.devices.platforms, { exact: true }).fill('Switch');
  await page.getByRole('option', { name: 'Nintendo Switch', exact: true }).click();
  // Close the dropdown, which covers the Save button.
  await dialog.getByLabel(en.common.name).click();
  await dialog.getByRole('button', { name: en.common.save }).click();
  await expect(dialog).toBeHidden();

  const row = page.getByTestId(`device-${name}`);
  await expect(row).toContainText(en.deviceKind.handheld);
  await expect(row).toContainText('Switch');

  await page.getByRole('button', { name: en.devices.add }).click();
  await dialog.getByLabel(en.common.name).fill(name.toUpperCase());
  await dialog.getByRole('button', { name: en.common.save }).click();
  await expect(dialog.getByText(en.errors.name_taken)).toBeVisible();
});

test('platforms: add, deactivate, reorder, delete', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'platform order is shared state');
  await openTab(page, 'platforms');

  await page.getByRole('button', { name: en.platforms.add }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.common.name).fill('Sega Dreamcast');
  await dialog.getByLabel(en.platforms.abbreviation).fill('DC');
  await dialog.getByRole('button', { name: en.common.save }).click();
  await expect(dialog).toBeHidden();

  const rows = page.locator('[data-testid^="platform-"]');
  const row = page.getByTestId('platform-sega-dreamcast');
  await expect(rows.last()).toHaveAttribute('data-testid', 'platform-sega-dreamcast');

  await row.getByRole('switch', { name: 'Active: Sega Dreamcast' }).click({ force: true });
  await expect(row.getByRole('switch')).not.toBeChecked();

  await row.getByRole('button', { name: en.common.moveUp }).click();
  await expect(rows.nth(-2)).toHaveAttribute('data-testid', 'platform-sega-dreamcast');

  await page.reload();
  await expect(rows.nth(-2)).toHaveAttribute('data-testid', 'platform-sega-dreamcast');
  await expect(row.getByRole('switch')).not.toBeChecked();

  await row.getByRole('button', { name: en.common.delete }).click();
  await page.getByRole('dialog').getByRole('button', { name: en.common.delete }).click();
  await expect(row).toBeHidden();
});
