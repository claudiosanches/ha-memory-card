import type { MantineThemeOverride } from '@mantine/core';
import { chartColors, variables as base } from '../default/variables';
import './portal.css';
import stars from './stars.gif';
import globe from './globe.gif';
import stillStars from './stars.png';
import stillGlobe from './globe.png';

export const internet2000s = {
  id: 'internet-2000s' as const,
  palette: {
    primaryColor: 'brand',
    primaryShade: { light: 8, dark: 3 },
    autoContrast: true,
    colors: {
      brand: ['#f1f2ff', '#dfe4ff', '#c6d0ff', '#aabaff', '#92a4ff', '#788cfb', '#6275ea', '#4d5dcd', '#3948af', '#303b8a'],
      dark: ['#eaf0ff', '#ced9f7', '#aab7da', '#828faf', '#536187', '#39476c', '#283655', '#1d2947', '#141d36', '#10172b'],
    },
  } satisfies MantineThemeOverride,
  variables: {
    variables: {
      ...base.variables,
      '--mc-background-image': `url("${stars}")`,
      '--mc-still-background-image': `url("${stillStars}")`,
      '--mc-still-nav-image': `url("${stillGlobe}")`,
      '--mc-nav-image': `url("${globe}")`,
      '--mc-header-image': [
        'radial-gradient(ellipse 32% 220% at 77% 165%, transparent 62%, #79ecff85 62.6%, transparent 64%, transparent 68%, #b4ddff50 68.6%, transparent 70%)',
        'linear-gradient(108deg, #12274e 5%, #104891 45%, #08628f 75%, #242c70)',
      ].join(', '),
      '--mc-selected-text': '#ffffff',
    },
    light: {
      '--mc-background': '#e3eaff',
      '--mc-surface': '#f6f8ff',
      '--mc-border': '#acbadd',
      '--mc-muted': '#4c5d81',
      '--mc-selected': '#224da2',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 1px 0 #ffffff, 0 3px 7px #384b8940',
      '--mantine-color-body': '#f6f8ff',
      '--mantine-color-text': '#202c51',
      '--mantine-color-dimmed': '#4c5d81',
      '--mantine-color-default-border': '#acbadd',
      ...chartColors.light,
    },
    dark: {
      '--mc-background': '#141d36',
      '--mc-surface': '#1d2947',
      '--mc-border': '#435175',
      '--mc-muted': '#b4c2e2',
      '--mc-selected': '#234d86',
      '--mc-on-accent': '#162142',
      '--mc-cover-shadow': '0 0 0 1px #7084be66, 0 3px 9px #090d2266',
      '--mantine-color-body': '#1d2947',
      '--mantine-color-text': '#eaf0ff',
      '--mantine-color-dimmed': '#b4c2e2',
      '--mantine-color-default-border': '#435175',
      ...chartColors.dark,
    },
  },
};
