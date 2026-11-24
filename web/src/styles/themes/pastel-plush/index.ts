import { virtualColor, type MantineThemeOverride } from '@mantine/core';
import { chartColors, variables as base } from '../default/variables';
import './plush.css';
import bearBg from './bear-bg.svg';
import bear from './bear.svg';
import bunnyBg from './bunny-bg.svg';
import bunny from './bunny.svg';

export const pastelPlush = {
  id: 'pastel-plush' as const,
  palette: {
    primaryColor: 'brand',
    primaryShade: { light: 8, dark: 8 },
    autoContrast: true,
    colors: {
      brand: virtualColor({ name: 'brand', light: 'plushLight', dark: 'plushDark' }),
      plushLight: ['#fff0f6', '#ffdeeb', '#fcc2d7', '#faa2c1', '#f783ac', '#ec6296', '#db4680', '#c9336f', '#b82564', '#9a1c52'],
      plushDark: ['#ffe3f0', '#ffc4df', '#ffa3cd', '#f781b7', '#f57eb5', '#dc4b8e', '#c93a7d', '#8c2859', '#c43276', '#d94589'],
      gray: ['#fae8f0', '#f5d9e6', '#ecc4d7', '#e5b3ca', '#c993ac', '#a6728b', '#87576f', '#75485e', '#593346', '#422232'],
      dark: ['#fdeef6', '#e6cad9', '#cfa9be', '#b0879e', '#7d596f', '#5e3f53', '#422a3a', '#2c1d27', '#20151c', '#170f14'],
    },
  } satisfies MantineThemeOverride,
  variables: {
    variables: {
      ...base.variables,
    },
    light: {
      '--mc-background': '#fdeef4',
      '--mc-background-image': `url("${bunnyBg}")`,
      '--mc-nav-image': `url("${bunny}")`,
      '--mc-header-image': 'linear-gradient(to top, #f3a6c6 0 3px, transparent 3px)',
      '--mc-surface': '#fff9fb',
      '--mc-border': '#ecc0d4',
      '--mc-muted': '#75485e',
      '--mc-selected': '#fad6e6',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 4px 12px #b84a7c24, 0 1px 3px #b84a7c18',
      '--mantine-color-text': '#422232',
      '--mantine-color-dimmed': '#75485e',
      '--mantine-color-body': '#fff9fb',
      '--mantine-color-default': '#fffcfe',
      '--mantine-color-default-hover': '#fae8f0',
      '--mantine-color-default-color': '#422232',
      '--mantine-color-default-border': '#ecc0d4',
      ...chartColors.light,
      '--mc-chart-bar': '#c9336f',
      '--mc-chart-physical': '#c9336f',
      '--mc-chart-digital': '#7c65d6',
    },
    dark: {
      '--mc-background': '#20151c',
      '--mc-background-image': `url("${bearBg}")`,
      '--mc-nav-image': `url("${bear}")`,
      '--mc-header-image': 'linear-gradient(to top, #e282ad 0 3px, transparent 3px)',
      '--mc-surface': '#2c1d27',
      '--mc-border': '#55394c',
      '--mc-muted': '#d6afc6',
      '--mc-selected': '#472a3f',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 5px 14px #0e060b80, 0 0 0 1px #e282ad26',
      '--mantine-color-text': '#fdeef6',
      '--mantine-color-dimmed': '#d6afc6',
      '--mantine-color-body': '#2c1d27',
      '--mantine-color-default-border': '#55394c',
      '--mantine-color-brand-light': '#4c243b',
      '--mantine-color-brand-light-hover': '#5e2e4a',
      '--mantine-color-brand-light-color': '#ffc4df',
      '--mantine-primary-color-light': '#4c243b',
      '--mantine-primary-color-light-hover': '#5e2e4a',
      '--mantine-primary-color-light-color': '#ffc4df',
      '--mantine-color-gray-light': '#3f2836',
      '--mantine-color-gray-light-hover': '#4e3243',
      '--mantine-color-gray-light-color': '#fdeef6',
      ...chartColors.dark,
      '--mc-chart-bar': '#f282b4',
      '--mc-chart-physical': '#f282b4',
      '--mc-chart-digital': '#b39ddb',
    },
  },
};
