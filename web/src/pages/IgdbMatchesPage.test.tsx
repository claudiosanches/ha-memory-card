import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { MatchGroup } from '../api/types';
import en from '../locales/en.json';
import { json, mockApi } from '../test/api';
import { renderAt } from '../test/render';

const hit = (igdb_id: number, name: string, score: number) => ({ igdb_id, name, year: 2017, platforms: ['Switch'], platform_ids: [130], game_id: null, score });

const GROUPS: MatchGroup[] = [
  {
    game_id: 3,
    title: 'Super Mario Odysey',
    platforms: ['Nintendo Switch'],
    candidates: [hit(1, 'Super Mario Odyssey', 97), hit(9, 'Super Mario Odyssey 64', 88)],
  },
  { game_id: 5, title: 'Homebrew Thing', platforms: [], candidates: [hit(40, 'Something Else', 30)] },
];

describe('IGDB match review', () => {
  it('links the closest match and skips games IGDB does not have', async () => {
    const calls = mockApi({
      'GET /api/igdb/matches': GROUPS,
      'POST /api/games/3/igdb': { id: 3 },
      'POST /api/games/5/igdb-skip': new Response(null, { status: 204 }),
    });
    renderAt('/igdb-matches');

    const first = await screen.findByTestId('match-3');
    expect(within(first).getByRole('radio', { name: /^Super Mario Odyssey 2017/ })).toBeChecked();
    fireEvent.click(within(first).getByRole('button', { name: en.matches.link }));
    await waitFor(() => expect(screen.queryByTestId('match-3')).not.toBeInTheDocument());
    expect(calls.find((c) => c.path === '/api/games/3/igdb')?.body).toEqual({
      igdb_id: 1,
      overwrite: false,
      unlinked_title: 'Super Mario Odysey',
    });

    const second = screen.getByTestId('match-5');
    expect(within(second).getByRole('button', { name: en.matches.link })).toBeDisabled();
    fireEvent.click(within(second).getByRole('button', { name: en.matches.notOnIgdb }));
    await waitFor(() => expect(screen.queryByTestId('match-5')).not.toBeInTheDocument());
    expect(calls.some((c) => c.path === '/api/games/5/igdb-skip')).toBe(true);
    expect(screen.getByText(en.matches.none)).toBeInTheDocument();
  });

  it('drops a card whose game changed since the page loaded', async () => {
    mockApi({
      'GET /api/igdb/matches': GROUPS,
      'POST /api/games/3/igdb': json({ code: 'igdb_changed' }, 409),
    });
    renderAt('/igdb-matches');

    fireEvent.click(within(await screen.findByTestId('match-3')).getByRole('button', { name: en.matches.link }));
    expect(await screen.findByText(en.errors.igdb_changed)).toBeInTheDocument();
    expect(screen.queryByTestId('match-3')).not.toBeInTheDocument();
    expect(screen.getByTestId('match-5')).toBeInTheDocument();
  });
});
