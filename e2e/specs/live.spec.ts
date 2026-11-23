import { expect, test } from '@playwright/test';
import en from '../../web/src/locales/en.json' with { type: 'json' };
import { PREFIX, WRITE_HEADERS } from '../constants.ts';

test('a game page shows a change made elsewhere without a reload', async ({ page, request }, testInfo) => {
  // Desktop and mobile share one server, so the title carries the project.
  const title = `Live Update ${testInfo.project.name}`;
  const created = await request.post(`${PREFIX}/api/games`, { data: { title }, headers: WRITE_HEADERS });
  expect(created.ok()).toBe(true);
  const { id } = (await created.json()) as { id: number };

  const listening = page.waitForResponse((response) => response.url().endsWith('/api/events'));
  await page.goto(`${PREFIX}/#/games/${id}`);
  await expect(page.getByRole('heading', { name: title })).toBeVisible();
  await listening;
  const status = page.getByRole('combobox', { name: en.game.playStatus, exact: true });
  await expect(status).not.toHaveValue(en.playStatus.playing);

  // Such as a tag scan, which changes the game on the server.
  const changed = await request.put(`${PREFIX}/api/games/${id}/play-status`, {
    data: { play_status: 'playing' },
    headers: WRITE_HEADERS,
  });
  expect(changed.ok()).toBe(true);
  await expect(status).toHaveValue(en.playStatus.playing);

  expect((await request.delete(`${PREFIX}/api/games/${id}`, { headers: WRITE_HEADERS })).ok()).toBe(true);
});
