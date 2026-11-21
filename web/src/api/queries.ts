import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { api } from './client';
import { call } from './errors';
import type { LibraryQuery } from './types';

export const queryKeys = {
  status: ['status'],
  platforms: ['platforms'],
  devices: ['devices'],
  stores: ['stores'],
  games: ['games'],
  game: (id: number) => ['game', id] as const,
  gameModes: ['game-modes'],
  genres: ['genres'],
  igdbSync: ['igdb-sync'],
  igdbTitles: ['igdb-titles'],
  duplicates: ['duplicates'],
} as const;

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

export function useGames(query: LibraryQuery) {
  return useQuery({
    queryKey: [...queryKeys.games, query],
    queryFn: () => call(api.GET('/api/games', { params: { query } })),
    placeholderData: keepPreviousData,
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
