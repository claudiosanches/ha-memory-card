import {
  FORMATS,
  type Format,
  type LibraryQuery,
  REGIONS,
  type Region,
} from '../../api/types';
import { listParam } from '../../searchParams';
import { isSlug } from '../../slug';

/** A play status key, or `none` for games without a status. */
export type StatusFilter = string;
export const NO_STATUS = 'none';

function statusKeys(value: unknown): string[] | undefined {
  const list = listParam(value).filter((v) => /^[a-z0-9_]+$/.test(v));
  return list.length ? list : undefined;
}

/** Library filters as kept in the URL. Absent means "no filter". */
export type LibrarySearch = {
  q?: string;
  /** Slugs, like the rest of these lists. */
  platform?: string[];
  device?: string[];
  store?: string[];
  genre?: string[];
  game_mode?: string[];
  region?: Region[];
  play_status?: StatusFilter[];
  format?: Format;
  min_local_players?: number;
  ownership?: 'any' | 'wanted';
  igdb?: 'linked' | 'unlinked';
  sort?: 'added' | 'played';
};

const SLUG_LISTS = ['platform', 'device', 'store', 'genre', 'game_mode'] as const;

function slugs(value: unknown): string[] | undefined {
  const list = listParam(value)
    .map((v) => v.toLowerCase())
    .filter(isSlug);
  return list.length ? list : undefined;
}

function oneOf<T extends string>(allowed: readonly T[], value: unknown): T[] | undefined {
  const list = listParam(value).filter((v): v is T => allowed.includes(v as T));
  return list.length ? list : undefined;
}

/** Keeps only valid values, so a hand-edited URL never reaches the API malformed. */
export function validateLibrarySearch(search: Record<string, unknown>): LibrarySearch {
  const result: LibrarySearch = {};
  if (typeof search.q === 'string' && search.q.trim()) result.q = search.q;
  for (const key of SLUG_LISTS) {
    const list = slugs(search[key]);
    if (list) result[key] = list;
  }
  const region = oneOf(REGIONS, search.region);
  if (region) result.region = region;
  const playStatus = statusKeys(search.play_status);
  if (playStatus) result.play_status = playStatus;
  if (FORMATS.includes(search.format as Format)) result.format = search.format as Format;
  const players = Number(search.min_local_players);
  if (Number.isInteger(players) && players > 1) result.min_local_players = players;
  if (search.ownership === 'any' || search.ownership === 'wanted') result.ownership = search.ownership;
  if (search.igdb === 'linked' || search.igdb === 'unlinked') result.igdb = search.igdb;
  if (search.sort === 'added' || search.sort === 'played') result.sort = search.sort;
  return result;
}

/** Number of active filters, not counting search text, ownership or sort. */
export function activeFilterCount(search: LibrarySearch): number {
  return (
    SLUG_LISTS.filter((key) => search[key]).length +
    (search.region ? 1 : 0) +
    (search.play_status ? 1 : 0) +
    (search.format ? 1 : 0) +
    (search.min_local_players ? 1 : 0) +
    (search.igdb ? 1 : 0)
  );
}

/**
 * The CSV export of what the library shows. The export defaults to every
 * copy, so ownership is always sent; sorting does not apply to it.
 */
export function exportHref(search: LibrarySearch): string {
  const params = new URLSearchParams({ ownership: 'owned' });
  for (const [key, value] of Object.entries(toApiQuery(search))) {
    if (key !== 'sort') params.set(key, String(value));
  }
  return `api/export.csv?${params}`;
}

export function toApiQuery(search: LibrarySearch): LibraryQuery {
  const query: LibraryQuery = {};
  if (search.q) query.q = search.q;
  for (const key of SLUG_LISTS) {
    const list = search[key];
    if (list) query[key] = list.join(',');
  }
  if (search.region) query.region = search.region.join(',');
  if (search.play_status) query.play_status = search.play_status.join(',');
  if (search.format) query.format = search.format;
  if (search.min_local_players) query.min_local_players = search.min_local_players;
  if (search.ownership) query.ownership = search.ownership;
  if (search.igdb) query.igdb = search.igdb;
  if (search.sort) query.sort = search.sort;
  return query;
}
