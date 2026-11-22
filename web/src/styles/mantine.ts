import { createTheme, defaultVariantColorsResolver } from '@mantine/core';

export const sharedTheme = createTheme({
  fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  headings: { fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif", fontWeight: '650' },
  spacing: { compact: '0.375rem', card: '1rem', section: '1.5rem' },
  defaultRadius: 'md',
  cursorType: 'pointer',
  variantColorResolver: (input) => {
    const colors = defaultVariantColorsResolver(input);
    return input.color === 'brand' && input.variant === 'filled'
      ? { ...colors, color: 'var(--mc-on-accent)' }
      : colors;
  },
  components: {
    InputWrapper: {
      defaultProps: { inputWrapperOrder: ['label', 'input', 'description', 'error'] },
      styles: { label: { marginBottom: 6, fontWeight: 600 } },
    },
    Button: { defaultProps: { fw: 600 } },
    Card: { defaultProps: { padding: 'lg' } },
  },
});
