import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Stats } from '../../api/types';
import en from '../../locales/en.json';
import { mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

const STATS: Stats = {
  games: 12,
  copies: 15,
  physical: 9,
  digital: 6,
  wishlist: 2,
  spent: [
    { currency: 'USD', amount: '420.50', copies: 10 },
    { currency: 'BRL', amount: '199.90', copies: 1 },
  ],
  unpriced: 4,
  wishlist_value: [{ currency: 'USD', amount: '60.00', copies: 1 }],
  estimated: [{ currency: 'USD', amount: '900.00', copies: 9 }],
  unvalued: 6,
  spent_by_year: [
    { year: '2025', currency: 'USD', amount: '120.50', copies: 4 },
    { year: '2026', currency: 'USD', amount: '300.00', copies: 6 },
  ],
  play_statuses: [
    { play_status: 'backlog', games: 7 },
    { play_status: null, games: 5 },
  ],
  platforms: [
    { platform_id: 1, games: 3, copies: 4 },
    { platform_id: 2, games: 9, copies: 11 },
  ],
  stores: [{ store_id: null, copies: 15 }],
  sessions_by_month: [],
  total_sessions: 3,
  total_minutes: 150,
  most_played: [{ game_id: 4, title: 'Hades', sessions: 3, minutes: 150 }],
};

const PLATFORMS = [1, 2].map((id) => ({
  id,
  name: id === 1 ? 'Nintendo Switch' : 'PlayStation 5',
  slug: `p${id}`,
  abbreviation: null,
  family: null,
  generation: null,
  igdb_platform_id: null,
  active: true,
  sort_order: id,
  game_count: 1,
  owned_game_count: 1,
  wanted_game_count: 0,
  device_count: 0,
}));

describe('Stats page', () => {
  it('totals the collection, its value and play time', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    const value = await screen.findByTestId('stats-value');
    expect(within(value).getByText('$420.50')).toBeInTheDocument();
    expect(within(value).getByText(/R\$\s?199\.90 · 4 copies without a price/)).toBeInTheDocument();
    const estimated = screen.getByTestId('stats-estimated');
    expect(within(estimated).getByText('$900.00')).toBeInTheDocument();
    expect(within(estimated).getByText('6 copies without an estimate')).toBeInTheDocument();
    expect(screen.getByText('9 physical, 6 digital')).toBeInTheDocument();
    expect(screen.getByText('$60.00 expected')).toBeInTheDocument();
    expect(screen.getByText('3 hours')).toBeInTheDocument();

    const platforms = screen.getByRole('heading', { name: en.stats.byPlatform }).closest('section')!;
    await within(platforms).findByText('PlayStation 5');
    const rows = within(platforms).getAllByRole('listitem');
    expect(rows[0]).toHaveTextContent('PlayStation 5');
    expect(rows[0]).toHaveTextContent('11 copies');
    expect(within(screen.getByRole('heading', { name: en.stats.byStatus }).closest('section')!).getByText(en.common.notSet)).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Hades' })).toBeInTheDocument();
    expect(screen.getAllByRole('listitem', { name: /sessions?$/ })).toHaveLength(12);
  });

  it('says when there is nothing to count', async () => {
    mockApi({
      'GET /api/stats': { ...STATS, copies: 0, wishlist: 0 },
      'GET /api/platforms': PLATFORMS,
      'GET /api/stores': [],
    });
    renderAt('/stats');

    expect(await screen.findByText(en.stats.empty)).toBeInTheDocument();
  });
});
