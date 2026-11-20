import {
  FORMATS,
  type Format,
  type LibraryQuery,
  PLAY_STATUSES,
  type PlayStatus,
  REGIONS,
  type Region,
} from '../../api/types';

/** Library filters as kept in the URL. Absent means "no filter". */
export type LibrarySearch = {
  q?: string;
  platform?: number[];
  device?: number[];
  store?: number[];
  genre?: number[];
  game_mode?: number[];
  region?: Region[];
  play_status?: PlayStatus[];
  format?: Format;
  min_local_players?: number;
  ownership?: 'any';
  sort?: 'added' | 'played';
};

const ID_LISTS = ['platform', 'device', 'store', 'genre', 'game_mode'] as const;

function ids(value: unknown): number[] | undefined {
  if (!Array.isArray(value)) return undefined;
  const list = value.map(Number).filter((n) => Number.isInteger(n) && n > 0);
  return list.length ? list : undefined;
}

function oneOf<T extends string>(allowed: readonly T[], value: unknown): T[] | undefined {
  if (!Array.isArray(value)) return undefined;
  const list = value.filter((v): v is T => allowed.includes(v as T));
  return list.length ? list : undefined;
}

/** Keeps only valid values, so a hand-edited URL never reaches the API malformed. */
export function validateLibrarySearch(search: Record<string, unknown>): LibrarySearch {
  const result: LibrarySearch = {};
  if (typeof search.q === 'string' && search.q.trim()) result.q = search.q;
  for (const key of ID_LISTS) {
    const list = ids(search[key]);
    if (list) result[key] = list;
  }
  const region = oneOf(REGIONS, search.region);
  if (region) result.region = region;
  const playStatus = oneOf(PLAY_STATUSES, search.play_status);
  if (playStatus) result.play_status = playStatus;
  if (FORMATS.includes(search.format as Format)) result.format = search.format as Format;
  const players = Number(search.min_local_players);
  if (Number.isInteger(players) && players > 1) result.min_local_players = players;
  if (search.ownership === 'any') result.ownership = 'any';
  if (search.sort === 'added' || search.sort === 'played') result.sort = search.sort;
  return result;
}

/** Number of active filters, not counting search text, ownership or sort. */
export function activeFilterCount(search: LibrarySearch): number {
  return (
    ID_LISTS.filter((key) => search[key]).length +
    (search.region ? 1 : 0) +
    (search.play_status ? 1 : 0) +
    (search.format ? 1 : 0) +
    (search.min_local_players ? 1 : 0)
  );
}

export function toApiQuery(search: LibrarySearch): LibraryQuery {
  const query: LibraryQuery = {};
  if (search.q) query.q = search.q;
  for (const key of ID_LISTS) {
    const list = search[key];
    if (list) query[key] = list.join(',');
  }
  if (search.region) query.region = search.region.join(',');
  if (search.play_status) query.play_status = search.play_status.join(',');
  if (search.format) query.format = search.format;
  if (search.min_local_players) query.min_local_players = search.min_local_players;
  if (search.ownership) query.ownership = search.ownership;
  if (search.sort) query.sort = search.sort;
  return query;
}
