import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { PlayStatus } from '../../api/types';
import en from '../../locales/en.json';
import { BUILTIN_STATUSES, json, mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

const REPLAYING: PlayStatus = {
  key: 'replaying',
  label: 'Replaying',
  color: 'teal',
  counts_as: 'playing',
  builtin: false,
  game_count: 3,
};
const STATUSES = [...BUILTIN_STATUSES, REPLAYING];

describe('Play statuses tab', () => {
  it('lists statuses with their group and only lets custom ones be deleted', async () => {
    mockApi({ 'GET /api/play-statuses': STATUSES });
    renderAt('/manage?tab=playStatuses');

    const custom = await screen.findByTestId('status-replaying');
    expect(custom).toHaveTextContent('Replaying');
    expect(custom).toHaveTextContent('Counts as Playing · 3 games');
    expect(within(custom).getByRole('button', { name: en.common.delete })).toBeInTheDocument();
    const builtin = screen.getByTestId('status-completed');
    expect(builtin).toHaveTextContent('Counts as Finished · 0 games · Built-in');
    expect(within(builtin).queryByRole('button', { name: en.common.delete })).toBeNull();
  });

  it('adds a status with a color and group', async () => {
    const calls = mockApi({
      'GET /api/play-statuses': STATUSES,
      'POST /api/play-statuses': (body: unknown) => json({ ...REPLAYING, ...(body as object), key: 'someday' }, 201),
    });
    renderAt('/manage?tab=playStatuses');

    fireEvent.click(await screen.findByRole('button', { name: en.playStatuses.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.playStatuses.name }), { target: { value: ' Someday ' } });
    fireEvent.click(screen.getByRole('radio', { name: en.playStatuses.colors.violet }));
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() =>
      expect(calls.find((c) => c.method === 'POST')?.body).toEqual({
        label: 'Someday',
        color: 'violet',
        counts_as: 'backlog',
      }),
    );
  });

  it('renames a built-in status and keeps its group fixed', async () => {
    const calls = mockApi({
      'GET /api/play-statuses': STATUSES,
      'PUT /api/play-statuses/paused': (body: unknown) => ({ ...BUILTIN_STATUSES[2], ...(body as object) }),
    });
    renderAt('/manage?tab=playStatuses');

    const row = await screen.findByTestId('status-paused');
    fireEvent.click(within(row).getByRole('button', { name: en.common.edit }));
    expect(screen.getByRole('textbox', { name: en.playStatuses.name })).toHaveAttribute('placeholder', 'Paused');
    expect(screen.getByRole('combobox', { name: en.playStatuses.countsAs })).toBeDisabled();
    fireEvent.change(screen.getByRole('textbox', { name: en.playStatuses.name }), { target: { value: 'On hold' } });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() =>
      expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({
        label: 'On hold',
        color: 'yellow',
        counts_as: 'paused',
      }),
    );
  });

  it('moves the games of a deleted status', async () => {
    const calls = mockApi({
      'GET /api/play-statuses': STATUSES,
      'DELETE /api/play-statuses/replaying': json(null, 204),
    });
    renderAt('/manage?tab=playStatuses');

    const row = await screen.findByTestId('status-replaying');
    fireEvent.click(within(row).getByRole('button', { name: en.common.delete }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(within(dialog).getByRole('combobox', { name: '3 games have this status. Move them to:' }));
    fireEvent.click(await screen.findByRole('option', { name: 'Playing', hidden: true }));
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.delete }));

    await waitFor(() => expect(calls.find((c) => c.method === 'DELETE')?.query.get('move_to')).toBe('playing'));
  });
});
