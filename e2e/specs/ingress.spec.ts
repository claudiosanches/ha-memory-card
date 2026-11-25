import { expect, type Page, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX, PROXY_ORIGIN } from '../constants.ts';

/** Records every request that leaves the ingress prefix. */
function trackEscapes(page: Page): string[] {
  const escaped: string[] = [];
  page.on('request', (request) => {
    const url = new URL(request.url());
    if (url.origin === PROXY_ORIGIN && !url.pathname.startsWith(PREFIX)) {
      escaped.push(url.pathname);
    }
  });
  return escaped;
}

async function openNav(page: Page) {
  const burger = page.getByRole('button', { name: en.app.toggleNavigation });
  if (await burger.isVisible()) await burger.click();
}

/** Opens a settings section: tabs on wide screens, a select on phones. */
async function openSection(page: Page, name: string) {
  const tab = page.getByRole('tab', { name });
  const picker = page.getByRole('combobox', { name: en.nav.settings });
  await expect(tab.or(picker)).toBeVisible();
  if (await tab.isVisible()) return tab.click();
  await picker.click();
  await page.getByRole('option', { name }).click();
}

test('loads under the ingress prefix and navigates', async ({ page }) => {
  const escaped = trackEscapes(page);

  await page.goto(`${PREFIX}/`);
  await expect(page.getByRole('heading', { name: en.nav.library })).toBeVisible();

  await openNav(page);
  await page.getByRole('link', { name: en.nav.settings }).click();
  await expect(page).toHaveURL(new RegExp(`${PREFIX}/#/settings$`));
  await openSection(page, en.settings.status);
  await expect(page.getByTestId('status-version')).toHaveText(/^\d+\.\d+\.\d+/);
  await expect(page.getByTestId('igdb-missing')).toBeVisible();

  await page.reload();
  await expect(page.getByTestId('status-version')).toBeVisible();

  expect(escaped).toEqual([]);
});

test('works when the panel URL has no trailing slash', async ({ page }) => {
  const escaped = trackEscapes(page);

  await page.goto(PREFIX);
  await expect(page.getByRole('heading', { name: en.nav.library })).toBeVisible();

  expect(escaped).toEqual([]);
});
