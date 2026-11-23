import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { api } from './client';
import { ApiError, call } from './errors';
import type { LibraryQuery } from './types';

export const queryKeys = {
  me: ['me'],
  status: ['status'],
  platforms: ['platforms'],
  devices: ['devices'],
  stores: ['stores'],
  labels: ['labels'],
  playStatuses: ['play-statuses'],
  games: ['games'],
  game: (id: number) => ['game', id] as const,
  gameModes: ['game-modes'],
  genres: ['genres'],
  igdbSync: ['igdb-sync'],
  igdbTitles: ['igdb-titles'],
  duplicates: ['duplicates'],
  missing: ['missing-details'],
  stats: ['stats'],
} as const;

/** Who is signed in and their role. Options only change on restart. */
export function useMe() {
  return useQuery({
    queryKey: queryKeys.me,
    queryFn: () => call(api.GET('/api/me')),
    staleTime: Infinity,
    // A missing identity does not fix itself; say so at once.
    retry: (failures, error) => !(error instanceof ApiError && error.code === 'identity_required') && failures < 3,
  });
}

/** Whether the user may change the collection. The layout loads this before any page. */
export function useCanManage() {
  return useMe().data?.role === 'manager';
}

/** Whether the user may change play status and last played. */
export function useCanTrackProgress() {
  return useMe().data?.can_track_progress ?? false;
}

export function useStatus() {
  return useQuery({ queryKey: queryKeys.status, queryFn: () => call(api.GET('/api/status')) });
}

export function usePlatforms() {
  return useQuery({ queryKey: queryKeys.platforms, queryFn: () => call(api.GET('/api/platforms')) });
}

export function useDevices() {
  return useQuery({ queryKey: queryKeys.devices, queryFn: () => call(api.GET('/api/devices')) });
}

export function useStores() {
  return useQuery({ queryKey: queryKeys.stores, queryFn: () => call(api.GET('/api/stores')) });
}

export function useLabels() {
  return useQuery({ queryKey: queryKeys.labels, queryFn: () => call(api.GET('/api/labels')) });
}

export function usePlayStatuses() {
  return useQuery({ queryKey: queryKeys.playStatuses, queryFn: () => call(api.GET('/api/play-statuses')) });
}

export function useGames(query: LibraryQuery, enabled = true) {
  return useQuery({
    queryKey: [...queryKeys.games, query],
    queryFn: () => call(api.GET('/api/games', { params: { query } })),
    placeholderData: keepPreviousData,
    enabled,
  });
}

/** Copies matching the library filters. Kept under the games key so game changes refresh it. */
export function useCopies(query: LibraryQuery, enabled = true) {
  return useQuery({
    queryKey: [...queryKeys.games, 'copies', query],
    queryFn: () => call(api.GET('/api/copies', { params: { query } })),
    placeholderData: keepPreviousData,
    enabled,
  });
}

export function useGame(id: number) {
  return useQuery({
    queryKey: queryKeys.game(id),
    queryFn: () => call(api.GET('/api/games/{id}', { params: { path: { id } } })),
  });
}

export function useGameModes() {
  return useQuery({
    queryKey: queryKeys.gameModes,
    queryFn: () => call(api.GET('/api/game-modes')),
    staleTime: Infinity,
  });
}

export function useGenres() {
  return useQuery({ queryKey: queryKeys.genres, queryFn: () => call(api.GET('/api/genres')) });
}

/** Polls once a second while a sync runs. */
export function useIgdbSync(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.igdbSync,
    queryFn: () => call(api.GET('/api/igdb/sync')),
    enabled,
    refetchInterval: (query) => (query.state.data?.running ? 1000 : false),
  });
}

export function useTitleDifferences() {
  return useQuery({ queryKey: queryKeys.igdbTitles, queryFn: () => call(api.GET('/api/igdb/titles')) });
}

export function useDuplicates() {
  return useQuery({ queryKey: queryKeys.duplicates, queryFn: () => call(api.GET('/api/duplicates')) });
}

export function useMissingDetails() {
  return useQuery({ queryKey: queryKeys.missing, queryFn: () => call(api.GET('/api/missing-details')) });
}
