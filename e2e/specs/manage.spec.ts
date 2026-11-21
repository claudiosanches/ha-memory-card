import { expect, type Page, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX } from '../constants.ts';

// Desktop and mobile run in parallel against one server, so names carry the project.
async function openTab(page: Page, tab: 'devices' | 'stores' | 'platforms') {
  await page.goto(`${PREFIX}/#/manage?tab=${tab}`);
  await expect(page.getByRole('tabpanel', { name: en.manage.tabs[tab] })).toBeVisible();
}

test('stores: add, rename, delete', async ({ page }, testInfo) => {
  const name = `Amazon ${testInfo.project.name}`;
  await openTab(page, 'stores');

  await page.getByRole('button', { name: en.stores.add }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.common.name).fill(name);
  await dialog.getByLabel(en.common.notes).fill('Prime');
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog).toBeHidden();

  const row = page.getByTestId(`store-${name}`);
  await expect(row).toContainText('Prime');

  await row.getByRole('button', { name: en.common.edit }).click();
  await dialog.getByLabel(en.common.name).fill(`${name} BR`);
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
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
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog).toBeHidden();

  const row = page.getByTestId(`device-${name}`);
  await expect(row).toContainText(en.deviceKind.handheld);
  await expect(row).toContainText('Switch');

  await page.getByRole('button', { name: en.devices.add }).click();
  await dialog.getByLabel(en.common.name).fill(name.toUpperCase());
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog.getByText(en.errors.name_taken)).toBeVisible();
});

test('platforms: add from the catalog, reorder, remove, delete a custom one', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'platform order is shared state');
  await openTab(page, 'platforms');
  const yours = page.locator('ol [data-testid^="platform-"]');

  await page.getByRole('textbox', { name: en.platforms.search }).fill('dreamcast');
  await page.getByRole('button', { name: 'Add Dreamcast' }).click();
  const dreamcast = yours.filter({ hasText: 'Dreamcast' });
  await expect(yours.last()).toContainText('Dreamcast');

  // Drag the grip onto the platform above it.
  const grip = await dreamcast.getByRole('button', { name: 'Move Dreamcast', exact: true }).boundingBox();
  const above = await yours.nth(-2).boundingBox();
  if (!grip || !above) throw new Error('rows are not visible');
  await page.mouse.move(grip.x + grip.width / 2, grip.y + grip.height / 2);
  await page.mouse.down();
  await page.mouse.move(grip.x + grip.width / 2, above.y + above.height / 2, { steps: 12 });
  await page.mouse.up();
  await expect(yours.nth(-2)).toContainText('Dreamcast');
  await page.reload();
  await expect(yours.nth(-2)).toContainText('Dreamcast');

  await dreamcast.getByRole('button', { name: 'Remove Dreamcast from your platforms' }).click();
  await expect(dreamcast).toHaveCount(0);

  await page.getByRole('button', { name: en.platforms.add }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.common.name).fill('Homebrew Console');
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog).toBeHidden();
  const custom = page.getByTestId('platform-homebrew-console');
  await custom.getByRole('button', { name: en.common.delete }).click();
  await page.getByRole('dialog').getByRole('button', { name: en.common.delete }).click();
  await expect(custom).toBeHidden();
});
