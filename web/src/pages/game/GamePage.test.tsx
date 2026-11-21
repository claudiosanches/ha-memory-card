import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { GameDetail } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi, VIEWER } from '../../test/api';
import { renderAt } from '../../test/render';

/** Matches the element whose whole text is `text`, across a bold label. */
const wholeText = (text: string) => (_: string, element: Element | null) =>
  element?.textContent === text && [...element.children].every((child) => child.textContent !== text);

const GAME: GameDetail = {
  id: 4,
  igdb_id: null,
  title: 'Metroid Dread',
  summary: null,
  cover_path: null,
  cover_uploaded: false,
  first_release_date: null,
  developers: [],
  publishers: [],
  play_status: 'backlog',
  last_played_at: null,
  last_played_copy_id: null,
  local_max_players: null,
  online_max_players: null,
  local_coop: null,
  online_coop: null,
  game_mode_ids: [],
  genre_ids: [],
  releases: [
    {
      id: 9,
      platform_id: 1,
      region: 'eu',
      title: null,
      product_code: 'HAC-AYL3A',
      release_date: null,
      cover_path: null,
      cover_uploaded: false,
      local_max_players: null,
      online_max_players: null,
      local_coop: null,
      online_coop: null,
      copies: [
        { id: 21, status: 'owned', format: 'physical', device_id: null, store_id: null, condition: 'cib', notes: null },
      ],
    },
  ],
};

const SETUP = {
  'GET /api/platforms': [
    {
      id: 1,
      name: 'Nintendo Switch',
      slug: 'switch',
      abbreviation: 'Switch',
      family: null,
      generation: null,
      igdb_platform_id: 130,
      active: true,
      sort_order: 10,
      game_count: 1,
      device_count: 0,
    },
  ],
  'GET /api/devices': [],
  'GET /api/stores': [],
  'GET /api/game-modes': [],
};

describe('Game page', () => {
  it('shows releases and copies', async () => {
    mockApi({ ...SETUP, 'GET /api/games/4': GAME });
    renderAt('/games/4');

    const release = await screen.findByTestId('release-9');
    await waitFor(() => expect(within(release).getByText('Nintendo Switch · Europe (PAL)')).toBeInTheDocument());
    expect(within(release).getByText('HAC-AYL3A')).toBeInTheDocument();
    expect(within(screen.getByTestId('copy-21')).getByText(en.condition.cib)).toBeInTheDocument();
  });

  it('shows a viewer the game without edit controls', async () => {
    mockApi({ ...SETUP, 'GET /api/me': VIEWER, 'GET /api/games/4': GAME });
    renderAt('/games/4');

    const copy = await screen.findByTestId('copy-21');
    expect(within(copy).getByText(en.condition.cib)).toBeInTheDocument();
    expect(within(copy).queryByRole('button')).not.toBeInTheDocument();
    expect(within(screen.getByTestId('release-9')).queryByRole('button')).not.toBeInTheDocument();
    for (const name of [en.common.edit, en.game.delete, en.game.addCopy, en.game.playedToday, en.cover.changeGame]) {
      expect(screen.queryByRole('button', { name })).not.toBeInTheDocument();
    }
  });

  it('opens the library filtered by a game mode or genre', async () => {
    mockApi({
      ...SETUP,
      'GET /api/game-modes': [{ id: 3, name: 'Co-operative', slug: 'co-operative' }],
      'GET /api/genres': [{ id: 8, name: 'Platform' }],
      'GET /api/games/4': { ...GAME, game_mode_ids: [3], genre_ids: [8] },
      'GET /api/games': [],
    });
    const { router } = renderAt('/games/4');

    fireEvent.click(await screen.findByRole('link', { name: 'Co-operative' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/'));
    expect(router.state.location.search).toEqual({ game_mode: [3] });

    await router.navigate({ to: '/games/$gameId', params: { gameId: '4' } });
    fireEvent.click(await screen.findByRole('link', { name: 'Platform' }));
    await waitFor(() => expect(router.state.location.search).toEqual({ genre: [8] }));
  });

  it('lists copies without a table on a phone', async () => {
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: query.includes('max-width'),
      media: query,
      addEventListener: () => {},
      removeEventListener: () => {},
    }));
    const copy = { id: 21, status: 'owned', format: 'physical', device_id: null, store_id: 3, condition: 'cib', notes: 'Sealed spare' };
    mockApi({
      ...SETUP,
      'GET /api/stores': [{ id: 3, name: 'Amazon', notes: null, copy_count: 1 }],
      'GET /api/games/4': { ...GAME, releases: [{ ...GAME.releases[0], copies: [copy] }] },
    });
    renderAt('/games/4');

    const item = await screen.findByTestId('copy-21');
    expect(item.tagName).toBe('LI');
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
    expect(within(item).getByText(`${en.format.physical} · ${en.condition.cib}`)).toBeInTheDocument();
    expect(await within(item).findByText('Amazon')).toBeInTheDocument();
    expect(within(item).getByText('Sealed spare')).toBeInTheDocument();
    expect(within(item).getByRole('button', { name: en.game.editCopy })).toBeInTheDocument();
  });

  it('shows co-op next to player counts', async () => {
    const release = { ...GAME.releases[0], online_coop: true };
    mockApi({
      ...SETUP,
      'GET /api/games/4': { ...GAME, local_max_players: 2, local_coop: true, releases: [release] },
    });
    renderAt('/games/4');

    expect(
      await screen.findByText(wholeText(`${en.game.players}: 2 on the same screen (co-op), no online play`)),
    ).toBeInTheDocument();
    // The release adds online co-op to the game's values.
    const card = screen.getByTestId('release-9');
    expect(within(card).getByText(`${en.game.players}: 2 on the same screen (co-op), Unknown online (co-op)`)).toBeInTheDocument();
  });

  it('shows one player when single player is the only mode and counts are unknown', async () => {
    const modes = [
      { id: 1, name: 'Single player', slug: 'single-player' },
      { id: 2, name: 'Multiplayer', slug: 'multiplayer' },
    ];
    mockApi({ ...SETUP, 'GET /api/game-modes': modes, 'GET /api/games/4': { ...GAME, game_mode_ids: [1] } });
    renderAt('/games/4');
    expect(await screen.findByText(wholeText(`${en.game.players}: ${en.game.singlePlayer}`))).toBeInTheDocument();
  });

  it('keeps unknown counts for games with a multiplayer mode', async () => {
    const modes = [
      { id: 1, name: 'Single player', slug: 'single-player' },
      { id: 2, name: 'Multiplayer', slug: 'multiplayer' },
    ];
    mockApi({ ...SETUP, 'GET /api/game-modes': modes, 'GET /api/games/4': { ...GAME, game_mode_ids: [1, 2] } });
    renderAt('/games/4');
    expect(
      await screen.findByText(wholeText(`${en.game.players}: Unknown locally, Unknown online`)),
    ).toBeInTheDocument();
  });

  it('hides a release title equal to the game title', async () => {
    const release = { ...GAME.releases[0], title: 'Metroid Dread' };
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, releases: [release] } });
    renderAt('/games/4');

    const card = await screen.findByTestId('release-9');
    expect(within(card).getByText('HAC-AYL3A')).toBeInTheDocument();
    expect(within(card).queryByText(/Metroid Dread/)).toBeNull();
  });

  it('shows a release cover when the release has its own', async () => {
    const release = { ...GAME.releases[0], title: 'メトロイド ドレッド', cover_path: 'release-9/co5wrq.jpg' };
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, releases: [release] } });
    renderAt('/games/4');

    const card = await screen.findByTestId('release-9');
    expect(within(card).getByRole('img')).toHaveAttribute('src', 'covers/release-9/co5wrq.jpg');
  });

  it('uploads a release cover from the cover menu', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'PUT /api/releases/9/cover': {
        ...GAME,
        releases: [{ ...GAME.releases[0], cover_path: 'release-9/upload-1.png', cover_uploaded: true }],
      },
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.cover.changeRelease }));
    expect(await screen.findByRole('menuitem', { name: en.cover.upload })).toBeInTheDocument();
    expect(screen.queryByRole('menuitem', { name: en.cover.remove })).toBeNull();
    const file = new File(['png bytes'], 'box.png', { type: 'image/png' });
    fireEvent.change(screen.getByTestId('cover-file-release-9'), { target: { files: [file] } });

    const card = screen.getByTestId('release-9');
    await waitFor(() =>
      expect(within(card).getByRole('img')).toHaveAttribute('src', 'covers/release-9/upload-1.png'),
    );
    expect(calls.find((c) => c.method === 'PUT')?.body).toBe('png bytes');
  });

  it('marks the only copy as played today', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'PUT /api/games/4/last-played': (body: unknown) => ({ ...GAME, ...(body as object) }),
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.playedToday }));

    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    const body = calls.find((c) => c.method === 'PUT')?.body as Record<string, unknown>;
    expect(body.last_played_at).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(body.last_played_copy_id).toBe(21);
  });

  it('lets a viewer track progress when the option allows it', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/me': { ...VIEWER, can_track_progress: true },
      'GET /api/games/4': GAME,
      'PUT /api/games/4/last-played': (body: unknown) => ({ ...GAME, ...(body as object) }),
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.playedToday }));
    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/4/last-played')).toBe(true));
    expect(screen.queryByRole('button', { name: en.common.edit })).not.toBeInTheDocument();
  });

  it('links the game to an IGDB search result', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true, games: 1, owned_copies: 1 },
      'GET /api/games/4': GAME,
      'GET /api/igdb/search': [{ igdb_id: 7, name: 'Metroid Dread', year: 2021, platforms: ['Switch'], game_id: null }],
      'POST /api/games/4/igdb': { ...GAME, igdb_id: 7, developers: ['MercurySteam'] },
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.igdb.link }));
    await screen.findByTestId('igdb-hit-7');
    const search = screen.getByRole('textbox', { name: en.igdb.searchLabel });
    fireEvent.change(search, { target: { value: 'Metroid Dread (2021)' } });
    expect(calls.filter((c) => c.path === '/api/igdb/search')).toHaveLength(1);
    fireEvent.submit(search);
    await waitFor(() => expect(calls.filter((c) => c.path === '/api/igdb/search')).toHaveLength(2));
    fireEvent.click(await screen.findByTestId('igdb-hit-7'));

    expect(await screen.findByText(`${en.game.developers}: MercurySteam`)).toBeInTheDocument();
    expect(calls.find((c) => c.path === '/api/igdb/search')?.query.get('q')).toBe('Metroid Dread');
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ igdb_id: 7, overwrite: false });
  });

  it('says when the game does not exist', async () => {
    mockApi({ ...SETUP, 'GET /api/games/77': json({ code: 'not_found' }, 404) });
    renderAt('/games/77');
    expect(await screen.findByText(en.game.notFound)).toBeInTheDocument();
  });
});
