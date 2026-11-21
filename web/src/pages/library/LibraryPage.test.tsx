import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameDetail, GameSummary, Platform } from '../../api/types';
import en from '../../locales/en.json';
import { mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

const SWITCH: Platform = {
  id: 1,
  name: 'Nintendo Switch',
  slug: 'switch',
  abbreviation: 'Switch',
  family: null,
  generation: null,
  igdb_platform_id: 130,
  custom: false,
  active: true,
  sort_order: 10,
  game_count: 1,
  device_count: 0,
};

const GAMES: GameSummary[] = [
  {
    id: 4,
    title: 'Mario Kart 8 Deluxe',
    igdb_id: null,
    play_status: 'playing',
    last_played_at: null,
    owned_copies: 2,
    platforms: [{ platform_id: 1, copies: 2 }],
  },
];

function detail(id: number, title: string, igdbId: number | null = null): GameDetail {
  return {
    id,
    igdb_id: igdbId,
    title,
    summary: null,
    cover_path: null,
    cover_uploaded: false,
    first_release_date: null,
    developers: [],
    publishers: [],
    play_status: null,
    last_played_at: null,
    last_played_copy_id: null,
    local_max_players: null,
    online_max_players: null,
    local_coop: null,
    online_coop: null,
    game_mode_ids: [],
    genre_ids: [],
    releases: [],
  };
}

const IGDB_STATUS = { version: '0.1.0', igdb_configured: true, external_services: true, games: 1, owned_copies: 1 };
const HITS = [
  { igdb_id: 7, name: 'Metroid Dread', year: 2021, platforms: ['Switch'], game_id: null },
  { igdb_id: 8, name: 'Metroid Prime', year: 2002, platforms: ['GC'], game_id: 12 },
];

describe('Adding a game', () => {
  it('searches IGDB only when asked and links the chosen match', async () => {
    const calls = mockApi({
      'GET /api/games': GAMES,
      'GET /api/platforms': [SWITCH],
      'GET /api/status': IGDB_STATUS,
      'GET /api/igdb/search': HITS,
      'POST /api/games': detail(30, 'Metroid Dread'),
      'POST /api/games/30/igdb': detail(30, 'Metroid Dread', 7),
      'GET /api/games/30': detail(30, 'Metroid Dread', 7),
    });
    renderAt('/');

    fireEvent.click(await screen.findByRole('button', { name: en.library.addGame }));
    fireEvent.change(await screen.findByRole('textbox', { name: en.game.title }), { target: { value: 'metroid' } });
    expect(calls.some((c) => c.path === '/api/igdb/search')).toBe(false);

    fireEvent.click(screen.getByRole('button', { name: en.igdb.search }));
    fireEvent.click(await screen.findByTestId('igdb-hit-7'));
    expect(screen.getByRole('textbox', { name: en.game.title })).toHaveValue('Metroid Dread');
    expect(within(screen.getByTestId('igdb-hit-8')).getByText(en.igdb.inCollection)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/30/igdb')).toBe(true));
    expect(calls.filter((c) => c.path === '/api/igdb/search')).toHaveLength(1);
    expect(calls.find((c) => c.method === 'POST' && c.path === '/api/games')?.body).toEqual({
      title: 'Metroid Dread',
      play_status: null,
    });
    expect(calls.find((c) => c.path === '/api/games/30/igdb')?.body).toEqual({ igdb_id: 7, overwrite: false });
  });

  it('opens a match that is already in the collection instead of adding it again', async () => {
    const calls = mockApi({
      'GET /api/games': GAMES,
      'GET /api/platforms': [SWITCH],
      'GET /api/status': IGDB_STATUS,
      'GET /api/igdb/search': HITS,
      'GET /api/games/12': detail(12, 'Metroid Prime', 8),
    });
    renderAt('/');

    fireEvent.click(await screen.findByRole('button', { name: en.library.addGame }));
    fireEvent.change(await screen.findByRole('textbox', { name: en.game.title }), { target: { value: 'metroid' } });
    fireEvent.click(screen.getByRole('button', { name: en.igdb.search }));
    fireEvent.click(await screen.findByTestId('igdb-hit-8'));

    expect(await screen.findByRole('heading', { name: 'Metroid Prime' })).toBeInTheDocument();
    expect(calls.some((c) => c.method === 'POST')).toBe(false);
  });

  it('adds a game by hand without IGDB', async () => {
    mockApi({ 'GET /api/games': GAMES, 'GET /api/platforms': [SWITCH] });
    renderAt('/');

    fireEvent.click(await screen.findByRole('button', { name: en.library.addGame }));
    expect(await screen.findByRole('textbox', { name: en.game.title })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: en.igdb.search })).not.toBeInTheDocument();
  });
});

describe('Library', () => {
  it('shows games with status and platform badges', async () => {
    mockApi({ 'GET /api/games': GAMES, 'GET /api/platforms': [SWITCH] });
    renderAt('/');

    const card = await screen.findByTestId('game-Mario Kart 8 Deluxe');
    expect(within(card).getByText(en.playStatus.playing)).toBeInTheDocument();
    await waitFor(() => expect(within(card).getByText('Switch ×2')).toBeInTheDocument());
    expect(within(card).queryByText('2 copies')).not.toBeInTheDocument();
  });

  it('sends the filters from the URL to the API', async () => {
    const calls = mockApi({ 'GET /api/games': [], 'GET /api/platforms': [SWITCH] });
    renderAt('/?platform=%5B1%5D&play_status=%5B%22backlog%22%5D&ownership=any');

    expect(await screen.findByText(en.library.noMatch)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Filters (2)' })).toBeInTheDocument();
    const query = calls.find((c) => c.path === '/api/games')?.query;
    expect(Object.fromEntries(query ?? [])).toEqual({ platform: '1', play_status: 'backlog', ownership: 'any' });
  });

  it('explains an empty collection', async () => {
    mockApi({ 'GET /api/games': [], 'GET /api/platforms': [SWITCH] });
    renderAt('/');
    expect(await screen.findByText(en.library.empty)).toBeInTheDocument();
  });
});
