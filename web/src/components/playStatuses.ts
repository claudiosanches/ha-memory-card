import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { usePlayStatuses } from '../api/queries';
import type { BuiltinStatus, PlayStatus } from '../api/types';

export type NamedStatus = PlayStatus & { name: string };

/** The play statuses in the user's order, with display names. */
export function usePlayStatusList() {
  const { t } = useTranslation();
  const statuses = usePlayStatuses();
  return useMemo(() => {
    const list: NamedStatus[] = (statuses.data ?? []).map((s) => ({
      ...s,
      name: s.label ?? t(`playStatus.${s.key as BuiltinStatus}`),
    }));
    const byKey = new Map(list.map((s) => [s.key, s]));
    return {
      list,
      isPending: statuses.isPending,
      get: (key: string) => byKey.get(key),
      name: (key: string) => byKey.get(key)?.name ?? key,
      color: (key: string) => byKey.get(key)?.color ?? 'gray',
      options: list.map((s) => ({ value: s.key, label: s.name })),
    };
  }, [statuses.data, statuses.isPending, t]);
}
