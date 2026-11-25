import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { TitleDifference } from '../../api/types';
import en from '../../locales/en.json';
import { MANAGER, mockApi, VIEWER } from '../../test/api';
import { renderAt } from '../../test/render';

const DIFFERENCES: TitleDifference[] = [
  { game_id: 3, title: 'Bomberman 93', igdb_title: "Bomberman '93", cover_path: null },
  { game_id: 8, title: 'de Blob 2', igdb_title: 'De Blob 2', cover_path: null },
];

const STATUS = {
  version: '0.1.0',
  igdb_configured: true,
  external_services: true,
  sensors: 'connected',
  tags: 'unavailable',
};

describe('Cleanup access and tabs', () => {
  it('shows Missing details, Duplicates, and IGDB tabs to managers when IGDB is configured', async () => {
    mockApi({
      'GET /api/me': MANAGER,
      'GET /api/status': STATUS,
      'GET /api/missing-details': [],
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': [],
    });
    renderAt('/cleanup');

    expect(await screen.findByRole('tab', { name: en.cleanup.tabs.missing })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('tab', { name: en.cleanup.tabs.duplicates })).toBeInTheDocument();
    fireEvent.click(await screen.findByRole('tab', { name: en.cleanup.tabs.igdb }));
    expect(await screen.findByRole('button', { name: en.igdb.syncButton })).toBeInTheDocument();
  });

  it('shows only Duplicates to viewers and keeps manager cleanup tools away', async () => {
    const calls = mockApi({
      'GET /api/me': VIEWER,
      'GET /api/status': STATUS,
      'GET /api/duplicates': [],
      'GET /api/platforms': [],
      'GET /api/devices': [],
      'GET /api/stores': [],
    });
    renderAt('/cleanup?tab=igdb');

    expect(await screen.findByText(en.duplicates.empty)).toBeInTheDocument();
    expect(screen.queryByRole('tab')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: en.igdb.syncButton })).not.toBeInTheDocument();
    expect(calls.some((c) => c.path.startsWith('/api/igdb'))).toBe(false);
    expect(calls.some((c) => c.path === '/api/missing-details')).toBe(false);
  });

  it('redirects legacy /settings?tab=igdb links to /cleanup?tab=igdb', async () => {
    mockApi({
      'GET /api/status': STATUS,
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': [],
    });
    renderAt('/settings?tab=igdb');

    expect(await screen.findByRole('button', { name: en.igdb.syncButton })).toBeInTheDocument();
  });
});

describe('Cleanup IGDB titles', () => {
  it('renames a game to its IGDB title', async () => {
    const calls = mockApi({
      'GET /api/status': STATUS,
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': DIFFERENCES,
      'POST /api/igdb/titles': [DIFFERENCES[1]],
    });
    renderAt('/cleanup?tab=igdb');

    const row = await screen.findByTestId('title-3');
    expect(within(row).getByText("IGDB: Bomberman '93")).toBeInTheDocument();
    expect(screen.getByText(en.igdb.titlesText_other.split('{{count}}').join('2'))).toBeInTheDocument();
    fireEvent.click(within(row).getByRole('button', { name: en.igdb.useIgdbTitle }));

    await waitFor(() => expect(screen.queryByTestId('title-3')).toBeNull());
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ use_igdb: [3] });
    expect(screen.getByTestId('title-8')).toBeInTheDocument();
  });

  it('sends every IGDB title in batches the API takes', async () => {
    const many: TitleDifference[] = Array.from({ length: 1001 }, (_, i) => ({
      game_id: i + 1,
      title: `Game ${i + 1}`,
      igdb_title: `IGDB ${i + 1}`,
      cover_path: null,
    }));
    const calls = mockApi({
      'GET /api/status': STATUS,
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': many,
      'POST /api/igdb/titles': [],
    });
    renderAt('/cleanup?tab=igdb');

    fireEvent.click(await screen.findByRole('button', { name: en.igdb.useAllIgdbTitles }));
    await waitFor(() => expect(calls.filter((c) => c.method === 'POST')).toHaveLength(2));
    const sent = calls.filter((c) => c.method === 'POST').map((c) => (c.body as { use_igdb: number[] }).use_igdb);
    expect(sent.map((ids) => ids.length)).toEqual([1000, 1]);
    expect(sent[1]).toEqual([1001]);
  });
});
