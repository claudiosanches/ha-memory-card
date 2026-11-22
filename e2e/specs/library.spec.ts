import { expect, type Page, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX } from '../constants.ts';

async function pick(page: Page, scope: ReturnType<Page['getByRole']>, label: string, option: string) {
  await scope.getByRole('combobox', { name: label, exact: true }).click();
  await page.getByRole('option', { name: option, exact: true }).click();
}

test('add a game and a copy, then find and delete it', async ({ page }, testInfo) => {
  // Desktop and mobile share one server, so the title carries the project.
  const title = `Library Quest ${testInfo.project.name}`;

  await page.goto(`${PREFIX}/#/`);
  await page.getByRole('button', { name: en.library.addGame }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(en.game.title).fill(title);
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();

  await expect(page).toHaveURL(/#\/games\/\d+$/);
  await expect(page.getByRole('heading', { name: title })).toBeVisible();

  await page.getByRole('button', { name: en.game.addCopy }).click();
  await pick(page, dialog, en.game.platform, 'Nintendo Switch');
  await pick(page, dialog, en.game.region, en.region.eu);
  await pick(page, dialog, en.game.condition, en.condition.cib);
  await dialog.getByRole('button', { name: en.common.save, exact: true }).click();
  await expect(dialog).toBeHidden();
  await expect(page.getByText('Nintendo Switch · Europe (PAL)')).toBeVisible();

  // A 1x1 PNG, uploaded through the cover menu and served under the ingress path.
  const png = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==',
    'base64',
  );
  await page.getByRole('button', { name: en.cover.changeGame }).click();
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('menuitem', { name: en.cover.upload }).click();
  await (await chooser).setFiles({ name: 'cover.png', mimeType: 'image/png', buffer: png });
  // The release card shows the game cover too; the first is the game's own.
  const cover = page.getByRole('button', { name: en.cover.open.replace('{{title}}', title) }).first().getByRole('img');
  await expect(cover).toHaveAttribute('src', /^covers\/game-\d+\/upload-\d+\.png$/);
  await expect.poll(() => cover.evaluate((img: HTMLImageElement) => img.naturalWidth)).toBe(1);

  await page.getByRole('combobox', { name: en.game.playStatus, exact: true }).click();
  await page.getByRole('option', { name: en.playStatus.playing }).click();
  await page.getByRole('button', { name: en.game.playedToday }).click();
  await expect(page.getByTestId('last-played')).not.toContainText(en.game.never);

  // Back in the library, the filters in the URL find it.
  await page.goto(`${PREFIX}/#/?play_status=playing&platform=switch&q=${encodeURIComponent(title)}`);
  const card = page.getByTestId(`game-${title}`);
  await expect(card).toContainText(en.playStatus.playing);
  await expect(card).toContainText('Switch');

  await page.goto(`${PREFIX}/#/?play_status=backlog,paused&q=${encodeURIComponent(title)}`);
  await expect(page.getByText(en.library.noMatch)).toBeVisible();

  await page.goto(`${PREFIX}/#/?q=${encodeURIComponent(title)}`);
  await card.click();
  await page.getByRole('button', { name: en.game.delete }).click();
  await page.getByRole('dialog').getByRole('button', { name: en.common.delete }).click();
  await expect(page).toHaveURL(/#\/$/);
  await expect(page.getByTestId(`game-${title}`)).toBeHidden();
});
