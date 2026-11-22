import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Stats } from '../../api/types';
import en from '../../locales/en.json';
import { mockApi } from '../../test/api';
import { renderAt } from '../../test/render';
import { calendarWeeks, firstWeekday } from './PlayCalendar';

function daysAgo(days: number): string {
  const date = new Date();
  date.setDate(date.getDate() - days);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

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
    { play_status: 'completed', games: 3 },
    { play_status: 'backlog', games: 4 },
    { play_status: null, games: 5 },
  ],
  platforms: [
    { platform_id: 1, games: 3, copies: 4, physical: 4, digital: 0 },
    { platform_id: 2, games: 9, copies: 11, physical: 5, digital: 6 },
  ],
  stores: [{ store_id: null, copies: 15 }],
  regions: [
    { region: 'na', copies: 10 },
    { region: 'jp', copies: 5 },
  ],
  conditions: [
    { condition: 'cib', copies: 6 },
    { condition: null, copies: 3 },
  ],
  added_by_month: [],
  added_by_year: [
    { period: '2023', copies: 5 },
    { period: '2025', copies: 10 },
  ],
  value_change: [{ currency: 'USD', paid: '300.00', estimated: '450.00', change: '150.00', copies: 6 }],
  most_valuable: [
    { copy_id: 7, game_id: 4, title: 'Hades', platform_id: 2, currency: 'USD', estimated: '200.00', paid: '20.00' },
  ],
  play_days: [{ day: daysAgo(2), sessions: 3, minutes: 150 }],
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

function section(name: string) {
  return screen.getByRole('heading', { name }).closest('section')!;
}

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

    const platforms = section(en.stats.byPlatform);
    await within(platforms).findByText('PlayStation 5');
    const rows = within(platforms).getAllByRole('listitem');
    expect(rows[0]).toHaveTextContent('PlayStation 5');
    expect(rows[0]).toHaveTextContent('11 copies · 73%');
    expect(rows[1]).toHaveTextContent('Nintendo Switch');
    expect(within(section(en.stats.byRegion)).getAllByRole('listitem')[1]).toHaveTextContent('Japan5 copies · 33%');
    expect(within(section(en.stats.byCondition)).getAllByRole('listitem')[0]).toHaveTextContent('Complete in box');
    expect(within(section(en.stats.formatByPlatform)).getByText('5 physical, 6 digital')).toBeInTheDocument();
    expect(within(section(en.stats.byStatus)).getByText(en.common.notSet)).toBeInTheDocument();
    expect(screen.getAllByRole('link', { name: 'Hades' })).toHaveLength(2);
  });

  it('groups play first, then the collection, then money', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    await screen.findByTestId('stats-value');
    const main = screen.getByRole('main');
    expect(within(main).getAllByRole('heading', { level: 2 }).map((h) => h.textContent)).toEqual([
      en.stats.groupPlay,
      en.stats.groupCollection,
      en.stats.groupMoney,
    ]);
    expect(within(main).getAllByRole('heading', { level: 3 })[0]).toHaveTextContent(en.stats.progress);
  });

  it('links each part to the games it counts', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    await screen.findByTestId('stats-value');
    const platforms = section(en.stats.byPlatform);
    const ps5 = await within(platforms).findByRole('link', { name: 'PlayStation 5' });
    expect(decodeURIComponent(ps5.getAttribute('href')!)).toContain('platform=[2]');
    expect(within(section(en.stats.byStore)).queryByRole('link')).not.toBeInTheDocument();
    const japan = within(section(en.stats.byRegion)).getByRole('link', { name: en.region.jp });
    expect(decodeURIComponent(japan.getAttribute('href')!)).toContain('region=["jp"]');
    const notSet = within(section(en.stats.byStatus)).getByRole('link', { name: en.common.notSet });
    expect(decodeURIComponent(notSet.getAttribute('href')!)).toContain('play_status=["none"]');
    const main = screen.getByRole('main');
    expect(within(main).getByRole('link', { name: en.stats.wishlist })).toHaveAttribute('href', expect.stringContaining('/wishlist'));
  });

  it('shows how much of the backlog is finished', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    await screen.findByTestId('stats-value');
    const progress = section(en.stats.progress);
    expect(await within(progress).findByText('25%')).toBeInTheDocument();
    expect(within(progress).getByText('3 of 12 games finished')).toBeInTheDocument();
    const groups = within(progress).getAllByRole('listitem');
    expect(groups.map((row) => row.textContent)).toEqual([
      `${en.stats.groups.finished}3 games · 25%`,
      `${en.stats.groups.notStarted}4 games · 33%`,
      `${en.stats.groups.notSet}5 games · 42%`,
    ]);
  });

  it('compares what copies cost with what they are worth', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    const change = await screen.findByTestId('value-change');
    expect(change).toHaveTextContent('$300.00');
    expect(change).toHaveTextContent('$450.00');
    expect(change).toHaveTextContent('+$150.00 +50%');
    const valuable = section(en.stats.valueTitle);
    expect(within(valuable).getByText('$200.00 · cost $20.00')).toBeInTheDocument();
  });

  it('counts copies added by month or by year', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    const added = await screen.findByRole('list', { name: en.stats.added });
    const years = within(added).getAllByRole('listitem');
    expect(years.map((y) => y.getAttribute('aria-label'))).toEqual(['2023: 5 copies', '2024: 0 copies', '2025: 10 copies']);
    fireEvent.click(screen.getByText(en.stats.lastYear));
    expect(within(screen.getByRole('list', { name: en.stats.added })).getAllByRole('listitem')).toHaveLength(12);
  });

  it('draws a year of play as a calendar', async () => {
    mockApi({ 'GET /api/stats': STATS, 'GET /api/platforms': PLATFORMS, 'GET /api/stores': [] });
    renderAt('/stats');

    expect(await screen.findByRole('img', { name: '3 sessions on 1 day' })).toBeInTheDocument();
  });

  it('says plainly when nothing is priced or played yet', async () => {
    mockApi({
      'GET /api/stats': {
        ...STATS,
        spent: [],
        spent_by_year: [],
        estimated: [],
        value_change: [],
        most_valuable: [],
        total_sessions: 0,
        total_minutes: 0,
        play_days: [],
        most_played: [],
      },
      'GET /api/platforms': PLATFORMS,
      'GET /api/stores': [],
    });
    renderAt('/stats');

    const paid = await screen.findByTestId('stats-value');
    expect(paid).toHaveTextContent(en.stats.noPrices);
    expect(paid).toHaveTextContent('4 copies without a price');
    expect(screen.getByTestId('stats-estimated')).toHaveTextContent('6 copies without an estimate');
    expect(within(section(en.stats.activity)).queryByText('0 minutes')).not.toBeInTheDocument();
    expect(screen.getByText(en.stats.noSessions)).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: en.stats.valueTitle })).not.toBeInTheDocument();
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

describe('Play calendar', () => {
  it('ends on today and starts each week on the locale first day', () => {
    const today = new Date(2026, 10, 22); // a Sunday
    const weeks = calendarWeeks(today, 1, new Map());
    expect(weeks).toHaveLength(53);
    expect(weeks[0]![0]!.date.getDay()).toBe(1);
    const last = weeks.at(-1)!;
    expect(last[6]!.iso).toBe('2026-11-22');
    expect(calendarWeeks(today, 0, new Map()).at(-1)!.filter(Boolean)).toHaveLength(1);
  });

  it('falls back to Monday when the browser cannot tell', () => {
    expect([0, 1, 6]).toContain(firstWeekday('en-US'));
    expect(firstWeekday('not a locale')).toBe(1);
  });
});
