import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { DuplicateGroup } from '../api/types';
import en from '../locales/en.json';
import { mockApi } from '../test/api';
import { renderAt } from '../test/render';

const copy = { release_title: null, device_id: null, store_id: null, condition: null, notes: null };

const GROUPS: DuplicateGroup[] = [
  {
    game_id: 1,
    title: 'Hades',
    cover_path: null,
    labels: ['cross_platform', 'physical_digital'],
    copies: [
      { ...copy, id: 1, release_id: 1, platform_id: 1, region: 'eu', format: 'digital', store_id: 5 },
      { ...copy, id: 2, release_id: 2, platform_id: 2, region: 'eu', format: 'physical', condition: 'cib' },
    ],
  },
  {
    game_id: 2,
    title: 'Tetris',
    cover_path: null,
    labels: ['same_release'],
    copies: [
      { ...copy, id: 3, release_id: 3, platform_id: 1, region: 'jp', format: 'physical' },
      { ...copy, id: 4, release_id: 3, platform_id: 1, region: 'jp', format: 'physical' },
    ],
  },
];

describe('Duplicates', () => {
  it('lists groups with their copies and filters by label', async () => {
    mockApi({
      'GET /api/duplicates': GROUPS,
      'GET /api/platforms': [
        { id: 1, name: 'Nintendo Switch', slug: 'switch', active: true, sort_order: 1, game_count: 2, device_count: 0 },
        { id: 2, name: 'PlayStation 4', slug: 'ps4', active: true, sort_order: 2, game_count: 1, device_count: 0 },
      ],
      'GET /api/devices': [],
      'GET /api/stores': [{ id: 5, name: 'Nintendo eShop', notes: null, copy_count: 1 }],
    });
    renderAt('/duplicates');

    const hades = await screen.findByTestId('duplicate-Hades');
    expect(within(hades).getByText(en.duplicates.label.physical_digital)).toBeInTheDocument();
    expect(await within(hades).findByText('PlayStation 4 · Europe (PAL)')).toBeInTheDocument();
    expect(await within(hades).findByText('Digital · Nintendo eShop')).toBeInTheDocument();
    expect(screen.getByTestId('duplicates-count')).toHaveTextContent('2 games');

    fireEvent.click(screen.getByRole('button', { name: new RegExp(en.duplicates.label.same_release) }));
    expect(screen.queryByTestId('duplicate-Hades')).not.toBeInTheDocument();
    expect(screen.getByTestId('duplicate-Tetris')).toBeInTheDocument();
    expect(screen.getByTestId('duplicates-count')).toHaveTextContent('1 game');
  });

  it('says when there are no duplicates', async () => {
    mockApi({ 'GET /api/duplicates': [], 'GET /api/platforms': [], 'GET /api/devices': [], 'GET /api/stores': [] });
    renderAt('/duplicates');
    expect(await screen.findByText(en.duplicates.empty)).toBeInTheDocument();
  });
});
