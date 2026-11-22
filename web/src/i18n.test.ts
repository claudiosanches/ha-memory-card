import { afterEach, describe, expect, it, vi } from 'vitest';
import i18n, { automaticLanguage, browserLanguage, chooseLanguage, followHomeAssistant } from './i18n';

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

describe('followHomeAssistant', () => {
  afterEach(async () => {
    vi.restoreAllMocks();
    chooseLanguage(null);
    followHomeAssistant(null);
    await i18n.changeLanguage('en');
  });

  it("follows Home Assistant's language until someone picks one", () => {
    vi.spyOn(navigator, 'languages', 'get').mockReturnValue(['en-US']);
    followHomeAssistant('pt-BR');
    expect(automaticLanguage()).toBe('pt-BR');
    expect(i18n.language).toBe('pt-BR');
    expect(localStorage.getItem('memory-card-home-assistant-language')).toBe('pt-BR');

    chooseLanguage('es');
    followHomeAssistant('pt-BR');
    expect(i18n.language).toBe('es');

    chooseLanguage(null);
    expect(i18n.language).toBe('pt-BR');
  });

  it("falls back to the browser for a language the app does not have", () => {
    vi.spyOn(navigator, 'languages', 'get').mockReturnValue(['es-ES']);
    followHomeAssistant('de');
    expect(automaticLanguage()).toBe('es');
    expect(i18n.language).toBe('es');
  });
});
