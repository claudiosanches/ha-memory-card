import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { TitleDifference } from '../api/types';
import en from '../locales/en.json';
import { mockApi } from '../test/api';
import { renderAt } from '../test/render';

const DIFFERENCES: TitleDifference[] = [
  { game_id: 3, title: 'Bomberman 93', igdb_title: "Bomberman '93", cover_path: null },
  { game_id: 8, title: 'de Blob 2', igdb_title: 'De Blob 2', cover_path: null },
];

describe('Settings titles', () => {
  it('renames a game to its IGDB title', async () => {
    const calls = mockApi({
      'GET /api/status': { version: '0.1.0', igdb_configured: true, games: 2, owned_copies: 2 },
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': DIFFERENCES,
      'POST /api/igdb/titles': [DIFFERENCES[1]],
    });
    renderAt('/settings');

    const row = await screen.findByTestId('title-3');
    expect(within(row).getByText("IGDB: Bomberman '93")).toBeInTheDocument();
    expect(screen.getByText(en.igdb.titlesText_other.split('{{count}}').join('2'))).toBeInTheDocument();
    fireEvent.click(within(row).getByRole('button', { name: en.igdb.useIgdbTitle }));

    await waitFor(() => expect(screen.queryByTestId('title-3')).toBeNull());
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ use_igdb: [3] });
    expect(screen.getByTestId('title-8')).toBeInTheDocument();
  });
});
