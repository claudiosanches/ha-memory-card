import type { TFunction } from 'i18next';
import type { Person } from '../api/types';
import { formatDate } from './status';

type Tracked = { created_at: string; created_by?: Person | null; updated_by?: Person | null };

/** Who added something and when, and who last changed it if someone else did. */
export function history(item: Tracked, t: TFunction, locale: string): string {
  const date = formatDate(item.created_at, locale);
  const name = (person: Person) => person.name ?? t('history.someone');
  const added = item.created_by
    ? t('history.addedBy', { name: name(item.created_by), date })
    : t('history.added', { date });
  const editor = item.updated_by;
  if (!editor || editor.id === item.created_by?.id) return added;
  return `${added} ${t('history.changedBy', { name: name(editor) })}`;
}
