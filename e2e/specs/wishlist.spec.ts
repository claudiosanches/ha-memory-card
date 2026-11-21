import { expect, type Page, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX } from '../constants.ts';

async function pick(page: Page, scope: ReturnType<Page['getByRole']>, label: string, option: string) {
  await scope.getByRole('combobox', { name: label, exact: true }).click();
  await page.getByRole('option', { name: option, exact: true }).click();
}

test('add a game to the wishlist, then move it into the collection', async ({ page }, testInfo) => {
  // Desktop and mobile share one server, so the title carries the project.
  const title = `Wish Quest ${testInfo.project.name}`;

  await page.goto(`${PREFIX}/#/wishlist`);
  await page.getByRole('button', { name: en.wishlist.add }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.game.title).fill(title);
  await dialog.getByRole('button', { name: en.wishlist.next }).click();

  await expect(dialog.getByRole('heading', { name: `Add ${title} to your wishlist` })).toBeVisible();
  await pick(page, dialog, en.game.platform, 'Nintendo Switch');
  await pick(page, dialog, en.game.region, en.region.ww);
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog).toBeHidden();

  const item = page.getByRole('listitem').filter({ hasText: title });
  await expect(item).toBeVisible();
  await item.getByRole('button', { name: en.game.gotIt }).click();
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(item).toBeHidden();

  await page.goto(`${PREFIX}/#/?q=${encodeURIComponent(title)}`);
  await expect(page.getByTestId(`game-${title}`)).toBeVisible();
});
