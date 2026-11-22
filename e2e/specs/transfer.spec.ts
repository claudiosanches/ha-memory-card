import { readFile } from 'node:fs/promises';
import { expect, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX, PROXY_ORIGIN } from '../constants.ts';

test('imports a CSV and exports it back', async ({ page }, testInfo) => {
  // Desktop and mobile share one server, so names carry the project.
  const p = testInfo.project.name;
  const csv = [
    'game_title,play_status,platform,region,format,device,store,condition,notes',
    `Ingress Quest ${p},backlog,Nintendo Switch,eu,physical,Console ${p},Shop ${p},cib,`,
    `Ingress Quest ${p},,Wii U,na,digital,Console ${p},,,"Bought on sale, 2015"`,
  ].join('\n');

  const escaped: string[] = [];
  page.on('request', (request) => {
    const url = new URL(request.url());
    if (url.origin === PROXY_ORIGIN && !url.pathname.startsWith(PREFIX)) escaped.push(url.pathname);
  });

  await page.goto(`${PREFIX}/#/import`);
  await page.locator('input[type="file"]').setInputFiles({
    name: 'games.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from(csv),
  });

  const preview = page.getByTestId('import-preview');
  await expect(preview.getByTestId('summary-new_games')).toHaveText('1');
  await expect(preview.getByTestId('summary-new_copies')).toHaveText('2');
  await expect(preview).toContainText(`Console ${p} (${en.deviceKind.console}): Nintendo Switch, Wii U`);
  await expect(preview).toContainText(`Shop ${p}`);

  await page.getByRole('button', { name: en.import.confirm }).click();
  await expect(page.getByTestId('import-done')).toContainText('Imported 1 new games and 2 new copies');

  const downloading = page.waitForEvent('download');
  await page.getByRole('link', { name: en.import.exportButton }).click();
  const download = await downloading;
  expect(download.suggestedFilename()).toMatch(/^memory-card-export-\d{8}\.csv$/);
  const exported = await readFile(await download.path(), 'utf8');
  expect(exported).toContain(`Ingress Quest ${p},backlog,Nintendo Switch,eu,,,physical,,Console ${p},Shop ${p},cib,,,,`);
  expect(exported).toContain(`Ingress Quest ${p},backlog,Wii U,na,,,digital,,Console ${p},,,,,,,,,"Bought on sale, 2015"`);

  await page.goto(`${PREFIX}/#/manage`);
  await expect(page.getByTestId(`device-Console ${p}`)).toContainText('Wii U');

  expect(escaped).toEqual([]);
});
