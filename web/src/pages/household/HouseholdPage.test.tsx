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
    top_game: { game_id: 7, title: 'Tetris', sessions: 3 },
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
];

describe('Household page', () => {
  afterEach(() => localStorage.clear());

  it("ranks everyone's play and marks yours", async () => {
    const calls = mockApi({ 'GET /api/household': HOUSEHOLD });
    renderAt('/household');

    const ana = await screen.findByTestId('person-b2');
    expect(within(ana).getByText('Ana')).toBeInTheDocument();
    expect(within(ana).getByText('3 hours')).toBeInTheDocument();
    expect(within(ana).getByText('4 sessions · 3 days · 2 games · 1 finished')).toBeInTheDocument();
    expect(within(ana).getByRole('link', { name: 'Tetris' })).toHaveAttribute('href', expect.stringContaining('/games/7'));
    expect(within(ana).queryByText(en.household.you)).not.toBeInTheDocument();
    const mine = screen.getByTestId('person-manager-id');
    expect(within(mine).getByText(en.household.you)).toBeInTheDocument();
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
