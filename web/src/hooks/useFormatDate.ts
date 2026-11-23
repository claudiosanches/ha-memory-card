import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useStatus } from '../api/queries';
import { formatDate } from '../components/status';

/** Writes dates the way the app's language does, or in the format the app options name. */
export function useFormatDate(): (value: string) => string {
  const { i18n } = useTranslation();
  const format = useStatus().data?.date_format ?? 'language';
  return useCallback((value: string) => formatDate(value, i18n.language, format), [i18n.language, format]);
}
