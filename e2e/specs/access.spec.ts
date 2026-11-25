import { expect, type Page, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX, PROXY_ORIGIN, VIEWER_ID, WRITE_HEADERS } from '../constants.ts';

function signInAs(page: Page, user: string) {
  return page.context().addCookies([{ name: 'e2e_user', value: user, url: PROXY_ORIGIN }]);
}

async function openNav(page: Page) {
  const burger = page.getByRole('button', { name: en.app.toggleNavigation });
  if (await burger.isVisible()) await burger.click();
}

test('a viewer browses the collection but cannot change it', async ({ page, request }, testInfo) => {
  // Desktop and mobile share one server, so the title carries the project.
  const title = `Viewer Quest ${testInfo.project.name}`;
  const created = await request.post(`${PROXY_ORIGIN}${PREFIX}/api/games`, { data: { title }, headers: WRITE_HEADERS });
  expect(created.ok()).toBe(true);
  const { id } = (await created.json()) as { id: number };

  await signInAs(page, VIEWER_ID);
  await page.goto(`${PREFIX}/#/games/${id}`);
  await expect(page.getByRole('heading', { name: title })).toBeVisible();
  await expect(page.getByRole('button', { name: en.common.edit })).toHaveCount(0);
  await expect(page.getByRole('button', { name: en.game.addCopy })).toHaveCount(0);

  await page.goto(`${PREFIX}/#/`);
  await expect(page.getByRole('heading', { name: en.nav.library })).toBeVisible();
  await expect(page.getByRole('button', { name: en.library.addGame })).toHaveCount(0);
  await openNav(page);
  await expect(page.getByRole('link', { name: en.nav.settings })).toBeVisible();
  await expect(page.getByRole('link', { name: en.nav.manage })).toHaveCount(0);

  const refused = await page.request.delete(`${PROXY_ORIGIN}${PREFIX}/api/games/${id}`, { headers: WRITE_HEADERS });
  expect(refused.status()).toBe(403);
  expect(await refused.json()).toEqual({ code: 'manager_required' });

  await page.goto(`${PREFIX}/#/settings?tab=status`);
  await expect(page.getByTestId('access-role')).toHaveText(en.settings.roles.viewer);
  await expect(page.getByTestId('user-id')).toHaveText(VIEWER_ID);
});

test('a request Home Assistant does not tie to a user is refused', async ({ page }) => {
  await signInAs(page, '');
  await page.goto(`${PREFIX}/`);
  await expect(page.getByRole('alert')).toHaveText(en.app.unidentified);

  const refused = await page.request.get(`${PROXY_ORIGIN}${PREFIX}/api/games`);
  expect(refused.status()).toBe(401);
});
