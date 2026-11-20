import type { components } from './schema';

type Schemas = components['schemas'];

export type Device = Schemas['Device'];
export type DeviceInput = Schemas['DeviceInput'];
export type DeviceKind = Schemas['DeviceKind'];
export type Platform = Schemas['Platform'];
export type PlatformInput = Schemas['PlatformInput'];
export type Store = Schemas['Store'];
export type StoreInput = Schemas['StoreInput'];

export const DEVICE_KINDS: readonly DeviceKind[] = ['console', 'handheld', 'pc', 'account'];
