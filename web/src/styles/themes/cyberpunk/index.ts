import { virtualColor, type MantineThemeOverride } from '@mantine/core';
import { chartColors, variables as base } from '../default/variables';
import './cyberpunk.css';
import emblem from './emblem.svg';

const LIGHT_GRID =
  'linear-gradient(#0077990d 1px, transparent 1px) 0 0 / 36px 36px, linear-gradient(90deg, #0077990d 1px, transparent 1px) 0 0 / 36px 36px';
const DARK_GRID =
  'linear-gradient(#00e5ff0e 1px, transparent 1px) 0 0 / 36px 36px, linear-gradient(90deg, #ff2a850c 1px, transparent 1px) 0 0 / 36px 36px';

export const cyberpunk = {
  id: 'cyberpunk' as const,
  palette: {
    primaryColor: 'brand',
    primaryShade: { light: 8, dark: 8 },
    autoContrast: true,
    colors: {
      brand: virtualColor({ name: 'brand', light: 'cyberLight', dark: 'cyberDark' }),
      cyberLight: ['#ffeaf4', '#ffd0e6', '#ff9ec9', '#f76ca9', '#ea408c', '#db2276', '#c91769', '#be1466', '#b81362', '#990e51'],
      cyberDark: ['#ffd4ea', '#ffadd8', '#ff85c4', '#ff5cb0', '#f73b9b', '#eb2689', '#db1d7d', '#9e1762', '#d41776', '#e82989'],
      gray: ['#e8eef7', '#d8e1f0', '#c5d1e5', '#b3c1db', '#8f9ebe', '#6b7b9e', '#526183', '#445070', '#2b334d', '#16182c'],
      dark: ['#eef2ff', '#cdd4f0', '#a9b2d9', '#7f8ab8', '#4f487a', '#352e60', '#211d3d', '#131126', '#090814', '#05040c'],
    },
  } satisfies MantineThemeOverride,
  variables: {
    variables: {
      ...base.variables,
      '--mc-nav-image': `url("${emblem}")`,
    },
    light: {
      '--mc-background': '#ecf1f8',
      '--mc-background-image': LIGHT_GRID,
      '--mc-header-image': 'linear-gradient(to top, #00a8c6 0 2px, transparent 2px), linear-gradient(90deg, #ffffff 0%, #f2f7ff 55%, #fff0f7 100%)',
      '--mc-surface': '#ffffff',
      '--mc-border': '#bcc8de',
      '--mc-muted': '#4b5575',
      '--mc-selected': '#ffe3f1',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 4px 14px #16182c1f, 0 0 0 1px #0099b826',
      '--mantine-color-text': '#16182c',
      '--mantine-color-dimmed': '#4b5575',
      '--mantine-color-body': '#ffffff',
      '--mantine-color-default': '#f8faff',
      '--mantine-color-default-hover': '#e8eef7',
      '--mantine-color-default-border': '#bcc8de',
      '--mantine-color-anchor': '#00768f',
      ...chartColors.light,
      '--mc-chart-bar': '#c91769',
      '--mc-chart-physical': '#c91769',
      '--mc-chart-digital': '#0099b8',
    },
    dark: {
      '--mc-background': '#090814',
      '--mc-background-image': DARK_GRID,
      '--mc-header-image':
        'linear-gradient(to top, #00e5ff 0 2px, transparent 2px), linear-gradient(90deg, #0d0b1e 0%, #1d1038 55%, #0c1f2e 100%)',
      '--mc-surface': '#131126',
      '--mc-border': '#31285c',
      '--mc-muted': '#a9b2d9',
      '--mc-selected': '#241a4a',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 0 0 1px #00e5ff40, 0 6px 18px #00000080',
      '--mantine-color-text': '#eef2ff',
      '--mantine-color-dimmed': '#a9b2d9',
      '--mantine-color-body': '#131126',
      '--mantine-color-default-border': '#31285c',
      '--mantine-color-anchor': '#00e5ff',
      '--mantine-color-brand-light': '#34143a',
      '--mantine-color-brand-light-hover': '#461b4e',
      '--mantine-color-brand-light-color': '#ff9ed0',
      '--mantine-primary-color-light': '#34143a',
      '--mantine-primary-color-light-hover': '#461b4e',
      '--mantine-primary-color-light-color': '#ff9ed0',
      '--mantine-color-gray-light': '#231f42',
      '--mantine-color-gray-light-hover': '#2e2954',
      '--mantine-color-gray-light-color': '#eef2ff',
      ...chartColors.dark,
      '--mc-chart-bar': '#ff3b9a',
      '--mc-chart-physical': '#ff3b9a',
      '--mc-chart-digital': '#00e5ff',
    },
  },
};
