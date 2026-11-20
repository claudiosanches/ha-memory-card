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

  it('says when the game does not exist', async () => {
    mockApi({ ...SETUP, 'GET /api/games/77': json({ code: 'not_found' }, 404) });
    renderAt('/games/77');
    expect(await screen.findByText(en.game.notFound)).toBeInTheDocument();
  });
});
