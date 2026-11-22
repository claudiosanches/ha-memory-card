import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en.json';
import es from './locales/es.json';
import ptBR from './locales/pt-BR.json';

export const defaultNS = 'translation';
export const resources = {
  en: { translation: en },
  es: { translation: es },
  'pt-BR': { translation: ptBR },
} as const;

export const LANGUAGES = ['en', 'es', 'pt-BR'] as const;
export type Language = (typeof LANGUAGES)[number];
/** Each language's name in itself, so anyone can find theirs. */
export const LANGUAGE_NAMES: Record<Language, string> = { en: 'English', es: 'Español', 'pt-BR': 'Português (Brasil)' };

const STORAGE_KEY = 'memory-card-language';
/** Home Assistant's language as last seen, so a reload starts in it. */
const HOME_ASSISTANT_KEY = 'memory-card-home-assistant-language';

/** The app's language for a language tag such as `pt-BR`, if it has one. */
export function supportedLanguage(tag: string): Language | null {
  const lower = tag.toLowerCase();
  if (lower.startsWith('pt')) return 'pt-BR';
  if (lower.startsWith('es')) return 'es';
  if (lower.startsWith('en')) return 'en';
  return null;
}

/** The first of the browser's languages the app has, else English. */
export function browserLanguage(): Language {
  for (const tag of navigator.languages ?? [navigator.language]) {
    const language = supportedLanguage(tag);
    if (language) return language;
  }
  return 'en';
}

let homeAssistant: Language | null = (() => {
  try {
    const seen = localStorage.getItem(HOME_ASSISTANT_KEY);
    return LANGUAGES.find((language) => language === seen) ?? null;
  } catch {
    return null;
  }
})();

/** What Automatic means: Home Assistant's language when the app has it, else the browser's. */
export function automaticLanguage(): Language {
  return homeAssistant ?? browserLanguage();
}

/** Keeps Home Assistant's language, and switches to it unless someone picked a language. */
export function followHomeAssistant(tag: string | null) {
  homeAssistant = tag ? supportedLanguage(tag) : null;
  try {
    if (homeAssistant) localStorage.setItem(HOME_ASSISTANT_KEY, homeAssistant);
    else localStorage.removeItem(HOME_ASSISTANT_KEY);
  } catch {
    // Without storage the next load starts in the browser's language.
  }
  if (!savedLanguage() && i18n.language !== automaticLanguage()) void i18n.changeLanguage(automaticLanguage());
}

/** The language picked in Settings, or null for Automatic. */
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
  void i18n.changeLanguage(choice ?? automaticLanguage());
}

i18n.on('languageChanged', (language) => {
  document.documentElement.lang = language;
});

void i18n.use(initReactI18next).init({
  lng: savedLanguage() ?? automaticLanguage(),
  fallbackLng: 'en',
  defaultNS,
  resources,
  interpolation: { escapeValue: false },
});

export default i18n;
