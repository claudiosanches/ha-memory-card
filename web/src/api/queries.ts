import { useQuery } from '@tanstack/react-query';
import { api } from './client';
import { call } from './errors';

export const queryKeys = {
  status: ['status'],
  platforms: ['platforms'],
  devices: ['devices'],
  stores: ['stores'],
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
