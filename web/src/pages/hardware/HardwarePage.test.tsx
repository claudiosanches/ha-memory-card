import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Device, Platform } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi } from '../../test/api';
import { renderAt } from '../../test/render';

function platform(id: number, name: string, extra: Partial<Platform> = {}): Platform {
  return {
    id,
    name,
    slug: name.toLowerCase().replace(/\W+/g, '-'),
    abbreviation: null,
    family: null,
    generation: null,
    igdb_platform_id: null,
    custom: false,
    active: true,
    sort_order: id * 10,
    game_count: 0,
    owned_game_count: 0,
    wanted_game_count: 0,
    device_count: 0,
    account_count: 0,
    ...extra,
  };
}

const PLATFORMS = [
  platform(1, 'Wii U', { abbreviation: 'Wii U', device_count: 1 }),
  platform(2, 'Wii', { abbreviation: 'Wii', device_count: 1 }),
  platform(3, 'Nintendo 3DS', { abbreviation: '3DS', custom: true }),
];

const WII_U: Device = {
  id: 7,
  name: 'Living room Wii U',
  slug: 'living-room-wii-u',
  kind: 'console',
  model: 'Deluxe',
  notes: null,
  active: true,
  platform_ids: [1, 2],
  copy_count: 3,
  status: 'owned',
  photos: [],
};

describe('Hardware', () => {
  it('lists devices with their kind and platforms', async () => {
    mockApi({ 'GET /api/devices': [WII_U], 'GET /api/platforms': PLATFORMS });
    renderAt('/hardware');

    const row = await screen.findByTestId('device-Living room Wii U');
    expect(row).toHaveTextContent(`${en.deviceKind.console}${en.common.separator}Deluxe`);
    expect(within(row).getByText('Wii U')).toBeInTheDocument();
    expect(within(row).getByText('Wii')).toBeInTheDocument();
  });

  it('creates a device', async () => {
    const calls = mockApi({
      'GET /api/devices': [],
      'GET /api/platforms': PLATFORMS,
      'POST /api/devices': (body: unknown) => json({ ...WII_U, ...(body as object), id: 8 }, 201),
    });
    renderAt('/hardware');

    fireEvent.click(await screen.findByRole('button', { name: en.hardware.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), {
      target: { value: 'Switch OLED' },
    });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({
      name: 'Switch OLED',
      slug: null,
      kind: 'console',
      status: 'owned',
      model: null,
      active: true,
      platform_ids: [],
      platform_id: null,
    });
  });

  it('shows a taken name on the name field', async () => {
    mockApi({
      'GET /api/devices': [WII_U],
      'GET /api/platforms': PLATFORMS,
      'POST /api/devices': json({ code: 'name_taken' }, 409),
    });
    renderAt('/hardware');

    fireEvent.click(await screen.findByRole('button', { name: en.hardware.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), {
      target: { value: 'living room wii u' },
    });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    expect(await screen.findByText(en.errors.name_taken)).toBeInTheDocument();
  });

  it('requires a name before calling the API', async () => {
    const calls = mockApi({ 'GET /api/devices': [], 'GET /api/platforms': PLATFORMS });
    renderAt('/hardware');

    fireEvent.click(await screen.findByRole('button', { name: en.hardware.add }));
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    expect(await screen.findByText(en.errors.required)).toBeInTheDocument();
    expect(calls.filter((c) => c.method === 'POST')).toEqual([]);
  });

  it('warns how many copies lose their device on delete', async () => {
    mockApi({ 'GET /api/devices': [WII_U], 'GET /api/platforms': PLATFORMS });
    renderAt('/hardware');

    const row = await screen.findByTestId('device-Living room Wii U');
    fireEvent.click(within(row).getByRole('button', { name: en.common.delete }));

    expect(
      await screen.findByText('3 copies are on this device. They will be kept with no device.'),
    ).toBeInTheDocument();
  });

  it('asks what a controller is for instead of what it plays', async () => {
    const calls = mockApi({
      'GET /api/devices': [],
      'GET /api/platforms': PLATFORMS,
      'POST /api/devices': (body: unknown) => json({ ...WII_U, ...(body as object), id: 9 }, 201),
    });
    renderAt('/hardware');

    fireEvent.click(await screen.findByRole('button', { name: en.hardware.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'Pro Controller' } });
    fireEvent.click(within(screen.getByRole('dialog')).getByRole('combobox', { name: en.devices.kind }));
    // The page's kind filter lists the same options; the form's opened last.
    const options = await screen.findAllByRole('option', { name: en.deviceKind.controller, hidden: true });
    fireEvent.click(options[options.length - 1] as HTMLElement);
    expect(screen.queryByText(en.devices.platformsDescription)).not.toBeInTheDocument();
    expect(screen.getByText(en.hardware.forDescription)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({ kind: 'controller', platform_ids: [], active: true });
  });
});
