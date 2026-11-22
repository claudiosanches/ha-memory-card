import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { GameDetail, Platform } from '../api/types';
import en from '../locales/en.json';
import { mockApi } from '../test/api';
import { renderAt } from '../test/render';

vi.mock('./barcode', async (original) => ({
  ...(await original<typeof import('./barcode')>()),
  decodeBarcode: vi.fn(async () => '4902370536485'),
  decodeVideoFrame: vi.fn(async () => '4902370536485'),
}));

const SWITCH: Platform = {
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
  game_count: 0,
  owned_game_count: 0,
  wanted_game_count: 0,
  device_count: 0,
};

const GAME = { id: 9, title: 'Super Mario Odyssey' } as GameDetail;

function scanPhoto() {
  fireEvent.change(screen.getByTestId('scan-input'), {
    target: { files: [new File(['jpeg'], 'box.jpg', { type: 'image/jpeg' })] },
  });
}

describe('Scanning a barcode', () => {
  it('names the product and adds the copy with the code', async () => {
    const calls = mockApi({
      'GET /api/games': [],
      'GET /api/platforms': [SWITCH],
      'GET /api/devices': [],
      'GET /api/stores': [],
      'GET /api/barcodes/4902370536485': {
        barcode: '4902370536485',
        copies: [],
        product: { title: 'Super Mario Odyssey - Nintendo Switch', brand: 'Nintendo' },
        lookup: 'found',
      },
      'POST /api/games': GAME,
      'POST /api/games/9/copies': GAME,
    });
    renderAt('/');

    await screen.findByRole('button', { name: en.scan.button });
    scanPhoto();
    const result = await screen.findByTestId('scan-result');
    expect(result).toHaveTextContent('Super Mario Odyssey - Nintendo Switch (Nintendo)');
    fireEvent.click(within(result).getByRole('button', { name: en.scan.addIt }));

    const pick = await screen.findByRole('dialog', { name: en.scan.whichGame });
    expect(within(pick).getByLabelText(new RegExp(en.game.title))).toHaveValue('Super Mario Odyssey');
    fireEvent.click(within(pick).getByRole('button', { name: en.wishlist.next }));

    const form = await screen.findByRole('dialog', { name: 'Add Super Mario Odyssey' });
    expect(within(form).getByLabelText(en.game.barcode)).toHaveValue('4902370536485');
    expect(within(form).getByRole('combobox', { name: en.game.platform })).toHaveValue('Nintendo Switch');
    expect(within(form).getByRole('combobox', { name: en.game.region })).toHaveValue(en.region.jp);
    fireEvent.click(within(form).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/9/copies')).toBe(true));
    expect(calls.find((c) => c.path === '/api/games/9/copies')?.body).toMatchObject({
      barcode: '4902370536485',
      platform_id: 1,
      region: 'jp',
      status: 'owned',
    });
  });

  it('says when the box is already in the collection', async () => {
    mockApi({
      'GET /api/games': [],
      'GET /api/platforms': [SWITCH],
      'GET /api/barcodes/4902370536485': {
        barcode: '4902370536485',
        copies: [{ copy_id: 3, game_id: 9, title: 'Super Mario Odyssey', platform_id: 1, region: 'jp', status: 'owned' }],
        product: null,
        lookup: 'in_collection',
      },
    });
    renderAt('/');

    await screen.findByRole('button', { name: en.scan.button });
    scanPhoto();
    const owned = await screen.findByTestId('scan-owned');
    expect(within(owned).getByRole('link', { name: 'Super Mario Odyssey' })).toBeInTheDocument();
    expect(owned).toHaveTextContent(`Nintendo Switch · ${en.region.jp}`);
  });
});

describe('Scanning with the camera', () => {
  const OWNED = {
    barcode: '4902370536485',
    copies: [{ copy_id: 3, game_id: 9, title: 'Super Mario Odyssey', platform_id: 1, region: 'jp', status: 'owned' }],
    product: null,
    lookup: 'in_collection',
  };

  function withCamera(getUserMedia: () => Promise<MediaStream>) {
    Object.defineProperty(window, 'isSecureContext', { value: true, configurable: true });
    Object.defineProperty(navigator, 'mediaDevices', { value: { getUserMedia: vi.fn(getUserMedia) }, configurable: true });
    vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue();
  }

  afterEach(() => {
    Reflect.deleteProperty(window, 'isSecureContext');
    Reflect.deleteProperty(navigator, 'mediaDevices');
    vi.restoreAllMocks();
  });

  it('reads the code from the camera and lets the camera go', async () => {
    const stop = vi.fn();
    withCamera(async () => ({ getTracks: () => [{ stop }] }) as unknown as MediaStream);
    mockApi({ 'GET /api/games': [], 'GET /api/platforms': [SWITCH], 'GET /api/barcodes/4902370536485': OWNED });
    renderAt('/');

    fireEvent.click(await screen.findByRole('button', { name: en.scan.button }));
    expect(await screen.findByTestId('scan-owned')).toBeInTheDocument();
    expect(stop).toHaveBeenCalled();
  });

  it('offers a photo or the digits when the camera is refused', async () => {
    withCamera(async () => {
      throw new DOMException('denied', 'NotAllowedError');
    });
    mockApi({ 'GET /api/games': [], 'GET /api/platforms': [SWITCH] });
    renderAt('/');

    fireEvent.click(await screen.findByRole('button', { name: en.scan.button }));
    expect(await screen.findByText(en.scan.noCamera)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: en.scan.photo })).toBeInTheDocument();
    expect(screen.getByLabelText(en.scan.code)).toBeInTheDocument();
  });
});
