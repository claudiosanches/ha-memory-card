import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameDetail, GameSummary } from '../../api/types';
import en from '../../locales/en.json';
import { BUILTIN_STATUSES, json, mockApi, VIEWER } from '../../test/api';
import { renderAt } from '../../test/render';

function game(id: number, title: string, play_status: string | null): GameSummary {
  return {
    id,
    title,
    igdb_id: null,
    cover_path: null,
    play_status,
    last_played_at: null,
    owned_copies: 1,
    wanted_copies: 0,
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

function detail(summary: GameSummary, play_status: string | null): GameDetail {
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
    local_max_players: null,
    online_max_players: null,
    local_coop: null,
    online_coop: null,
    game_mode_ids: [],
    genre_ids: [],
    created_at: '2026-11-01T10:00:00.000Z',
    sessions: [],
    releases: [],
  };
}

const SETUP = {
  'GET /api/games': GAMES,
  'GET /api/platforms': [],
  'GET /api/devices': [],
};

const statusPut = (id: number) => (body: unknown) =>
  detail(GAMES.find((g) => g.id === id) as GameSummary, (body as { play_status: string }).play_status);

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

  it('groups custom statuses by what they count as', async () => {
    mockApi({
      ...SETUP,
      'GET /api/games': [...GAMES, game(8, 'Okami HD', 'replaying'), game(9, 'Pong', 'collected')],
      'GET /api/play-statuses': [
        ...BUILTIN_STATUSES,
        { key: 'replaying', label: 'Replaying', color: 'teal', counts_as: 'playing', builtin: false, game_count: 1 },
        { key: 'collected', label: 'Collected', color: 'gray', counts_as: 'not_counted', builtin: false, game_count: 1 },
      ],
    });
    renderAt('/backlog');

    expect(await screen.findByText('1 of 6 cleared')).toBeInTheDocument();
    expect(screen.getByTestId('row-8')).toHaveTextContent('Replaying');
    expect(screen.getByTestId('row-1')).not.toHaveTextContent('Playing');
    expect(screen.queryByText('Pong')).toBeNull();
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

  it('sorts with number keys and skips with S', async () => {
    const calls = mockApi({ ...SETUP, 'PUT /api/games/6/play-status': statusPut(6) });
    renderAt('/backlog');

    fireEvent.click(await screen.findByRole('button', { name: en.backlog.sort }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.keyDown(document.documentElement, { key: '2', code: 'Digit2' });
    expect(within(dialog).getByTestId('sort-game')).toHaveTextContent('Kirby');
    fireEvent.keyDown(document.documentElement, { key: 's', code: 'KeyS' });
    expect(within(dialog).getByText(en.backlog.sortDone)).toBeInTheDocument();

    await waitFor(() => expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({ play_status: 'playing' }));
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

  it('keeps the current pick on screen while the next one loads', async () => {
    let finish = () => {};
    const rolls = ['Tunic', 'Celeste'];
    mockApi({
      ...SETUP,
      'POST /api/suggestions': async () => {
        const title = rolls.shift();
        if (title === 'Celeste') await new Promise<void>((resolve) => (finish = resolve));
        const summary = GAMES.find((g) => g.title === title) as GameSummary;
        return { game: detail(summary, 'backlog'), copy_ids: [] };
      },
    });
    renderAt('/backlog');

    fireEvent.click(await screen.findByRole('button', { name: en.backlog.pick }));
    expect(await screen.findByTestId('picked')).toHaveTextContent('Tunic');
    fireEvent.click(screen.getByRole('button', { name: en.backlog.pickAgain }));
    await waitFor(() => expect(screen.getByTestId('picked')).toHaveAttribute('data-rolling'));
    expect(screen.getByTestId('picked')).toHaveTextContent('Tunic');
    expect(screen.getByRole('button', { name: en.backlog.pickAgain })).toBeDisabled();

    finish();
    await waitFor(() => expect(screen.getByTestId('picked')).toHaveTextContent('Celeste'));
    expect(screen.getByTestId('picked')).not.toHaveAttribute('data-rolling');
  });

  it('lets a viewer browse but not pick or change a status', async () => {
    const calls = mockApi({ ...SETUP, 'GET /api/me': VIEWER });
    renderAt('/backlog');

    expect(await screen.findByTestId('row-1')).toBeInTheDocument();
    expect(screen.queryByText(en.backlog.unsortedTitle)).not.toBeInTheDocument();
    expect(within(screen.getByTestId('row-1')).queryByRole('button')).not.toBeInTheDocument();
    expect(within(screen.getByTestId('backlog-3')).queryByRole('button')).not.toBeInTheDocument();

    expect(screen.queryByRole('button', { name: en.backlog.pick })).not.toBeInTheDocument();
    expect(calls.filter((c) => c.method !== 'GET')).toEqual([]);
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
