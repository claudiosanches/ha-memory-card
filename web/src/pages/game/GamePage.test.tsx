import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameDetail } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

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
      'PUT /api/games/4': (body: unknown) => ({ ...GAME, ...(body as object) }),
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.playedToday }));

    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    const body = calls.find((c) => c.method === 'PUT')?.body as Record<string, unknown>;
    expect(body.last_played_at).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(body.last_played_copy_id).toBe(21);
    expect(body.play_status).toBe('backlog');
    expect(body.title).toBe('Metroid Dread');
  });

  it('links the game to an IGDB search result', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: true, games: 1, owned_copies: 1 },
      'GET /api/games/4': GAME,
      'GET /api/igdb/search': [{ igdb_id: 7, name: 'Metroid Dread', year: 2021, platforms: ['Switch'] }],
      'POST /api/games/4/igdb': { ...GAME, igdb_id: 7, developers: ['MercurySteam'] },
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.igdb.link }));
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
