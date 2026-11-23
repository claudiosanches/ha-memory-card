import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { GameDetail, GameSummary, Platform } from '../../api/types';
import en from '../../locales/en.json';
import { mockApi, VIEWER } from '../../test/api';
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
  owned_game_count: 1,
  wanted_game_count: 0,
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
    wanted_copies: 0,
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
    local_max_players: null,
    online_max_players: null,
    local_coop: null,
    online_coop: null,
    game_mode_ids: [],
    genre_ids: [],
    labels: [],
    created_at: '2026-11-01T10:00:00.000Z',
    sessions: [],
    versions: [],
    names: [],
    releases: [],
  };
}

const IGDB_STATUS = { version: '0.1.0', igdb_configured: true, external_services: true };
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
  afterEach(() => localStorage.clear());

  it('lists copies in a table that follows the filters', async () => {
    const calls = mockApi({
      'GET /api/games': GAMES,
      'GET /api/platforms': [SWITCH],
      'GET /api/stores': [{ id: 3, name: 'Amazon', notes: null, copy_count: 1 }],
      'GET /api/copies': [
        {
          copy_id: 21,
          game_id: 4,
          title: 'Mario Kart 8 Deluxe',
          cover_path: null,
          play_status: 'playing',
          release_id: 9,
          platform_id: 1,
          region: 'eu',
          release_title: null,
          product_code: null,
          status: 'owned',
          format: 'physical',
          device_id: null,
          store_id: 3,
          condition: 'cib',
          notes: null,
        },
      ],
    });
    renderAt('/?platform=switch');

    fireEvent.click(await screen.findByRole('button', { name: en.library.copiesView }));
    const row = await screen.findByTestId('copy-row-21');
    expect(within(row).getByRole('link', { name: 'Mario Kart 8 Deluxe' })).toBeInTheDocument();
    expect(row).toHaveTextContent(en.region.eu);
    await waitFor(() => expect(row).toHaveTextContent('Amazon'));
    expect(screen.getByTestId('library-count')).toHaveTextContent('1 copy');
    expect(calls.find((c) => c.path === '/api/copies')?.query.get('platform')).toBe('switch');
    expect(localStorage.getItem('memory-card-library-view')).toBe('"copies"');
    expect(screen.getByRole('link', { name: en.library.export })).toHaveAttribute(
      'href',
      'api/export.csv?ownership=owned&platform=switch',
    );
  });

  it('leaves the export out for viewers', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/games': GAMES, 'GET /api/platforms': [SWITCH] });
    renderAt('/');
    expect(await screen.findByTestId('game-Mario Kart 8 Deluxe')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: en.library.export })).not.toBeInTheDocument();
  });

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
    renderAt('/?platform=switch,wii-u&play_status=backlog&ownership=any');

    expect(await screen.findByText(en.library.noMatch)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Filters (2)' })).toBeInTheDocument();
    const query = calls.find((c) => c.path === '/api/games')?.query;
    expect(Object.fromEntries(query ?? [])).toEqual({ platform: 'switch,wii-u', play_status: 'backlog', ownership: 'any' });
  });

  it('offers platform chips only for platforms with games under the Show filter', async () => {
    const SWITCH_2: Platform = { ...SWITCH, id: 2, name: 'Nintendo Switch 2', abbreviation: 'Switch 2', owned_game_count: 0, wanted_game_count: 1 };
    mockApi({ 'GET /api/games': GAMES, 'GET /api/platforms': [SWITCH, SWITCH_2] });
    renderAt('/');

    expect(await screen.findByRole('button', { name: 'Switch' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Switch 2' })).not.toBeInTheDocument();
  });

  it('offers the wishlist platforms when showing the wishlist', async () => {
    const SWITCH_2: Platform = { ...SWITCH, id: 2, name: 'Nintendo Switch 2', abbreviation: 'Switch 2', owned_game_count: 0, wanted_game_count: 1 };
    mockApi({ 'GET /api/games': [], 'GET /api/platforms': [SWITCH, SWITCH_2] });
    renderAt('/?ownership=wanted');

    expect(await screen.findByRole('button', { name: 'Switch 2' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Switch' })).not.toBeInTheDocument();
  });

  it('explains an empty collection', async () => {
    mockApi({ 'GET /api/games': [], 'GET /api/platforms': [SWITCH] });
    renderAt('/');
    expect(await screen.findByText(en.library.empty)).toBeInTheDocument();
  });

  it('does not ask viewers to add games', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/games': [], 'GET /api/platforms': [SWITCH] });
    renderAt('/');
    expect(await screen.findByText(en.library.emptyViewer)).toBeInTheDocument();
    expect(screen.queryByText(en.library.empty)).not.toBeInTheDocument();
  });
});
