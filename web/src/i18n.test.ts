import { afterEach, describe, expect, it, vi } from 'vitest';
import { browserLanguage } from './i18n';

describe('browserLanguage', () => {
  afterEach(() => vi.restoreAllMocks());

  it.each([
    [['es-MX', 'en'], 'es'],
    [['pt-PT'], 'pt-BR'],
    [['de-DE', 'en-GB'], 'en'],
    [['fr'], 'en'],
  ])('reads %j as %s', (languages, expected) => {
    vi.spyOn(navigator, 'languages', 'get').mockReturnValue(languages);
    expect(browserLanguage()).toBe(expected);
  });
});
