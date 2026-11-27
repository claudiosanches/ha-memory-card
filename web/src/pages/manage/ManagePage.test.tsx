import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Label, Platform, Store } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi } from '../../test/api';
import { renderAt } from '../../test/render';
import { reorderedIds } from './PlatformsTab';

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

describe('Manage accounts', () => {
  it('lists accounts with their username, store and person, and adds one', async () => {
    const steam = {
      id: 3,
      name: 'Steam',
      slug: 'steam',
      username: 'ana_steam',
      store_id: 5,
      person: { id: 'ana', name: 'Ana' },
      notes: null,
      active: true,
      platform_ids: [],
      copy_count: 2,
    };
    const calls = mockApi({
      'GET /api/accounts': [steam],
      'GET /api/platforms': PLATFORMS,
      'GET /api/stores': [{ id: 5, name: 'Steam store', slug: 'steam-store', notes: null, copy_count: 0 }],
      'GET /api/people': [{ id: 'ana', name: 'Ana' }],
      'POST /api/accounts': (body: unknown) => json({ ...steam, ...(body as object), id: 9 }, 201),
    });
    renderAt('/manage');

    const row = await screen.findByTestId('account-Steam');
    expect(row).toHaveTextContent('ana_steam');
    expect(row).toHaveTextContent('2 copies');
    await waitFor(() => expect(row).toHaveTextContent(`Steam store${en.common.separator}Ana`));

    fireEvent.click(screen.getByRole('button', { name: en.accounts.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'GOG' } });
    fireEvent.change(screen.getByRole('textbox', { name: en.accounts.username }), { target: { value: 'ana_gog' } });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({
      name: 'GOG',
      username: 'ana_gog',
      store_id: null,
      user_id: null,
    });
  });
});

describe('Manage stores', () => {
  it('opens on the tab named in the URL', async () => {
    const stores: Store[] = [{ id: 1, name: 'Amazon', slug: 'amazon', notes: 'Prime', copy_count: 0 }];
    mockApi({ 'GET /api/stores': stores });
    renderAt('/manage?tab=stores');

    const row = await screen.findByTestId('store-Amazon');
    expect(within(row).getByText('Prime')).toBeInTheDocument();
  });

  it('gives a new store a slug from its name until the slug is edited', async () => {
    const calls = mockApi({
      'GET /api/stores': [],
      'POST /api/stores': (body: unknown) => json({ id: 2, copy_count: 0, ...(body as object) }, 201),
    });
    renderAt('/manage?tab=stores');

    fireEvent.click(await screen.findByRole('button', { name: en.stores.add }));
    const name = screen.getByRole('textbox', { name: en.common.name });
    const slug = screen.getByRole('textbox', { name: en.slug.label });
    fireEvent.change(name, { target: { value: 'Café Jogos' } });
    expect(slug).toHaveValue('cafe-jogos');

    fireEvent.change(slug, { target: { value: 'My Shop!' } });
    expect(slug).toHaveValue('my-shop-');
    fireEvent.blur(slug);
    expect(slug).toHaveValue('my-shop');
    fireEvent.change(name, { target: { value: 'Café Jogos Centro' } });
    expect(slug).toHaveValue('my-shop');
    expect(screen.queryByTestId('slug-changed')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ name: 'Café Jogos Centro', slug: 'my-shop', notes: '' });
  });

  it('warns that changing a saved slug breaks its links', async () => {
    mockApi({
      'GET /api/stores': [{ id: 1, name: 'Amazon', slug: 'amazon', notes: null, copy_count: 0 }],
      'PUT /api/stores/1': () => json({ code: 'slug_taken' }, 409),
    });
    renderAt('/manage?tab=stores');

    fireEvent.click(within(await screen.findByTestId('store-Amazon')).getByRole('button', { name: en.common.edit }));
    const slug = screen.getByRole('textbox', { name: en.slug.label });
    expect(slug).toHaveValue('amazon');
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'Amazon US' } });
    expect(slug).toHaveValue('amazon');
    expect(screen.queryByTestId('slug-changed')).not.toBeInTheDocument();

    fireEvent.change(slug, { target: { value: 'steam' } });
    expect(screen.getByTestId('slug-changed')).toHaveTextContent(en.slug.changed);
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));
    expect(await screen.findByText(en.errors.slug_taken)).toBeInTheDocument();
  });

  it('hides the slug field when adding and editing under ingress', async () => {
    const base = document.createElement('base');
    base.href = '/api/hassio_ingress/test/';
    document.head.appendChild(base);
    try {
      const calls = mockApi({
        'GET /api/stores': [{ id: 1, name: 'Amazon', slug: 'amazon', notes: null, copy_count: 0 }],
        'POST /api/stores': (body: unknown) => json({ id: 2, slug: 'cafe-jogos', copy_count: 0, ...(body as object) }, 201),
        'PUT /api/stores/1': (body: unknown) => json({ id: 1, copy_count: 0, ...(body as object) }),
      });
      renderAt('/manage?tab=stores');

      fireEvent.click(await screen.findByRole('button', { name: en.stores.add }));
      expect(screen.queryByRole('textbox', { name: en.slug.label })).toBeNull();
      expect(screen.queryByText(en.slug.label)).toBeNull();
      fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'Café Jogos' } });
      fireEvent.click(screen.getByRole('button', { name: en.common.save }));
      await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
      expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ name: 'Café Jogos', slug: null, notes: '' });

      fireEvent.click(within(await screen.findByTestId('store-Amazon')).getByRole('button', { name: en.common.edit }));
      expect(screen.queryByRole('textbox', { name: en.slug.label })).toBeNull();
      expect(screen.queryByText(en.slug.label)).toBeNull();
      fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'Amazon US' } });
      fireEvent.click(screen.getByRole('button', { name: en.common.save }));
      await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
      expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({ name: 'Amazon US', slug: 'amazon', notes: '' });
    } finally {
      base.remove();
    }
  });
});

describe('Manage platforms', () => {
  const CATALOG = [
    ...PLATFORMS,
    platform(4, 'Dreamcast', { active: false, category: 'console', igdb_platform_id: 23, sort_order: 1010 }),
    platform(5, 'Game Gear', { active: false, category: 'handheld', igdb_platform_id: 35, sort_order: 1020 }),
  ];

  it('reorders your platforms and keeps the catalog after them', () => {
    const active = CATALOG.filter((p) => p.active);
    const inactive = CATALOG.filter((p) => !p.active);
    expect(reorderedIds(active, inactive, 0, 2)).toEqual([2, 3, 1, 4, 5]);
    expect(reorderedIds(active, inactive, 2, 0)).toEqual([3, 1, 2, 4, 5]);
  });

  it('gives each of your platforms a drag handle', async () => {
    mockApi({ 'GET /api/platforms': CATALOG });
    renderAt('/manage?tab=platforms');
    const row = await screen.findByTestId('platform-wii-u');
    expect(within(row).getByRole('button', { name: 'Move Wii U' })).toBeInTheDocument();
    expect(within(screen.getByTestId('platform-dreamcast')).queryByRole('button', { name: 'Move Dreamcast' })).toBeNull();
  });

  it('only offers deleting unused platforms added by hand', async () => {
    mockApi({ 'GET /api/platforms': CATALOG });
    renderAt('/manage?tab=platforms');

    const used = await screen.findByTestId('platform-wii-u');
    expect(within(used).queryByRole('button', { name: en.common.delete })).toBeNull();
    const unused = screen.getByTestId('platform-nintendo-3ds');
    expect(within(unused).getByRole('button', { name: en.common.delete })).toBeEnabled();
    const fromIgdb = screen.getByTestId('platform-dreamcast');
    expect(within(fromIgdb).queryByRole('button', { name: en.common.delete })).toBeNull();
  });

  it('removes a platform from your list', async () => {
    const calls = mockApi({
      'GET /api/platforms': CATALOG,
      'PUT /api/platforms/3': (body: unknown) => ({ ...CATALOG[2], ...(body as object) }),
    });
    renderAt('/manage?tab=platforms');

    fireEvent.click(await screen.findByRole('button', { name: 'Remove Nintendo 3DS from your platforms' }));

    await waitFor(() =>
      expect(calls.find((c) => c.method === 'PUT')?.body).toEqual({
        name: 'Nintendo 3DS',
        abbreviation: '3DS',
        active: false,
      }),
    );
  });

  it('searches the catalog and adds a platform', async () => {
    const calls = mockApi({
      'GET /api/platforms': CATALOG,
      'PUT /api/platforms/4': (body: unknown) => ({ ...CATALOG[3], ...(body as object) }),
    });
    renderAt('/manage?tab=platforms');

    fireEvent.change(await screen.findByRole('textbox', { name: en.platforms.search }), {
      target: { value: 'dream' },
    });
    expect(screen.queryByTestId('platform-game-gear')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Add Dreamcast' }));

    await waitFor(() =>
      expect(calls.find((c) => c.method === 'PUT')?.body).toMatchObject({ name: 'Dreamcast', active: true }),
    );
  });
});

describe('Manage labels', () => {
  const RETRO: Label = { id: 3, name: 'Retro', slug: 'retro', color: 'orange', game_count: 2 };

  it('lists labels with how many games have them', async () => {
    mockApi({ 'GET /api/labels': [RETRO] });
    renderAt('/manage?tab=labels');

    const row = await screen.findByTestId('label-Retro');
    expect(within(row).getByText('2 games')).toBeInTheDocument();
  });

  it('creates a label with a color', async () => {
    const calls = mockApi({
      'GET /api/labels': [],
      'POST /api/labels': (body: unknown) => json({ ...RETRO, ...(body as object), id: 4, game_count: 0 }, 201),
    });
    renderAt('/manage?tab=labels');

    fireEvent.click(await screen.findByRole('button', { name: en.labels.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'Couch co-op' } });
    fireEvent.click(screen.getByRole('radio', { name: en.playStatuses.colors.teal }));
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ name: 'Couch co-op', slug: 'couch-co-op', color: 'teal' });
  });

  it('refuses commas and semicolons in a name before calling the API', async () => {
    const calls = mockApi({ 'GET /api/labels': [] });
    renderAt('/manage?tab=labels');

    fireEvent.click(await screen.findByRole('button', { name: en.labels.add }));
    fireEvent.change(screen.getByRole('textbox', { name: en.common.name }), { target: { value: 'Mario, Luigi' } });
    fireEvent.click(screen.getByRole('button', { name: en.common.save }));

    expect(await screen.findByText(en.errors.label_separator)).toBeInTheDocument();
    expect(calls.some((c) => c.method === 'POST')).toBe(false);
  });

  it('warns how many games lose the label on delete', async () => {
    mockApi({ 'GET /api/labels': [RETRO] });
    renderAt('/manage?tab=labels');

    const row = await screen.findByTestId('label-Retro');
    fireEvent.click(within(row).getByRole('button', { name: en.common.delete }));
    expect(await screen.findByText('2 games have this label. They will be kept without it.')).toBeInTheDocument();
  });
});
