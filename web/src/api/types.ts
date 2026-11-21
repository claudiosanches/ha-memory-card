import type { components, operations } from './schema';

type Schemas = components['schemas'];

export type Condition = Schemas['Condition'];
export type CopyDetail = Schemas['CopyDetail'];
export type CopyInput = Schemas['CopyInput'];
export type Device = Schemas['Device'];
export type DeviceInput = Schemas['DeviceInput'];
export type DeviceKind = Schemas['DeviceKind'];
export type DuplicateCopy = Schemas['DuplicateCopy'];
export type DuplicateGroup = Schemas['DuplicateGroup'];
export type TitleDifference = Schemas['TitleDifference'];
export type DuplicateLabel = Schemas['DuplicateLabel'];
export type Format = Schemas['Format'];
export type GameDetail = Schemas['GameDetail'];
export type GameInput = Schemas['GameInput'];
export type GameMode = Schemas['GameMode'];
export type GameSummary = Schemas['GameSummary'];
export type Genre = Schemas['Genre'];
export type Platform = Schemas['Platform'];
export type PlatformCategory = Schemas['PlatformCategory'];
export type PlatformInput = Schemas['PlatformInput'];
export type PlayStatus = Schemas['PlayStatus'];
export type Region = Schemas['Region'];
export type ReleaseDetail = Schemas['ReleaseDetail'];
export type ReleaseInput = Schemas['ReleaseInput'];
export type Store = Schemas['Store'];
export type StoreInput = Schemas['StoreInput'];
export type LibraryQuery = NonNullable<operations['listGames']['parameters']['query']>;

export const DEVICE_KINDS: readonly DeviceKind[] = ['console', 'handheld', 'pc', 'account'];
export const REGIONS: readonly Region[] = ['na', 'eu', 'au', 'nz', 'jp', 'cn', 'asia', 'kr', 'br', 'ww'];
export const FORMATS: readonly Format[] = ['physical', 'digital'];
export const PLATFORM_CATEGORIES: readonly PlatformCategory[] = [
  'console',
  'handheld',
  'computer',
  'mobile',
  'arcade',
  'other',
];
export const DUPLICATE_LABELS: readonly DuplicateLabel[] = [
  'same_release',
  'cross_platform',
  'cross_region',
  'cross_device',
  'physical_digital',
];
export const CONDITIONS: readonly Condition[] = ['sealed', 'cib', 'no_manual', 'loose'];
export const PLAY_STATUSES: readonly PlayStatus[] = [
  'backlog',
  'playing',
  'paused',
  'completed',
  'dropped',
  'mastered',
];
