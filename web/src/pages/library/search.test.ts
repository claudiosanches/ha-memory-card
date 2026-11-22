import { describe, expect, it } from 'vitest';
import { activeFilterCount, exportHref, toApiQuery, validateLibrarySearch } from './search';

describe('library search', () => {
  it('keeps valid values and drops the rest', () => {
    expect(
      validateLibrarySearch({
        q: 'zelda',
        platform: ['switch', 'PS5', 'wii u', 3],
        region: ['eu', 'mars'],
        play_status: ['backlog'],
        format: 'cassette',
        min_local_players: 1,
        ownership: 'owned',
        sort: 'played',
      }),
    ).toEqual({ q: 'zelda', platform: ['switch', 'ps5'], region: ['eu'], play_status: ['backlog'], sort: 'played' });
    expect(validateLibrarySearch({ platform: 'itch.io', min_local_players: 4 })).toEqual({ min_local_players: 4 });
    expect(validateLibrarySearch({ play_status: ['none', 'Not a key!', 3, 'paused'] })).toEqual({
      play_status: ['none', 'paused'],
    });
    expect(toApiQuery({ play_status: ['none', 'paused'] })).toEqual({ play_status: 'none,paused' });
  });

  it('reads comma lists from the address', () => {
    expect(
      validateLibrarySearch({ platform: 'switch,ps5,', store: 'itch-io', region: 'eu,jp', play_status: 'none,paused', min_local_players: '2' }),
    ).toEqual({
      platform: ['switch', 'ps5'],
      store: ['itch-io'],
      region: ['eu', 'jp'],
      play_status: ['none', 'paused'],
      min_local_players: 2,
    });
  });

  it('builds the API query with comma lists', () => {
    expect(
      toApiQuery({ q: 'mario', platform: ['switch', 'ps5'], region: ['eu', 'jp'], format: 'digital', ownership: 'any' }),
    ).toEqual({ q: 'mario', platform: 'switch,ps5', region: 'eu,jp', format: 'digital', ownership: 'any' });
    expect(toApiQuery({})).toEqual({});
  });

  it('counts filters but not search, ownership or sort', () => {
    expect(activeFilterCount({ q: 'x', ownership: 'any', sort: 'added' })).toBe(0);
    expect(activeFilterCount({ platform: ['switch'], play_status: ['backlog'], min_local_players: 2 })).toBe(3);
    expect(activeFilterCount({ igdb: 'unlinked' })).toBe(1);
    expect(toApiQuery({ igdb: 'unlinked' })).toEqual({ igdb: 'unlinked' });
    expect(validateLibrarySearch({ igdb: 'maybe' })).toEqual({});
  });
});

describe('exportHref', () => {
  it('exports what the library shows, owned copies unless it shows all games', () => {
    expect(exportHref({})).toBe('api/export.csv?ownership=owned');
    expect(exportHref({ platform: ['switch', 'ps5'], format: 'digital', sort: 'added', q: 'mario kart' })).toBe(
      'api/export.csv?ownership=owned&q=mario+kart&platform=switch%2Cps5&format=digital',
    );
    expect(exportHref({ ownership: 'any' })).toBe('api/export.csv?ownership=any');
  });
});
