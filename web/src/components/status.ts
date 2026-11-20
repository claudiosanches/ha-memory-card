import type { PlayStatus } from '../api/types';

export const STATUS_COLORS: Record<PlayStatus, string> = {
  backlog: 'gray',
  playing: 'blue',
  paused: 'yellow',
  completed: 'green',
  dropped: 'red',
  mastered: 'grape',
};

/** Today in the browser's time zone, as YYYY-MM-DD. */
export function today(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** A YYYY-MM-DD date in the user's locale. */
export function formatDate(value: string, locale?: string): string {
  const [year, month, day] = value.split('-').map(Number);
  if (!year || !month || !day) return value;
  return new Date(year, month - 1, day).toLocaleDateString(locale);
}
