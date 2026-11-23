import { describe, expect, it } from 'vitest';
import { formatDate } from './status';

describe('formatDate', () => {
  it('follows the language by default', () => {
    expect(formatDate('2026-11-05', 'en')).toBe('11/5/2026');
    expect(formatDate('2026-11-05', 'pt-BR')).toBe('05/11/2026');
  });

  it('writes the format the app options name, whatever the language', () => {
    expect(formatDate('2026-11-05', 'en', 'dmy')).toBe('05/11/2026');
    expect(formatDate('2026-11-05', 'pt-BR', 'mdy')).toBe('11/05/2026');
    expect(formatDate('2026-11-05', 'es', 'ymd')).toBe('2026-11-05');
  });

  it('takes the local day of a timestamp and leaves other text alone', () => {
    expect(formatDate(new Date(2026, 10, 5, 12).toISOString(), 'en', 'ymd')).toBe('2026-11-05');
    expect(formatDate('soon', 'en', 'dmy')).toBe('soon');
  });
});
