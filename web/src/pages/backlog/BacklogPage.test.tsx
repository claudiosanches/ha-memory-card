import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameDetail, GameSummary, PlayStatus } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

function game(id: number, title: string, play_status: PlayStatus | null): GameSummary {
  return {
    id,
    title,
    igdb_id: null,
    cover_path: null,
    play_status,
    last_played_at: null,
    owned_copies: 1,
    platforms: [{ platform_id: 1, copies: 1 }],
  };
}

const GAMES = [
  game(1, 'Hades', 'playing'),
  game(2, 'Okami', 'paused'),
  game(3, 'Celeste', 'backlog'),
  game(4, 'Tunic', 'backlog'),
  game(5, 'Zelda', 'completed'),
  game(6, 'Metroid', null),
  game(7, 'Kirby', null),
];

function detail(summary: GameSummary, play_status: PlayStatus | null): GameDetail {
  return {
    id: summary.id,
    igdb_id: null,
    title: summary.title,
    summary: null,
    cover_path: null,
    cover_uploaded: false,
    first_release_date: null,
    developers: [],
    publishers: [],
    play_status,
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

const SETUP = {
  'GET /api/games': GAMES,
  'GET /api/platforms': [],
  'GET /api/devices': [],
};

const statusPut = (id: number) => (body: unknown) =>
  detail(GAMES.find((g) => g.id === id) as GameSummary, (body as { play_status: PlayStatus }).play_status);

describe('Backlog page', () => {
  it('groups games by play status and shows progress', async () => {
    mockApi(SETUP);
    renderAt('/backlog');

    expect(await screen.findByText('1 of 5 cleared')).toBeInTheDocument();
    expect(screen.getByTestId('row-1')).toHaveTextContent('Hades');
    expect(screen.getByTestId('row-2')).toHaveTextContent('Okami');
    expect(screen.getByTestId('backlog-3')).toHaveTextContent('Celeste');
    expect(screen.queryByTestId('backlog-5')).toBeNull();
    expect(screen.getByText('2 games have no play status, so they are not in the backlog yet.')).toBeInTheDocument();
  });

  it('finishes a game from the playing row and starts one from the backlog', async () => {
    const calls = mockApi({
      ...SETUP,
      'PUT /api/games/1/play-status': statusPut(1),
      'PUT /api/games/3/play-status': statusPut(3),
    });
    renderAt('/backlog');

    fireEvent.click(within(await screen.findByTestId('row-1')).getByRole('button', { name: en.backlog.complete }));
    await waitFor(() => expect(calls.find((c) => c.path === '/api/games/1/play-status')?.body).toEqual({ play_status: 'completed' }));

    fireEvent.click(screen.getByRole('button', { name: 'Start playing Celeste' }));
    await waitFor(() => expect(calls.find((c) => c.path === '/api/games/3/play-status')?.body).toEqual({ play_status: 'playing' }));
  });

  it('sorts games without a status one at a time', async () => {
    const calls = mockApi({ ...SETUP, 'PUT /api/games/6/play-status': statusPut(6) });
    renderAt('/backlog');

    fireEvent.click(await screen.findByRole('button', { name: en.backlog.sort }));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByTestId('sort-game')).toHaveTextContent('Metroid');
    fireEvent.click(within(dialog).getByRole('button', { name: en.playStatus.backlog }));
    expect(within(dialog).getByTestId('sort-game')).toHaveTextContent('Kirby');
    fireEvent.click(within(dialog).getByRole('button', { name: en.backlog.skip }));
    expect(within(dialog).getByText(en.backlog.sortDone)).toBeInTheDocument();

    await waitFor(() => expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({ play_status: 'backlog' }));
    expect(calls.filter((c) => c.method === 'PUT')).toHaveLength(1);
  });

  it('picks a random backlog game and starts it', async () => {
    const calls = mockApi({
      ...SETUP,
      'POST /api/suggestions': { game: detail(GAMES[3] as GameSummary, 'backlog'), copy_ids: [] },
      'PUT /api/games/4/play-status': statusPut(4),
    });
    renderAt('/backlog');

    fireEvent.click(await screen.findByRole('button', { name: en.backlog.pick }));
    const picked = await screen.findByTestId('picked');
    expect(picked).toHaveTextContent('Tunic');
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({});

    fireEvent.click(within(picked).getByRole('button', { name: en.backlog.start }));
    await waitFor(() => expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({ play_status: 'playing' }));
    await waitFor(() => expect(screen.queryByTestId('picked')).toBeNull());
  });

  it('shows on hold games ten at a time', async () => {
    const paused = Array.from({ length: 23 }, (_, i) => game(100 + i, `Paused ${i}`, 'paused'));
    mockApi({ ...SETUP, 'GET /api/games': [...GAMES, ...paused] });
    renderAt('/backlog');

    expect(await screen.findByTestId('row-100')).toBeInTheDocument();
    expect(screen.queryByTestId('row-109')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show 10 more (14 left)' }));
    expect(screen.getByTestId('row-109')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Show 4 more (4 left)' }));
    expect(screen.getByTestId('row-122')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /more/ })).toBeNull();
  });

  it('says when nothing in the backlog matches', async () => {
    mockApi({ ...SETUP, 'POST /api/suggestions': json({ code: 'no_candidates' }, 404) });
    renderAt('/backlog');
    fireEvent.click(await screen.findByRole('button', { name: en.backlog.pick }));
    expect(await screen.findByText(en.errors.no_candidates)).toBeInTheDocument();
  });
});
