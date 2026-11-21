import { describe, expect, it } from 'vitest';

const PLATFORMS = [
  { name: 'Nintendo Switch', abbreviation: 'Switch' },
  { name: 'Nintendo Switch 2', abbreviation: 'Switch 2' },
  { name: 'PlayStation 4', abbreviation: 'PS4' },
  { name: 'Wii', abbreviation: 'Wii' },
];

describe('barcode helpers', async () => {
  // Imported lazily, like the app does, so the WASM loader is not set up for other tests.
  const { gameTitle, platformInTitle, regionFromBarcode } = await import('./barcode');

  it('guesses the region from the prefix', () => {
    expect(regionFromBarcode('045496590024')).toBe('na');
    expect(regionFromBarcode('4902370536485')).toBe('jp');
    expect(regionFromBarcode('0045496420253')).toBe('na');
    expect(regionFromBarcode('5030917299148')).toBe('eu');
    expect(regionFromBarcode('7891234567895')).toBe('br');
    expect(regionFromBarcode('6001234567890')).toBeNull();
  });

  it('finds the platform and cleans the title', () => {
    const title = 'Super Mario Odyssey - Nintendo Switch (Standard Edition)';
    const found = platformInTitle(title, PLATFORMS);
    expect(found?.name).toBe('Nintendo Switch');
    expect(gameTitle(title, found)).toBe('Super Mario Odyssey');
    expect(platformInTitle('Mario Kart World Nintendo Switch 2', PLATFORMS)?.name).toBe('Nintendo Switch 2');
    expect(platformInTitle('Wii Sports Resort', PLATFORMS)?.name).toBe('Wii');
    expect(gameTitle('Bloodborne Game of the Year Edition - PS4', PLATFORMS[2]!)).toBe('Bloodborne Game of the Year Edition');
    expect(gameTitle('Wii Sports Resort (Wii)', PLATFORMS[3]!)).toBe('Wii Sports Resort');
    expect(gameTitle('Wii Sports Resort Wii', PLATFORMS[3]!)).toBe('Wii Sports Resort');
  });
});
