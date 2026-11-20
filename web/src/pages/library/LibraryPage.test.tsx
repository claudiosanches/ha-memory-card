import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameSummary, Platform } from '../../api/types';
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

describe('Library', () => {
  it('shows games with status and platform badges', async () => {
    mockApi({ 'GET /api/games': GAMES, 'GET /api/platforms': [SWITCH] });
    renderAt('/');

    const card = await screen.findByTestId('game-Mario Kart 8 Deluxe');
    expect(within(card).getByText(en.playStatus.playing)).toBeInTheDocument();
    await waitFor(() => expect(within(card).getByText('Switch ×2')).toBeInTheDocument());
    expect(within(card).getByText('2 copies')).toBeInTheDocument();
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
