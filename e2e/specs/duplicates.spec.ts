import { expect, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX, PROXY_ORIGIN, WRITE_HEADERS } from '../constants.ts';

test('lists a game owned twice and filters by label', async ({ page, request }, testInfo) => {
  // Desktop and mobile share one server, so titles carry the project.
  const title = `Twice Owned ${testInfo.project.name}`;
  const csv = ['game_title,platform,region,format', `${title},Switch,eu,physical`, `${title},Switch,eu,physical`].join('\n');
  const imported = await request.post(`${PROXY_ORIGIN}${PREFIX}/api/import`, {
    data: csv,
    headers: { ...WRITE_HEADERS, 'Content-Type': 'text/csv' },
  });
  expect(imported.ok()).toBe(true);

  await page.goto(`${PREFIX}/#/duplicates`);
  const group = page.getByTestId(`duplicate-${title}`);
  await expect(group).toContainText(en.duplicates.label.same_release);
  await expect(group.getByRole('listitem')).toHaveCount(2);

  await page.getByRole('button', { name: new RegExp(`^${en.duplicates.label.same_release}`) }).click();
  await expect(group).toBeVisible();

  await group.getByRole('link', { name: title }).click();
  await expect(page).toHaveURL(/#\/games\/\d+$/);
});
