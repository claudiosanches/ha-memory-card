import { expect, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX } from '../constants.ts';

// Desktop and mobile run in parallel against one server, so names carry the project.
test('hardware: add a device with platforms, reject a taken name', async ({ page }, testInfo) => {
  const name = `Switch ${testInfo.project.name}`;
  await page.goto(`${PREFIX}/#/hardware`);

  await page.getByRole('button', { name: en.hardware.add }).click();
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

  await page.getByRole('button', { name: en.hardware.add }).click();
  await dialog.getByLabel(en.common.name).fill(name.toUpperCase());
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog.getByText(en.errors.name_taken)).toBeVisible();
});
