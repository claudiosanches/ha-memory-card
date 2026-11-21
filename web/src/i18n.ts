import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en.json';
import ptBR from './locales/pt-BR.json';

export const defaultNS = 'translation';
export const resources = { en: { translation: en }, 'pt-BR': { translation: ptBR } } as const;

export const LANGUAGES = ['en', 'pt-BR'] as const;
export type Language = (typeof LANGUAGES)[number];
/** Each language's name in itself, so anyone can find theirs. */
export const LANGUAGE_NAMES: Record<Language, string> = { en: 'English', 'pt-BR': 'Português (Brasil)' };

const STORAGE_KEY = 'memory-card-language';

/** The first of the browser's languages the app has, else English. */
export function browserLanguage(): Language {
  for (const tag of navigator.languages ?? [navigator.language]) {
    const lower = tag.toLowerCase();
    if (lower.startsWith('pt')) return 'pt-BR';
    if (lower.startsWith('en')) return 'en';
  }
  return 'en';
}

/** The language picked in Settings, or null to follow the browser. */
export function savedLanguage(): Language | null {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    return LANGUAGES.find((language) => language === saved) ?? null;
  } catch {
    return null;
  }
}

export function chooseLanguage(choice: Language | null) {
  try {
    if (choice) localStorage.setItem(STORAGE_KEY, choice);
    else localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Without storage the choice lasts until the page reloads.
  }
  void i18n.changeLanguage(choice ?? browserLanguage());
}

i18n.on('languageChanged', (language) => {
  document.documentElement.lang = language;
});

void i18n.use(initReactI18next).init({
  lng: savedLanguage() ?? browserLanguage(),
  fallbackLng: 'en',
  defaultNS,
  resources,
  interpolation: { escapeValue: false },
});

export default i18n;
