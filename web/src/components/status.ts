import type { DateFormat } from '../api/types';

/** Today in the browser's time zone, as YYYY-MM-DD. */
export function today(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** A YYYY-MM-DD date, or the day of a timestamp, as `format` writes it. */
export function formatDate(value: string, locale?: string, format: DateFormat = 'language'): string {
  let date: Date;
  if (value.includes('T')) {
    date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
  } else {
    const [year, month, day] = value.split('-').map(Number);
    if (!year || !month || !day) return value;
    date = new Date(year, month - 1, day);
  }
  if (format === 'language') return date.toLocaleDateString(locale);
  const pad = (n: number) => String(n).padStart(2, '0');
  const [y, m, d] = [String(date.getFullYear()), pad(date.getMonth() + 1), pad(date.getDate())];
  if (format === 'dmy') return `${d}/${m}/${y}`;
  if (format === 'mdy') return `${m}/${d}/${y}`;
  return `${y}-${m}-${d}`;
}
