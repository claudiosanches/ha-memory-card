import { describe, expect, it } from 'vitest';
import { listParam, parseSearch, stringifySearch } from './searchParams';

describe('search params', () => {
  it('joins lists with plain commas', () => {
    expect(stringifySearch({ device: ['switch'], platform: ['wii-u', 'ps5'], min_local_players: 2 })).toBe(
      '?device=switch&platform=wii-u,ps5&min_local_players=2',
    );
    expect(stringifySearch({ q: 'mario, kart & co', platform: [], sort: undefined })).toBe('?q=mario%2C%20kart%20%26%20co');
    expect(stringifySearch({})).toBe('');
  });

  it('reads back what it writes', () => {
    const search = parseSearch(stringifySearch({ q: 'mario, kart', platform: ['wii-u', 'ps5'], tab: 'stores' }));
    expect(search).toEqual({ q: 'mario, kart', platform: 'wii-u,ps5', tab: 'stores' });
    expect(listParam(search.platform)).toEqual(['wii-u', 'ps5']);
    expect(parseSearch('?q=a+b&bad=%E0%A4%A&flag')).toEqual({ q: 'a b', bad: '%E0%A4%A', flag: '' });
  });
});
