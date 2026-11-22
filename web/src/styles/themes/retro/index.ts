import type { MantineThemeOverride } from '@mantine/core';
import { chartSeries, themePair, variables as base } from '../default/variables';

export const retro = {
  id: 'retro' as const,
  palette: {
    primaryColor: 'brand',
    primaryShade: { light: 8, dark: 3 },
    autoContrast: true,
    colors: {
      brand: ['#fff3df', '#ffe3b9', '#ffcc8a', '#fcb269', '#ef934c', '#d97635', '#c35e28', '#a74821', '#8c391e', '#722c19'],
      dark: ['#f4ead6', '#dac9ab', '#b5a994', '#938470', '#685e50', '#50483e', '#3b352e', '#2d2924', '#221f1b', '#191714'],
    },
  } satisfies MantineThemeOverride,
  variables: {
    variables: {
      ...base.variables,
      '--mc-background-image': 'repeating-linear-gradient(0deg, transparent 0 3px, #90755012 3px 4px)',
      '--mc-header-image': 'linear-gradient(to top, #a94c2b 0 2px, #dd9050 2px 4px, #d3b269 4px 6px, transparent 6px)',
    },
    light: {
      '--mc-background': '#ede3ce',
      '--mc-surface': '#fff7e6',
      '--mc-border': '#c7b79a',
      '--mc-muted': '#665742',
      '--mc-selected': '#efdbb5',
      '--mc-on-accent': '#fff7ea',
      '--mc-cover-shadow': '2px 3px 0 #59402e40',
      '--mantine-color-text': '#352b24',
      '--mantine-color-dimmed': '#665742',
      '--mantine-color-body': '#fff7e6',
      '--mantine-color-default-border': '#c7b79a',
      ...chartSeries.light,
      ...themePair.light,
    },
    dark: {
      '--mc-background': '#221f1b',
      '--mc-surface': '#2d2924',
      '--mc-border': '#51483b',
      '--mc-muted': '#c4b69f',
      '--mc-selected': '#49392d',
      '--mc-on-accent': '#302117',
      '--mc-cover-shadow': '2px 3px 0 #120f0c99',
      '--mantine-color-text': '#f4ead6',
      '--mantine-color-dimmed': '#c4b69f',
      '--mantine-color-body': '#2d2924',
      '--mantine-color-default-border': '#51483b',
      ...chartSeries.dark,
      ...themePair.dark,
    },
  },
};
