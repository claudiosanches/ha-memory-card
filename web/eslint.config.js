import js from '@eslint/js';
import i18next from 'eslint-plugin-i18next';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist', 'src/api/schema.d.ts'] },
  js.configs.recommended,
  tseslint.configs.recommended,
  reactHooks.configs.flat.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: { globals: globals.browser },
  },
  {
    files: ['src/**/*.tsx'],
    plugins: { i18next },
    rules: {
      // UI text must come from src/locales; attributes users read are checked too.
      'i18next/no-literal-string': [
        'error',
        {
          mode: 'jsx-only',
          'jsx-attributes': {
            include: ['title', 'label', 'placeholder', 'description', 'aria-label', 'alt', 'error'],
            exclude: [],
          },
          // The default also skips any single word ("Library"), which is real UI text.
          words: { exclude: ['[0-9!-/:-@[-`{-~]+', /^\p{Emoji}+$/u] },
          // Arguments to these are keys and identifiers, never display text.
          callees: {
            exclude: [
              'i18n(ext)?',
              't',
              'getInputProps',
              'setFieldError',
              'navigate',
              'useMediaQuery',
              'getRouteApi',
              'includes',
              'startsWith',
              'endsWith',
            ],
          },
        },
      ],
    },
  },
  {
    files: ['src/**/*.test.tsx', 'src/test/**'],
    rules: { 'i18next/no-literal-string': 'off' },
  },
);
