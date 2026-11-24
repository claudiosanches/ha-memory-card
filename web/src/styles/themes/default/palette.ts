import { virtualColor, type MantineThemeOverride } from '@mantine/core';

export const palette = {
  primaryColor: 'brand',
  autoContrast: true,
  primaryShade: { light: 8, dark: 8 },
  colors: {
    brand: virtualColor({ name: 'brand', light: 'ocean', dark: 'blue' }),
    ocean: ['#edfbfd', '#e7f8fb', '#cfedf2', '#9edce6', '#69cbd9', '#3dbbcc', '#20aabd', '#138fa3', '#087b8f', '#086578'],
    dark: ['#e7ebf0', '#c6cdd6', '#a1acba', '#788493', '#48525f', '#323c48', '#252d37', '#1b222b', '#141a22', '#10151c'],
  },
} satisfies Pick<MantineThemeOverride, 'primaryColor' | 'autoContrast' | 'primaryShade' | 'colors'>;
