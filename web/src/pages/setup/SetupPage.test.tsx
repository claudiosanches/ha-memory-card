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
    game_mode_ids: [],
    genre_ids: [],
    created_at: '2026-11-01T10:00:00.000Z',
    sessions: [],
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
          { id: 21, status: 'owned', format: 'physical', device_id: null, store_id: null, condition: 'cib', notes: 'Spare', created_at: '2026-11-01T10:00:00.000Z', contents: [], photos: [], tags: [] },
        ],
      },
    ],
  };
}

const MISSING: MissingDetails[] = [
  { game_id: 4, title: 'Metroid Dread', fields: ['play_status', 'store'] },
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
    renderAt('/setup');

    fireEvent.click(await screen.findByRole('button', { name: en.setup.start_other.replace('{{count}}', '2') }));
    expect(await screen.findByText('Metroid Dread')).toBeInTheDocument();
    const missing = screen.getByTestId('missing-fields');
    expect(within(missing).getByText(en.setup.fields.play_status)).toBeInTheDocument();
    expect(within(missing).getByText(en.setup.fields.store)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('combobox', { name: en.game.playStatus }));
    fireEvent.click(await screen.findByRole('option', { name: en.playStatus.playing, hidden: true }));
    const copy = screen.getByTestId('setup-copy-21');
    expect(within(copy).queryByRole('combobox', { name: en.game.device })).not.toBeInTheDocument();
    fireEvent.click(within(copy).getByRole('combobox', { name: en.game.store }));
    fireEvent.click(await screen.findByRole('option', { name: 'Amazon', hidden: true }));
    fireEvent.click(screen.getByRole('button', { name: en.setup.saveNext }));

    expect(await screen.findByText('Tunic')).toBeInTheDocument();
    const game = calls.find((c) => c.method === 'PUT' && c.path === '/api/games/4')?.body as Record<string, unknown>;
    expect(game).toMatchObject({ title: 'Metroid Dread', play_status: 'playing' });
    const copyBody = calls.find((c) => c.path === '/api/copies/21')?.body;
    expect(copyBody).toMatchObject({ platform_id: 1, region: 'eu', format: 'physical', store_id: 3, condition: 'cib', notes: 'Spare' });

    fireEvent.click(screen.getByRole('button', { name: en.setup.skip }));
    expect(await screen.findByText(en.setup.done)).toBeInTheDocument();
  });

  it('only goes through games lacking the chosen details', async () => {
    mockApi(SETUP);
    renderAt('/setup');

    fireEvent.click(await screen.findByRole('checkbox', { name: /^Cover/ }));
    expect(screen.getByRole('button', { name: en.setup.start_one.replace('{{count}}', '1') })).toBeInTheDocument();
    expect(localStorage.getItem('memory-card-setup-fields')).not.toContain('"cover"');
  });
});
