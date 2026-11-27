import type { components, operations } from './schema';

type Schemas = components['schemas'];

export type Condition = Schemas['Condition'];
export type BarcodeLookup = Schemas['BarcodeLookup'];
export type CopyDetail = Schemas['CopyDetail'];
export type CopyInput = Schemas['CopyInput'];
export type CopyPhoto = Schemas['CopyPhoto'];
export type CopyStatus = Schemas['CopyStatus'];
export type Account = Schemas['Account'];
export type AccountInput = Schemas['AccountInput'];
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
export type GameSummary = Schemas['GameSummary'];
export type CopyRow = Schemas['CopyRow'];
export type Label = Schemas['Label'];
export type LabelInput = Schemas['LabelInput'];
export type Person = Schemas['Person'];
export type Platform = Schemas['Platform'];
export type DateFormat = Schemas['DateFormat'];
export type PlatformCategory = Schemas['PlatformCategory'];
export type PlatformInput = Schemas['PlatformInput'];
export type MatchGroup = Schemas['MatchGroup'];
export type PlaySession = Schemas['PlaySession'];
export type PlayStatus = Schemas['PlayStatus'];
export type PlayStatusFields = Schemas['PlayStatusFields'];
export type Region = Schemas['Region'];
export type ReleaseDetail = Schemas['ReleaseDetail'];
export type Store = Schemas['Store'];
export type Stats = Schemas['Stats'];
export type StatusColor = Schemas['StatusColor'];
export type StatusGroup = Schemas['StatusGroup'];
export type StoreInput = Schemas['StoreInput'];
export type Suggestion = Schemas['Suggestion'];
export type Me = Schemas['Me'];
export type MissingDetails = Schemas['MissingDetails'];
export type MissingField = Schemas['MissingField'];
export type LibraryQuery = NonNullable<operations['listGames']['parameters']['query']>;

export const DEVICE_KINDS: readonly DeviceKind[] = ['console', 'handheld', 'pc', 'controller', 'accessory', 'toy'];
/** Kinds that hold copies; the rest is hardware for a platform. */
export const GAME_DEVICE_KINDS: readonly DeviceKind[] = ['console', 'handheld', 'pc'];
export const holdsGames = (kind: DeviceKind) => GAME_DEVICE_KINDS.includes(kind);
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
/** Keys of the play statuses every install has. */
export type BuiltinStatus = 'backlog' | 'playing' | 'paused' | 'completed' | 'dropped' | 'mastered';
export const STATUS_COLORS: readonly StatusColor[] = [
  'gray',
  'red',
  'pink',
  'grape',
  'violet',
  'indigo',
  'blue',
  'cyan',
  'teal',
  'green',
  'lime',
  'yellow',
  'orange',
];
export const STATUS_GROUPS: readonly StatusGroup[] = ['backlog', 'playing', 'paused', 'finished', 'not_counted'];
