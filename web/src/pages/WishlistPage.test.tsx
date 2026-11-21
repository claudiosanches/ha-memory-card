import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { CopyRow, GameDetail, GameSummary } from '../api/types';
import en from '../locales/en.json';
import { mockApi, VIEWER } from '../test/api';
import { renderAt } from '../test/render';

const SWITCH = {
  id: 1,
  name: 'Nintendo Switch',
  slug: 'switch',
  abbreviation: 'Switch',
  family: null,
  generation: null,
  igdb_platform_id: 130,
  custom: false,
  active: true,
  sort_order: 10,
  game_count: 1,
  owned_game_count: 1,
  wanted_game_count: 0,
  device_count: 0,
};

const WISH: CopyRow = {
  copy_id: 31,
  game_id: 8,
  title: 'Hades II',
  cover_path: null,
  play_status: null,
  release_id: 12,
  platform_id: 1,
  region: 'ww',
  release_title: null,
  product_code: null,
  status: 'wanted',
  format: 'physical',
  device_id: null,
  store_id: 3,
  condition: null,
  notes: 'Limited Run',
  edition: 'Collector\'s',
  price: '59.90',
  currency: 'USD',
};

function detail(id: number, title: string, copies: number): GameDetail {
  return {
    id,
    igdb_id: null,
    title,
    summary: null,
    cover_path: null,
    cover_uploaded: false,
    first_release_date: null,
    developers: [],
    publishers: [],
    play_status: null,
    local_max_players: null,
    online_max_players: null,
    local_coop: null,
    online_coop: null,
    game_mode_ids: [],
    genre_ids: [],
    created_at: '2026-11-01T10:00:00.000Z',
    sessions: [],
    versions: [],
    names: [],
    releases: [
      {
        id: 12,
        platform_id: 1,
        region: 'ww',
        title: null,
        product_code: null,
        release_date: null,
        cover_path: null,
        cover_uploaded: false,
        local_max_players: null,
        online_max_players: null,
        local_coop: null,
        online_coop: null,
        copies: Array.from({ length: copies }, (_, i) => ({
          id: 31 + i,
          status: 'wanted' as const,
          format: 'physical' as const,
          device_id: null,
          store_id: null,
          condition: null,
          notes: null,
          created_at: '2026-11-01T10:00:00.000Z',
          contents: [],
          photos: [],
          tags: [],
        })),
      },
    ],
  };
}

const CELESTE: GameSummary = {
  id: 5,
  title: 'Celeste',
  igdb_id: null,
  cover_path: null,
  play_status: 'completed',
  last_played_at: null,
  owned_copies: 1,
  wanted_copies: 0,
  platforms: [],
};

const SETUP = {
  'GET /api/platforms': [SWITCH],
  'GET /api/devices': [],
  'GET /api/stores': [{ id: 3, name: 'Amazon', notes: null, copy_count: 0 }],
  'GET /api/games': [CELESTE],
};

async function chooseCopy(dialog: HTMLElement) {
  fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.platform }));
  fireEvent.click(await screen.findByRole('option', { name: 'Nintendo Switch', hidden: true }));
  fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.region }));
  fireEvent.click(await screen.findByRole('option', { name: en.region.ww, hidden: true }));
}

describe('Wishlist', () => {
  it('lists wanted copies and moves one into the collection', async () => {
    const calls = mockApi({ ...SETUP, 'GET /api/copies': [WISH], 'PUT /api/copies/31': detail(8, 'Hades II', 1) });
    renderAt('/wishlist');

    const item = await screen.findByTestId('wish-31');
    expect(within(item).getByRole('link', { name: 'Hades II' })).toBeInTheDocument();
    await waitFor(() => expect(item).toHaveTextContent('Buy at Amazon · $59.90'));
    expect(item).toHaveTextContent("Physical · Collector's");
    expect(item).toHaveTextContent('Limited Run');
    expect(calls.find((c) => c.path === '/api/copies')?.query.get('ownership')).toBe('wanted');

    fireEvent.click(within(item).getByRole('button', { name: en.game.gotIt }));
    const dialog = await screen.findByRole('dialog', { name: en.game.gotItTitle });
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    expect(calls.find((c) => c.method === 'PUT')?.body).toMatchObject({ status: 'owned', store_id: 3, notes: 'Limited Run' });
  });

  it('adds a new game to the wishlist only once its copy is saved', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/copies': [],
      'POST /api/games': detail(9, 'Silksong', 0),
      'POST /api/games/9/copies': detail(9, 'Silksong', 1),
    });
    renderAt('/wishlist');

    fireEvent.click(await screen.findByRole('button', { name: en.wishlist.add }));
    fireEvent.change(await screen.findByRole('textbox', { name: en.game.title }), { target: { value: 'Silksong' } });
    fireEvent.click(screen.getByRole('button', { name: en.wishlist.next }));

    const dialog = await screen.findByRole('dialog', { name: 'Add Silksong to your wishlist' });
    expect(calls.some((c) => c.method === 'POST')).toBe(false);
    expect(within(dialog).queryByRole('combobox', { name: en.game.device })).not.toBeInTheDocument();
    expect(within(dialog).getByRole('combobox', { name: en.game.buyAt })).toBeInTheDocument();
    await chooseCopy(dialog);
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/9/copies')).toBe(true));
    expect(calls.find((c) => c.method === 'POST' && c.path === '/api/games')?.body).toEqual({ title: 'Silksong', play_status: null });
    expect(calls.find((c) => c.path === '/api/games/9/copies')?.body).toMatchObject({ status: 'wanted', platform_id: 1, region: 'ww' });
  });

  it('puts the copy on a game already in the collection', async () => {
    const calls = mockApi({ ...SETUP, 'GET /api/copies': [], 'POST /api/games/5/copies': detail(5, 'Celeste', 1) });
    renderAt('/wishlist');

    fireEvent.click(await screen.findByRole('button', { name: en.wishlist.add }));
    fireEvent.change(await screen.findByRole('textbox', { name: en.game.title }), { target: { value: 'celeste' } });
    expect(await screen.findByTestId('wish-existing')).toHaveTextContent('Celeste');
    fireEvent.click(screen.getByRole('button', { name: en.wishlist.next }));
    const dialog = await screen.findByRole('dialog', { name: 'Add Celeste to your wishlist' });
    await chooseCopy(dialog);
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/5/copies')).toBe(true));
    expect(calls.some((c) => c.method === 'POST' && c.path === '/api/games')).toBe(false);
  });

  it('offers to delete a game whose only copy leaves the wishlist', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/copies': [WISH],
      'GET /api/games/8': detail(8, 'Hades II', 1),
      'DELETE /api/games/8': new Response(null, { status: 204 }),
    });
    renderAt('/wishlist');

    fireEvent.click(within(await screen.findByTestId('wish-31')).getByRole('button', { name: en.wishlist.remove }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(await within(dialog).findByRole('button', { name: en.game.deleteGameToo }));
    await waitFor(() => expect(calls.some((c) => c.method === 'DELETE' && c.path === '/api/games/8')).toBe(true));
  });

  it('shows viewers the wishlist without changes', async () => {
    mockApi({ ...SETUP, 'GET /api/me': VIEWER, 'GET /api/copies': [WISH] });
    renderAt('/wishlist');

    const item = await screen.findByTestId('wish-31');
    expect(within(item).queryByRole('button')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: en.wishlist.add })).not.toBeInTheDocument();
  });
});
