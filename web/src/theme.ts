import { createTheme, defaultVariantColorsResolver, virtualColor } from '@mantine/core';

export const theme = createTheme({
  primaryColor: 'brand',
  autoContrast: true,
  primaryShade: { light: 7, dark: 4 },
  colors: {
    brand: virtualColor({ name: 'brand', light: 'ocean', dark: 'blue' }),
    ocean: ['#e7f8fb', '#cfedf2', '#9edce6', '#69cbd9', '#3dbbcc', '#20aabd', '#138fa3', '#087b8f', '#086578', '#085363'],
    dark: ['#e7ebf0', '#c6cdd6', '#a1acba', '#788493', '#48525f', '#323c48', '#252d37', '#1b222b', '#141a22', '#10151c'],
  },
  fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  headings: { fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif", fontWeight: '650' },
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
