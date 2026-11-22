import { describe, expect, it } from 'vitest';
import { isSlug, sanitizeSlug, slugify } from './slug';

describe('slugs', () => {
  it('makes slugs the way the server does', () => {
    expect(slugify('Sega Mega Drive/Genesis')).toBe('sega-mega-drive-genesis');
    expect(slugify('  itch.io ')).toBe('itch-io');
    expect(slugify('Pokémon Mini')).toBe('pokemon-mini');
    expect(slugify('ポケモン')).toBe('');
    expect(slugify('a'.repeat(70))).toHaveLength(64);
  });

  it('keeps typing valid', () => {
    expect(sanitizeSlug('Nintendo ')).toBe('nintendo-');
    expect(sanitizeSlug(' -Wii U!!')).toBe('wii-u-');
    expect(sanitizeSlug('Café')).toBe('cafe');
  });

  it('validates', () => {
    expect(['switch', 'edsac--1', '3ds'].every(isSlug)).toBe(true);
    expect(['', 'Switch', '-a', 'a-', 'wii u', 'itch.io', 'a'.repeat(65)].some(isSlug)).toBe(false);
  });
});
