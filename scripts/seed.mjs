// Fills an empty collection with test data: games on many platforms, owned
// and wanted copies, devices, stores, prices, play sessions, custom play
// statuses and a bundle, then links what it can on IGDB.
// Usage: node scripts/seed.mjs <url> [user id], for `make run` (user `dev`).
// scripts/ha-demo.mjs runs it through Home Assistant's ingress.

/** Calls the API as a manager. */
export async function seed(api) {
  if ((await api('GET', '/api/games')).length > 0) {
    console.log('The collection is not empty, so no test data was added');
    return false;
  }
  console.log('Adding test data');
  const platforms = new Map((await api('GET', '/api/platforms')).map((p) => [p.slug, p.id]));
  for (const [name, kind, slugs] of DEVICES) {
    await api('POST', '/api/devices', { name, kind, platform_ids: slugs.map((slug) => platforms.get(slug)) });
  }
  for (const [label, color, counts_as] of STATUSES) {
    await api('POST', '/api/play-statuses', { label, color, counts_as });
  }
  await api('POST', '/api/import', csv(), 'text/csv');

  const games = new Map((await api('GET', '/api/games')).map((g) => [g.title, g.id]));
  const bundle = (await api('GET', '/api/copies')).find((c) => c.title === BUNDLE.title);
  for (const title of BUNDLE.contents) {
    const id = games.get(title) ?? (await api('POST', '/api/games', { title })).id;
    await api('POST', `/api/copies/${bundle.copy_id}/contents`, { game_id: id });
  }

  const random = mulberry32(7);
  for (const [title, count] of SESSIONS) {
    const ago = new Set();
    while (ago.size < count) ago.add(Math.floor(random() * 120) + (title === LAST_PLAYED ? 0 : 1));
    for (const days of [...ago].sort((a, b) => b - a)) {
      const minutes = 30 + Math.floor(random() * 6) * 15;
      await api('POST', `/api/games/${games.get(title)}/sessions`, { played_on: daysAgo(title === LAST_PLAYED ? 0 : days), minutes });
    }
  }

  if ((await api('GET', '/api/status')).igdb_configured) {
    console.log('Linking games on IGDB');
    let sync = await api('POST', '/api/igdb/sync');
    while (sync.running) {
      await new Promise((resolve) => setTimeout(resolve, 2000));
      sync = await api('GET', '/api/igdb/sync');
    }
    console.log(`IGDB linked ${sync.linked}, left ${sync.unmatched} unmatched`);
  }
  await api('POST', '/api/suggestions', {});
  console.log(`Added ${(await api('GET', '/api/games')).length} games`);
  return true;
}

const DEVICES = [
  ['Switch OLED', 'console', ['switch']],
  ['Nintendo Account', 'account', ['switch']],
  ['PS5', 'console', ['ps5', 'ps4']],
  ['PlayStation Network', 'account', ['ps5', 'ps4']],
  ['PS2 Slim', 'console', ['ps2']],
  ['Gaming PC', 'pc', ['pc']],
  ['Steam account', 'account', ['pc']],
  ['GOG account', 'account', ['pc']],
  ['Xbox Series X', 'console', ['xbox-series']],
  ['Microsoft account', 'account', ['xbox-series', 'pc']],
  ['GameCube', 'console', ['gamecube']],
  ['Nintendo 64', 'console', ['n64']],
  ['Super Nintendo', 'console', ['snes']],
  ['Game Boy Advance SP', 'handheld', ['gba']],
  ['New Nintendo 3DS XL', 'handheld', ['3ds']],
];

const STATUSES = [
  ['Replaying', 'violet', 'playing'],
  ['Collection only', 'gray', 'not_counted'],
];

const BUNDLE = {
  title: 'Super Mario 3D All-Stars',
  contents: ['Super Mario 64', 'Super Mario Sunshine', 'Super Mario Galaxy'],
};

// [title, play status, platform, region, format, device, store, condition,
//  bought on, price, currency, estimated value, extra columns]
const ROWS = [
  ['The Legend of Zelda: Breath of the Wild', 'completed', 'switch', 'eu', 'physical', 'Switch OLED', 'Amazon', 'cib', '2023-02-10', '299.90', 'BRL', '180.00'],
  ['The Legend of Zelda: Tears of the Kingdom', 'playing', 'switch', 'eu', 'physical', 'Switch OLED', 'Local game shop', 'cib', '2023-05-12', '1299.90', 'BRL', '1500.00', { edition: "Collector's Edition" }],
  ['Super Mario Odyssey', 'completed', 'switch', 'na', 'physical', 'Switch OLED', 'eBay', 'cib', '2023-08-03', '39.99', 'USD'],
  ['Metroid Dread', 'mastered', 'switch', 'eu', 'physical', 'Switch OLED', 'Amazon', 'cib', '2023-01-20', '89.99', 'EUR', '120.00', { edition: 'Special Edition' }],
  ['Animal Crossing: New Horizons', 'paused', 'switch', 'br', 'digital', 'Nintendo Account', 'Nintendo eShop', '', '2024-03-15', '299.00', 'BRL'],
  ['Mario Kart 8 Deluxe', 'completed', 'switch', 'br', 'physical', 'Switch OLED', 'Mercado Livre', 'cib', '2023-12-25', '249.90', 'BRL'],
  ['Hades', 'completed', 'switch', 'ww', 'digital', 'Nintendo Account', 'Nintendo eShop', '', '2024-06-01', '87.49', 'BRL'],
  ['Hollow Knight', 'backlog', 'switch', 'ww', 'digital', 'Nintendo Account', 'Nintendo eShop', '', '2025-01-10', '27.99', 'BRL'],
  ['Xenoblade Chronicles 3', 'backlog', 'switch', 'eu', 'physical', 'Switch OLED', 'Amazon', 'sealed', '2025-07-29', '249.00', 'BRL'],
  ['Fire Emblem: Three Houses', 'dropped', 'switch', 'na', 'physical', 'Switch OLED', 'eBay', 'cib', '2024-02-02', '35.00', 'USD'],
  ['Super Smash Bros. Ultimate', 'completed', 'switch', 'br', 'physical', 'Switch OLED', 'Mercado Livre', 'cib', '2023-12-25', '279.90', 'BRL'],
  ['Metroid Prime Remastered', 'backlog', 'switch', 'jp', 'physical', 'Switch OLED', 'Local game shop', 'cib', '2025-03-02', '5980', 'JPY', '', { release_title: 'メトロイドプライム リマスタード' }],
  ['Okami HD', 'backlog', 'switch', 'na', 'digital', 'Nintendo Account', 'Nintendo eShop', '', '2024-11-29', '19.99', 'USD'],
  [BUNDLE.title, 'completed', 'switch', 'na', 'physical', 'Switch OLED', 'eBay', 'cib', '2023-09-18', '59.99', 'USD', '90.00'],
  ['Metroid Prime', 'completed', 'gamecube', 'na', 'physical', 'GameCube', 'eBay', 'no_manual', '2024-08-10', '25.00', 'USD', '35.00'],
  ['The Legend of Zelda: The Wind Waker', 'completed', 'gamecube', 'eu', 'physical', 'GameCube', 'eBay', 'cib', '2024-08-10', '45.00', 'EUR', '70.00'],
  ['Super Smash Bros. Melee', 'Collection only', 'gamecube', 'na', 'physical', 'GameCube', 'Local game shop', 'loose', '2024-09-01', '30.00', 'USD', '45.00'],
  ['Super Mario 64', 'completed', 'n64', 'na', 'physical', 'Nintendo 64', 'eBay', 'loose', '2023-04-04', '30.00', 'USD', '40.00'],
  ['The Legend of Zelda: Ocarina of Time', 'completed', 'n64', 'br', 'physical', 'Nintendo 64', 'Mercado Livre', 'cib', '2023-04-04', '350.00', 'BRL', '500.00'],
  ['Super Metroid', 'Replaying', 'snes', 'na', 'physical', 'Super Nintendo', 'eBay', 'loose', '2023-06-06', '60.00', 'USD', '75.00'],
  ['Chrono Trigger', 'backlog', 'snes', 'jp', 'physical', 'Super Nintendo', 'Local game shop', 'cib', '2023-06-06', '4500', 'JPY', '9000', { release_title: 'クロノ・トリガー' }],
  ['Super Mario World', 'Collection only', 'snes', 'br', 'physical', 'Super Nintendo', 'Mercado Livre', 'loose', '2022-12-24', '80.00', 'BRL'],
  ['Metroid Fusion', 'completed', 'gba', 'eu', 'physical', 'Game Boy Advance SP', 'eBay', 'loose', '2024-01-15', '25.00', 'EUR'],
  ['Pokémon Emerald Version', 'paused', 'gba', 'na', 'physical', 'Game Boy Advance SP', 'eBay', 'loose', '2024-01-15', '60.00', 'USD', '90.00'],
  ['The Legend of Zelda: Ocarina of Time 3D', 'backlog', '3ds', 'eu', 'physical', 'New Nintendo 3DS XL', 'Amazon', 'cib', '2024-05-05', '20.00', 'EUR'],
  ['Okami', 'completed', 'ps2', 'eu', 'physical', 'PS2 Slim', 'eBay', 'cib', '2023-10-10', '30.00', 'EUR', '45.00'],
  ['Final Fantasy X', 'completed', 'ps2', 'na', 'physical', 'PS2 Slim', 'eBay', 'cib', '2023-10-10', '15.00', 'USD'],
  ['Bloodborne', 'completed', 'ps4', 'eu', 'physical', 'PS5', 'Amazon', 'cib', '2023-03-03', '99.90', 'BRL'],
  ['Persona 5 Royal', 'dropped', 'ps4', 'na', 'physical', 'PS5', 'Amazon', 'cib', '2024-04-04', '29.99', 'USD'],
  ['Elden Ring', 'playing', 'ps5', 'br', 'physical', 'PS5', 'Amazon', 'cib', '2024-02-25', '249.90', 'BRL'],
  ['Final Fantasy VII Rebirth', 'backlog', 'ps5', 'br', 'physical', 'PS5', 'Amazon', 'sealed', '2024-02-29', '349.90', 'BRL', '', { edition: 'Deluxe Edition' }],
  ['Astro Bot', 'mastered', 'ps5', 'ww', 'digital', 'PlayStation Network', 'PlayStation Store', '', '2024-09-06', '299.90', 'BRL'],
  ["Marvel's Spider-Man 2", 'backlog', 'ps5', 'ww', 'digital', 'PlayStation Network', 'PlayStation Store', '', '2025-02-14', '149.95', 'BRL'],
  ['Hades', 'completed', 'pc', 'ww', 'digital', 'Steam account', 'Steam', '', '2023-07-07', '47.49', 'BRL'],
  ['Celeste', 'completed', 'pc', 'ww', 'digital', 'Steam account', 'Steam', '', '2023-07-07', '11.99', 'BRL'],
  ['Stardew Valley', 'playing', 'pc', 'ww', 'digital', 'Steam account', 'Steam', '', '2023-07-07', '24.99', 'BRL'],
  ['The Witcher 3: Wild Hunt', 'backlog', 'pc', 'ww', 'digital', 'GOG account', 'GOG', '', '2024-12-18', '39.99', 'BRL'],
  ["Baldur's Gate 3", 'paused', 'pc', 'ww', 'digital', 'Steam account', 'Steam', '', '2024-08-03', '199.99', 'BRL'],
  ['Disco Elysium', 'backlog', 'pc', 'ww', 'digital', 'GOG account', 'GOG', '', '2025-06-20', '18.49', 'BRL'],
  ['Forza Horizon 5', 'paused', 'xbox-series', 'ww', 'digital', 'Microsoft account', 'Microsoft Store', '', '2024-10-10', '249.00', 'BRL'],
  ['Halo Infinite', 'dropped', 'xbox-series', 'ww', 'digital', 'Microsoft account', 'Microsoft Store', '', '', '', ''],
  ['Pikmin 4', 'backlog', 'switch', 'eu', 'physical', '', '', '', '', '299.00', 'BRL', '', { status: 'wanted' }],
  ['Metroid Prime 4: Beyond', '', 'switch-2', 'br', 'physical', '', '', '', '', '449.90', 'BRL', '', { status: 'wanted' }],
  ['EarthBound', '', 'snes', 'na', 'physical', '', '', 'loose', '', '250.00', 'USD', '', { status: 'wanted' }],
  ['Hollow Knight: Silksong', '', 'switch', 'ww', 'digital', '', '', '', '', '59.99', 'BRL', '', { status: 'wanted' }],
];

/** Games with no copy yet. */
const NO_COPY = [['Outer Wilds', 'backlog']];

/** Play sessions per game; the last played game gets one today. */
const SESSIONS = [
  ['The Legend of Zelda: Tears of the Kingdom', 6],
  ['Elden Ring', 4],
  ['Stardew Valley', 5],
  ['Super Metroid', 2],
  ['Animal Crossing: New Horizons', 3],
  ["Baldur's Gate 3", 2],
  ['Metroid Dread', 3],
  ['Astro Bot', 2],
  ['Hades', 4],
  ['Celeste', 2],
];
const LAST_PLAYED = 'The Legend of Zelda: Tears of the Kingdom';

const COLUMNS = ['status', 'game_title', 'play_status', 'platform', 'region', 'release_title', 'format', 'edition', 'device', 'store', 'condition', 'purchased_on', 'price', 'currency', 'estimated_value', 'barcode'];

function csv() {
  const lines = [COLUMNS.join(',')];
  ROWS.forEach((row, n) => {
    const [title, play_status, platform, region, format, device, store, condition, purchased_on, price, currency, estimated, extra = {}] = row;
    const values = {
      status: 'owned', game_title: title, play_status, platform, region, format, device, store, condition, purchased_on, price, currency,
      estimated_value: estimated,
      barcode: format === 'physical' && n % 3 === 0 ? ean13(`789000${String(n).padStart(6, '0')}`) : '',
      ...extra,
    };
    lines.push(COLUMNS.map((column) => quote(values[column] ?? '')).join(','));
  });
  for (const [title, play_status] of NO_COPY) {
    lines.push(COLUMNS.map((column) => quote({ game_title: title, play_status }[column] ?? '')).join(','));
  }
  return `${lines.join('\n')}\n`;
}

function quote(value) {
  return /[",\n]/.test(value) ? `"${value.replaceAll('"', '""')}"` : value;
}

function ean13(twelve) {
  const sum = [...twelve].reduce((total, digit, i) => total + Number(digit) * (i % 2 ? 3 : 1), 0);
  return `${twelve}${(10 - (sum % 10)) % 10}`;
}

function daysAgo(days) {
  const date = new Date();
  date.setDate(date.getDate() - days);
  return date.toISOString().slice(0, 10);
}

function mulberry32(seed) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const [base, user = 'dev'] = process.argv.slice(2);
  if (!base) {
    console.error('usage: node scripts/seed.mjs <url> [user id]');
    process.exit(1);
  }
  const api = async (method, path, body, type = 'application/json') => {
    const headers = { 'x-remote-user-id': user, 'x-remote-user-name': user, 'x-remote-user-display-name': user };
    if (body !== undefined) headers['content-type'] = type;
    const response = await fetch(`${base}${path}`, {
      method,
      headers,
      body: body === undefined || typeof body === 'string' ? body : JSON.stringify(body),
    });
    const text = await response.text();
    if (!response.ok) throw new Error(`${method} ${path}: ${response.status} ${text}`);
    return text ? JSON.parse(text) : null;
  };
  seed(api).catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}
