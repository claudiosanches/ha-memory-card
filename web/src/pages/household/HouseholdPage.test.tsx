import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import { mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

const HOUSEHOLD = [
  {
    person: { id: 'b2', name: 'Ana' },
    sessions: 4,
    minutes: 180,
    days: 3,
    games: 2,
    finished: 1,
    top_game: { game_id: 7, title: 'Tetris', cover_path: 'game-7/cover.jpg', sessions: 3 },
  },
  {
    person: { id: 'manager-id', name: 'Alex' },
    sessions: 1,
    minutes: 30,
    days: 1,
    games: 1,
    finished: 0,
    top_game: null,
  },
  {
    person: { id: 'c3', name: 'Sam' },
    sessions: 0,
    minutes: 0,
    days: 0,
    games: 0,
    finished: 0,
    top_game: null,
  },
];

describe('Household page', () => {
  afterEach(() => localStorage.clear());

  it("ranks everyone's play, summarizes totals, and marks yours", async () => {
    const calls = mockApi({ 'GET /api/household': HOUSEHOLD });
    renderAt('/household');

    const summary = await screen.findByTestId('household-summary');
    expect(within(summary).getByText('4 hours')).toBeInTheDocument();
    expect(within(summary).getByText('2 of 3 playing')).toBeInTheDocument();
    expect(within(summary).getByText('5')).toBeInTheDocument();

    const ana = screen.getByTestId('person-b2');
    expect(within(ana).getByText('Ana')).toBeInTheDocument();
    expect(within(ana).getByText('3 hours')).toBeInTheDocument();
    expect(within(ana).getByText('4')).toBeInTheDocument();
    expect(within(ana).getByText('3')).toBeInTheDocument();
    expect(within(ana).getByRole('link', { name: 'Tetris' })).toHaveAttribute('href', expect.stringContaining('/games/7'));
    expect(within(ana).getByText('3 sessions')).toBeInTheDocument();
    expect(within(ana).queryByText(en.household.you)).not.toBeInTheDocument();

    const mine = screen.getByTestId('person-manager-id');
    expect(within(mine).getByText(en.household.you)).toBeInTheDocument();
    expect(mine).toHaveAttribute('data-you');

    const sam = screen.getByTestId('person-c3');
    expect(within(sam).getByText(en.household.noPlay)).toBeInTheDocument();
    expect(sam).toHaveAttribute('data-quiet');

    expect(calls.find((c) => c.path === '/api/household')?.query.get('period')).toBe('month');

    fireEvent.click(screen.getByText(en.household.periods.all));
    await waitFor(() =>
      expect(calls.some((c) => c.path === '/api/household' && c.query.get('period') === 'all')).toBe(true),
    );
  });

  it('says when nobody played in the period', async () => {
    mockApi({ 'GET /api/household': [] });
    renderAt('/household');
    expect(await screen.findByText(en.household.empty)).toBeInTheDocument();
  });
});
