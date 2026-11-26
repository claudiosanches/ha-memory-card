import { fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { GameDetail, MissingDetails } from '../../api/types';
import en from '../../locales/en.json';
import { mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

function detail(id: number, title: string): GameDetail {
  return {
    id,
    igdb_id: null,
    title,
    summary: null,
    cover_path: 'game-1/cover.jpg',
    cover_uploaded: false,
    first_release_date: null,
    developers: [],
    publishers: [],
    play_status: null,
    local_max_players: null,
    online_max_players: null,
    local_coop: null,
    online_coop: null,
    time_to_beat: {},
    game_mode_ids: [],
    genre_ids: [],
    labels: [],
    created_at: '2026-11-01T10:00:00.000Z',
    sessions: [],
    household: [],
    versions: [],
    names: [],
    releases: [
      {
        id: 9,
        platform_id: 1,
        region: 'eu',
        title: null,
        product_code: null,
        release_date: null,
        cover_path: null,
        cover_uploaded: false,
        local_max_players: null,
        online_max_players: null,
        local_coop: null,
        online_coop: null,
        copies: [
          { id: 21, status: 'owned', format: 'physical', device_id: null, store_id: null, condition: 'cib', notes: 'Spare', estimated_value: '40.00', estimated_currency: 'EUR', source: 'manual', created_at: '2026-11-01T10:00:00.000Z', contents: [], photos: [], tags: [] },
        ],
      },
    ],
  };
}

const MISSING: MissingDetails[] = [
  { game_id: 4, title: 'Metroid Dread', fields: ['store'] },
  { game_id: 5, title: 'Tunic', fields: ['cover'] },
];

const SETUP = {
  'GET /api/missing-details': MISSING,
  'GET /api/games/4': detail(4, 'Metroid Dread'),
  'GET /api/games/5': detail(5, 'Tunic'),
  'GET /api/platforms': [],
  'GET /api/devices': [],
  'GET /api/stores': [{ id: 3, name: 'Amazon', notes: null, copy_count: 0 }],
  'GET /api/game-modes': [],
};

describe('Setup wizard', () => {
  afterEach(() => localStorage.clear());

  it('fills in what a game lacks, then moves to the next one', async () => {
    const calls = mockApi({
      ...SETUP,
      'PUT /api/games/4': (body: unknown) => ({ ...detail(4, 'Metroid Dread'), ...(body as object) }),
      'PUT /api/copies/21': () => detail(4, 'Metroid Dread'),
    });
    renderAt('/cleanup?tab=missing');

    fireEvent.click(await screen.findByRole('button', { name: en.setup.start_other.replace('{{count}}', '2') }));
    expect(await screen.findByText('Metroid Dread')).toBeInTheDocument();
    const missing = screen.getByTestId('missing-fields');
    expect(within(missing).getByText(en.setup.fields.store)).toBeInTheDocument();

    const copy = screen.getByTestId('setup-copy-21');
    expect(within(copy).queryByRole('combobox', { name: en.game.device })).not.toBeInTheDocument();
    fireEvent.click(within(copy).getByRole('combobox', { name: en.game.store }));
    fireEvent.click(await screen.findByRole('option', { name: 'Amazon', hidden: true }));
    fireEvent.click(screen.getByRole('button', { name: en.setup.saveNext }));

    expect(await screen.findByText('Tunic')).toBeInTheDocument();
    expect(calls.some((c) => c.method === 'PUT' && c.path === '/api/games/4')).toBe(false);
    const copyBody = calls.find((c) => c.path === '/api/copies/21')?.body;
    expect(copyBody).toMatchObject({ platform_id: 1, region: 'eu', format: 'physical', store_id: 3, condition: 'cib', notes: 'Spare', estimated_value: '40.00', estimated_currency: 'EUR' });

    fireEvent.click(screen.getByRole('button', { name: en.setup.skip }));
    expect(await screen.findByText(en.setup.done)).toBeInTheDocument();
  });

  it('keeps the account of a digital copy it saves', async () => {
    const base = detail(4, 'Metroid Dread');
    const game: GameDetail = {
      ...base,
      releases: base.releases.map((release) => ({
        ...release,
        copies: release.copies.map((copy) => ({ ...copy, format: 'digital' as const, condition: null, account_id: 7 })),
      })),
    };
    const calls = mockApi({
      ...SETUP,
      'GET /api/missing-details': [MISSING[0]],
      'GET /api/games/4': game,
      'PUT /api/copies/21': () => game,
    });
    renderAt('/cleanup?tab=missing');

    fireEvent.click(await screen.findByRole('button', { name: en.setup.start_one.replace('{{count}}', '1') }));
    const copy = await screen.findByTestId('setup-copy-21');
    fireEvent.click(within(copy).getByRole('combobox', { name: en.game.store }));
    fireEvent.click(await screen.findByRole('option', { name: 'Amazon', hidden: true }));
    fireEvent.click(screen.getByRole('button', { name: en.setup.saveNext }));

    expect(await screen.findByText(en.setup.done)).toBeInTheDocument();
    expect(calls.find((c) => c.path === '/api/copies/21')?.body).toMatchObject({ account_id: 7, store_id: 3 });
  });

  it('only goes through games lacking the chosen details', async () => {
    mockApi(SETUP);
    renderAt('/cleanup?tab=missing');

    fireEvent.click(await screen.findByRole('checkbox', { name: /^Cover/ }));
    expect(screen.getByRole('button', { name: en.setup.start_one.replace('{{count}}', '1') })).toBeInTheDocument();
    expect(localStorage.getItem('memory-card-setup-fields')).not.toContain('"cover"');
  });
});
