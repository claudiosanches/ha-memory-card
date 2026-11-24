import { virtualColor, type MantineThemeOverride } from '@mantine/core';
import { chartColors, variables as base } from '../default/variables';
import './cyberpunk.css';
import emblem from './emblem.svg';

const LIGHT_BG = [
  'radial-gradient(ellipse 80% 42% at 50% 0%, #ff79c622, transparent 70%)',
  'linear-gradient(#9b5de518 1px, transparent 1px)',
  'linear-gradient(90deg, #00bbf918 1px, transparent 1px)',
].join(', ');

const DARK_BG = [
  'radial-gradient(ellipse 80% 45% at 50% 0%, #ff2a9d28, transparent 70%)',
  'linear-gradient(#ff2a9d1a 1px, transparent 1px)',
  'linear-gradient(90deg, #00f5ff18 1px, transparent 1px)',
].join(', ');

export const cyberpunk = {
  id: 'cyberpunk' as const,
  palette: {
    primaryColor: 'brand',
    primaryShade: { light: 8, dark: 8 },
    autoContrast: true,
    colors: {
      brand: virtualColor({ name: 'brand', light: 'cyberLight', dark: 'cyberDark' }),
      cyberLight: ['#ffe8f6', '#ffcce9', '#ff9ed6', '#f76ec0', '#e843a5', '#d6228d', '#c7187e', '#bd1574', '#b81570', '#960f5b'],
      cyberDark: ['#ffd6ee', '#ffb0df', '#ff87cf', '#ff5ec0', '#fa46b3', '#eb269a', '#db1c8b', '#9e176c', '#d91b80', '#ec2e93'],
      gray: ['#f0e6fa', '#e4d5f5', '#d5bef0', '#c5a8e8', '#a387c9', '#7d65a3', '#634d85', '#523e70', '#382557', '#251342'],
      dark: ['#f8f0ff', '#dccbf5', '#c2b2e8', '#9683c7', '#5e4494', '#402670', '#23144a', '#170d33', '#0d061f', '#070312'],
    },
  } satisfies MantineThemeOverride,
  variables: {
    variables: {
      ...base.variables,
      '--mc-nav-image': `url("${emblem}")`,
    },
    light: {
      '--mc-background': '#f5ecfc',
      '--mc-background-image': LIGHT_BG,
      '--mc-header-image': [
        'linear-gradient(to top, #3ce8ff 0 2px, #ff70c8 2px 3px, transparent 3px)',
        'linear-gradient(90deg, #c91878 0%, #7b2cbf 52%, #0088b8 100%)',
      ].join(', '),
      '--mc-surface': '#ffffff',
      '--mc-border': '#d5bef0',
      '--mc-muted': '#5c487a',
      '--mc-selected': '#f5d8f7',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 4px 14px #5a189a24, 0 0 0 1px #c9187822',
      '--mantine-color-text': '#251342',
      '--mantine-color-dimmed': '#5c487a',
      '--mantine-color-body': '#ffffff',
      '--mantine-color-default': '#fcf8ff',
      '--mantine-color-default-hover': '#f0e6fa',
      '--mantine-color-default-color': '#251342',
      '--mantine-color-default-border': '#cbb0eb',
      '--mantine-color-anchor': '#7b2cbf',
      ...chartColors.light,
      '--mc-chart-bar': '#c91878',
      '--mc-chart-physical': '#c91878',
      '--mc-chart-digital': '#0088b8',
    },
    dark: {
      '--mc-background': '#0d061f',
      '--mc-background-image': DARK_BG,
      '--mc-header-image':
        'linear-gradient(to top, #ff2a9d 0 2px, #00f5ff 2px 3px, transparent 3px), linear-gradient(90deg, #1b0a3d 0%, #3a0f5c 50%, #102a4c 100%)',
      '--mc-surface': '#170d33',
      '--mc-border': '#49237a',
      '--mc-muted': '#c2b2e8',
      '--mc-selected': '#2d1459',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 0 0 1px #ff2a9d44, 0 6px 20px #06021299',
      '--mantine-color-text': '#f8f0ff',
      '--mantine-color-dimmed': '#c2b2e8',
      '--mantine-color-body': '#170d33',
      '--mantine-color-default-border': '#5a2c96',
      '--mantine-color-anchor': '#00f5ff',
      '--mantine-color-brand-light': '#3d1254',
      '--mantine-color-brand-light-hover': '#50186e',
      '--mantine-color-brand-light-color': '#ff99d6',
      '--mantine-primary-color-light': '#3d1254',
      '--mantine-primary-color-light-hover': '#50186e',
      '--mantine-primary-color-light-color': '#ff99d6',
      '--mantine-color-gray-light': '#2a1a52',
      '--mantine-color-gray-light-hover': '#362266',
      '--mantine-color-gray-light-color': '#f8f0ff',
      ...chartColors.dark,
      '--mc-chart-bar': '#ff3b9a',
      '--mc-chart-physical': '#ff3b9a',
      '--mc-chart-digital': '#00f5ff',
    },
  },
};
