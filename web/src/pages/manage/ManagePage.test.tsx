import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Device, Platform, Store } from '../../api/types';
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
    active: true,
    sort_order: id * 10,
    release_count: 0,
    device_count: 0,
    ...extra,
  };
}

const PLATFORMS = [
  platform(1, 'Wii U', { abbreviation: 'Wii U', device_count: 1 }),
  platform(2, 'Wii', { abbreviation: 'Wii', device_count: 1 }),
  platform(3, 'Nintendo 3DS', { abbreviation: '3DS' }),
];

const WII_U: Device = {
  id: 7,
  name: 'Living room Wii U',
  kind: 'console',
  model: 'Deluxe',
  notes: null,
  active: true,
  platform_ids: [1, 2],
  copy_count: 3,
};

describe('Manage devices', () => {
  it('lists devices with their kind and platforms', async () => {
    mockApi({ 'GET /api/devices': [WII_U], 'GET /api/platforms': PLATFORMS });
    renderAt('/manage');

    const row = await screen.findByTestId('device-Living room Wii U');
    expect(within(row).getByText(en.deviceKind.console)).toBeInTheDocument();
    expect(within(row).getByText('Deluxe')).toBeInTheDocument();
    expect(within(row).getByText('Wii U')).toBeInTheDocument();
    expect(within(row).getByText('Wii')).toBeInTheDocument();
  });

  it('creates a device', async () => {
    const calls = mockApi({
      'GET /api/devices': [],
      'GET /api/platforms': PLATFORMS,
      'POST /api/devices': (body: unknown) => json({ ...WII_U, ...(body as object), id: 8 }, 201),
    });
    renderAt('/manage');

    fireEvent.click(await screen.findByRole('button', { name: en.devices.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), {
      target: { value: 'Switch OLED' },
    });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({
      name: 'Switch OLED',
      kind: 'console',
      model: '',
      notes: '',
      active: true,
      platform_ids: [],
    });
  });

  it('shows a taken name on the name field', async () => {
    mockApi({
      'GET /api/devices': [WII_U],
      'GET /api/platforms': PLATFORMS,
      'POST /api/devices': json({ code: 'name_taken' }, 409),
    });
    renderAt('/manage');

    fireEvent.click(await screen.findByRole('button', { name: en.devices.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), {
      target: { value: 'living room wii u' },
    });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    expect(await screen.findByText(en.errors.name_taken)).toBeInTheDocument();
  });

  it('requires a name before calling the API', async () => {
    const calls = mockApi({ 'GET /api/devices': [], 'GET /api/platforms': PLATFORMS });
    renderAt('/manage');

    fireEvent.click(await screen.findByRole('button', { name: en.devices.add }));
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    expect(await screen.findByText(en.errors.required)).toBeInTheDocument();
    expect(calls.filter((c) => c.method === 'POST')).toEqual([]);
  });

  it('warns how many copies lose their device on delete', async () => {
    mockApi({ 'GET /api/devices': [WII_U], 'GET /api/platforms': PLATFORMS });
    renderAt('/manage');

    const row = await screen.findByTestId('device-Living room Wii U');
    fireEvent.click(within(row).getByRole('button', { name: en.common.delete }));

    expect(
      await screen.findByText('3 copies are on this device. They will be kept with no device.'),
    ).toBeInTheDocument();
  });
});

describe('Manage stores', () => {
  it('opens on the tab named in the URL', async () => {
    const stores: Store[] = [{ id: 1, name: 'Amazon', notes: 'Prime', copy_count: 0 }];
    mockApi({ 'GET /api/stores': stores });
    renderAt('/manage?tab=stores');

    const row = await screen.findByTestId('store-Amazon');
    expect(within(row).getByText('Prime')).toBeInTheDocument();
  });
});

describe('Manage platforms', () => {
  it('moves a platform down', async () => {
    const calls = mockApi({
      'GET /api/platforms': PLATFORMS,
      'PUT /api/platforms/order': (body: unknown) => {
        const ids = (body as { ids: number[] }).ids;
        return ids.map((id) => PLATFORMS.find((p) => p.id === id));
      },
    });
    renderAt('/manage?tab=platforms');

    const row = await screen.findByTestId('platform-wii-u');
    fireEvent.click(within(row).getByRole('button', { name: en.common.moveDown }));

    await waitFor(() =>
      expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({ ids: [2, 1, 3] }),
    );
  });

  it('only allows deleting unused platforms', async () => {
    mockApi({ 'GET /api/platforms': PLATFORMS });
    renderAt('/manage?tab=platforms');

    const used = await screen.findByTestId('platform-wii-u');
    expect(within(used).getByRole('button', { name: en.common.delete })).toBeDisabled();
    const unused = screen.getByTestId('platform-nintendo-3ds');
    expect(within(unused).getByRole('button', { name: en.common.delete })).toBeEnabled();
  });

  it('toggles a platform off', async () => {
    const calls = mockApi({
      'GET /api/platforms': PLATFORMS,
      'PUT /api/platforms/3': (body: unknown) => ({ ...PLATFORMS[2], ...(body as object) }),
    });
    renderAt('/manage?tab=platforms');

    fireEvent.click(await screen.findByRole('switch', { name: 'Active: Nintendo 3DS' }));

    await waitFor(() =>
      expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({
        name: 'Nintendo 3DS',
        abbreviation: '3DS',
        active: false,
      }),
    );
  });
});
