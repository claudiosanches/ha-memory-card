// Takes the docs screenshots into docs/public/images, which only the docs site
// serves; pages link them by their pages.dev URL. Runs its own server with the
// seed's test data, so it needs `make web`, `cargo build` and IGDB credentials in
// .data/options.json for the covers.
// Usage: node scripts/screenshots.mjs [name prefix...], or `make screenshots`.
// With prefixes, only matching images are written, such as `themes/`.
import { execFileSync, spawn } from 'node:child_process';
import { chmodSync, copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:net';
import { join } from 'node:path';
import { chromium } from '@playwright/test';

const root = join(import.meta.dirname, '..');
const out = join(root, 'docs/public/images');
const data = join(root, 'target/screenshots');
const user = { 'x-remote-user-id': 'morgan', 'x-remote-user-name': 'morgan', 'x-remote-user-display-name': 'Morgan' };
const hiddenThemes = new Set(['cursed-90s']);
const themes = Object.keys(JSON.parse(readFileSync(join(root, 'web/src/locales/en.json'), 'utf8')).settings.themes).filter(
  (theme) => !hiddenThemes.has(theme),
);
const only = process.argv.slice(2);

const options = JSON.parse(readFileSync(join(root, '.data/options.json'), 'utf8'));
if (!options.igdb_client_id || !options.igdb_client_secret) {
  console.error('Set igdb_client_id and igdb_client_secret in .data/options.json first, for the covers');
  process.exit(1);
}

const port = await new Promise((resolve) => {
  const probe = createServer().listen(0, '127.0.0.1', () => {
    const { port } = probe.address();
    probe.close(() => resolve(port));
  });
});
const base = `http://127.0.0.1:${port}`;

rmSync(data, { recursive: true, force: true });
mkdirSync(join(data, 'tmp'), { recursive: true });
mkdirSync(join(out, 'themes'), { recursive: true });
copyFileSync(join(root, '.data/options.json'), join(data, 'options.json'));
chmodSync(join(data, 'options.json'), 0o600);
const server = spawn(join(root, 'target/debug/memory-card'), ['serve'], {
  env: {
    ...process.env,
    MEMORY_CARD_DATA_DIR: data,
    MEMORY_CARD_TMP_DIR: join(data, 'tmp'),
    MEMORY_CARD_LISTEN: `127.0.0.1:${port}`,
    MEMORY_CARD_TRUSTED_PEER: '127.0.0.1',
    MEMORY_CARD_MANAGERS: 'morgan',
    MEMORY_CARD_VIEWERS_TRACK_PROGRESS: 'true',
    MEMORY_CARD_SENSORS: 'off',
    MEMORY_CARD_LOG_LEVEL: 'warning',
  },
  stdio: ['ignore', 'ignore', 'inherit'],
});

/** Calls the API as Morgan, the seed's manager. */
async function api(method, path, body) {
  const headers = { ...user, 'x-requested-with': 'memory-card' };
  if (body !== undefined) headers['content-type'] = 'application/json';
  const response = await fetch(`${base}${path}`, { method, headers, body: body && JSON.stringify(body) });
  if (!response.ok) throw new Error(`${method} ${path}: ${response.status}`);
  return response.json();
}

let browser;
try {
  for (let i = 0; ; i++) {
    try {
      if ((await fetch(`${base}/api/status`, { headers: user })).ok) break;
    } catch {}
    if (i > 100) throw new Error('The server did not start');
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  execFileSync('node', [join(root, 'scripts/seed.mjs'), base, 'morgan'], { stdio: 'inherit' });

  browser = await chromium.launch();
  const encoder = await browser.newPage();

  /** A PNG of `path` in the app, after `before` ran on the page. */
  async function capture(path, { width = 1440, height = 900, scheme = 'light', mobile = false, theme, view, before } = {}) {
    const context = await browser.newContext({
      viewport: { width, height },
      deviceScaleFactor: 2,
      isMobile: mobile,
      hasTouch: mobile,
      colorScheme: scheme,
      extraHTTPHeaders: user,
    });
    await context.addInitScript(
      ([theme, view]) => {
        localStorage.setItem('memory-card-language', 'en');
        if (theme) localStorage.setItem('memory-card-theme', JSON.stringify(theme));
        if (view) localStorage.setItem('memory-card-library-view', JSON.stringify(view));
      },
      [theme, view],
    );
    const page = await context.newPage();
    await page.goto(`${base}/#${path}`);
    await page.waitForLoadState('networkidle');
    if (before) await before(page);
    await page.waitForTimeout(1500);
    const png = await page.screenshot();
    await context.close();
    return png;
  }

  /** Takes the PNGs unless ONLY leaves `name` out, then saves them side by side as one WebP, scaled by `scale`. */
  async function save(name, shots, { scale = 1, gap = 0, pad = 0 } = {}) {
    if (only.length && !only.some((prefix) => name.startsWith(prefix))) return;
    const pngs = await shots();
    const url = await encoder.evaluate(
      async ({ images, scale, gap, pad }) => {
        const bitmaps = await Promise.all(
          images.map(async (b64) => createImageBitmap(await (await fetch(`data:image/png;base64,${b64}`)).blob())),
        );
        const width = bitmaps.reduce((sum, b) => sum + b.width, 0) + gap * (bitmaps.length - 1) + 2 * pad;
        const height = Math.max(...bitmaps.map((b) => b.height)) + 2 * pad;
        const canvas = document.createElement('canvas');
        canvas.width = Math.round(width * scale);
        canvas.height = Math.round(height * scale);
        const context = canvas.getContext('2d');
        context.imageSmoothingQuality = 'high';
        context.fillStyle = '#e9ecef';
        context.fillRect(0, 0, canvas.width, canvas.height);
        let x = pad;
        for (const b of bitmaps) {
          context.drawImage(b, x * scale, pad * scale, b.width * scale, b.height * scale);
          x += b.width + gap;
        }
        return canvas.toDataURL('image/webp', 0.82);
      },
      { images: pngs.map((png) => png.toString('base64')), scale, gap, pad },
    );
    if (!url.startsWith('data:image/webp')) throw new Error('This Chromium cannot write WebP');
    writeFileSync(join(out, `${name}.webp`), Buffer.from(url.split(',')[1], 'base64'));
    console.log(`docs/public/images/${name}.webp`);
  }

  // The seed leaves a few games for this page. Linking each to its closest
  // match afterwards, as Link there does, gives every other shot covers.
  await save('close-matches', async () => [await capture('/igdb-matches')]);
  for (const group of await api('GET', '/api/igdb/matches')) {
    await api('POST', `/api/games/${group.game_id}/igdb`, { igdb_id: group.candidates[0].igdb_id });
  }
  const games = await api('GET', '/api/games');
  const id = (title) => games.find((g) => g.title === title).id;

  await save('library', async () => [await capture('/')]);
  await save('library-dark', async () => [await capture('/', { scheme: 'dark' })]);
  await save('library-copies', async () => [await capture('/', { view: 'copies' })]);
  await save('add-game', async () => [
    await capture('/', {
      before: async (page) => {
        await page.getByRole('button', { name: 'Add game' }).click();
        const dialog = page.getByRole('dialog');
        await dialog.getByLabel('Title').fill('Tunic');
        await dialog.getByRole('button', { name: 'Search IGDB' }).click();
        await dialog.locator('[data-testid^="igdb-hit-"]').first().waitFor();
      },
    }),
  ]);
  await save('game', async () => [await capture(`/games/${id('Stardew Valley')}`, { height: 1380 })]);
  await save('backlog', async () => [
    await capture('/backlog', {
      before: async (page) => {
        await page.getByRole('button', { name: 'Pick for me' }).click();
        await page.waitForTimeout(2500);
      },
    }),
  ]);
  await save('wishlist', async () => [await capture('/wishlist')]);
  await save('duplicates', async () => [await capture('/maintenance?tab=duplicates')]);
  await save('stats', async () => [await capture('/stats', { scheme: 'dark' })]);
  await save('household', async () => [await capture('/household')]);
  await save('missing-details', async () => [await capture('/maintenance?tab=missing')]);
  await save('import', async () => [
    await capture('/import', {
      height: 1260,
      before: async (page) => {
        // Headers from another spreadsheet, to show them being recognized.
        const rows = [
          'Title,System,Region,Format,Price Paid,Currency,Comments',
          'Tunic,PS5,PAL,Physical,29.99,EUR,Birthday gift',
          'Hollow Knight: Silksong,Switch,NTSC-U,Digital,19.99,USD,',
          'Metroid Dread,Switch,NTSC-U,Physical,,,Second copy',
        ];
        await page.locator('input[type=file]').setInputFiles({
          name: 'my-games.csv',
          mimeType: 'text/csv',
          buffer: Buffer.from(rows.join('\n')),
        });
        await page.waitForLoadState('networkidle');
      },
    }),
  ]);
  await save('manage', async () => [await capture('/manage?tab=platforms')]);
  await save('hardware', async () => [await capture('/hardware')]);
  await save('settings', async () => [await capture('/settings')]);

  const phone = { width: 390, height: 844, mobile: true };
  await save(
    'phone',
    async () => [await capture('/', phone), await capture(`/games/${id('Mario Kart 8 Deluxe')}`, phone)],
    { scale: 0.6, gap: 48, pad: 48 },
  );

  for (const theme of themes) {
    for (const scheme of ['light', 'dark']) {
      // Themes hide their sidebar picture below 45em (720 pixels) tall.
      await save(`themes/${theme}-${scheme}`, async () => [
        await capture('/', { width: 1280, height: 900, theme, scheme }),
      ]);
    }
  }
} finally {
  await browser?.close();
  server.kill();
  rmSync(data, { recursive: true, force: true });
}
