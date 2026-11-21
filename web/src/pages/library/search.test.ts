import { describe, expect, it } from 'vitest';
import { activeFilterCount, toApiQuery, validateLibrarySearch } from './search';

describe('library search', () => {
  it('keeps valid values and drops the rest', () => {
    expect(
      validateLibrarySearch({
        q: 'zelda',
        platform: [1, '2', 'x', -3],
        region: ['eu', 'mars'],
        play_status: ['backlog'],
        format: 'cassette',
        min_local_players: 1,
        ownership: 'owned',
        sort: 'played',
      }),
    ).toEqual({ q: 'zelda', platform: [1, 2], region: ['eu'], play_status: ['backlog'], sort: 'played' });
    expect(validateLibrarySearch({ platform: 'not-a-list', min_local_players: 4 })).toEqual({ min_local_players: 4 });
    expect(validateLibrarySearch({ play_status: ['none', 'unknown', 'paused'] })).toEqual({
      play_status: ['none', 'paused'],
    });
    expect(toApiQuery({ play_status: ['none', 'paused'] })).toEqual({ play_status: 'none,paused' });
  });

  it('builds the API query with comma lists', () => {
    expect(
      toApiQuery({ q: 'mario', platform: [1, 2], region: ['eu', 'jp'], format: 'digital', ownership: 'any' }),
    ).toEqual({ q: 'mario', platform: '1,2', region: 'eu,jp', format: 'digital', ownership: 'any' });
    expect(toApiQuery({})).toEqual({});
  });

  it('counts filters but not search, ownership or sort', () => {
    expect(activeFilterCount({ q: 'x', ownership: 'any', sort: 'added' })).toBe(0);
    expect(activeFilterCount({ platform: [1], play_status: ['backlog'], min_local_players: 2 })).toBe(3);
    expect(activeFilterCount({ igdb: 'unlinked' })).toBe(1);
    expect(toApiQuery({ igdb: 'unlinked' })).toEqual({ igdb: 'unlinked' });
    expect(validateLibrarySearch({ igdb: 'maybe' })).toEqual({});
  });
});
