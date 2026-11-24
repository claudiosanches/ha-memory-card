import { virtualColor, type MantineThemeOverride } from '@mantine/core';
import { chartColors, variables as base } from '../default/variables';
import chip from './chip.svg';
import './circuit.css';

const LIGHT_BG = 'radial-gradient(#12784438 1px, transparent 1px)';
const DARK_BG =
  'repeating-linear-gradient(0deg, transparent 0 3px, #00ff660c 3px 4px), radial-gradient(#00ff6624 1px, transparent 1px)';

export const circuit = {
  id: 'circuit' as const,
  palette: {
    primaryColor: 'brand',
    primaryShade: { light: 8, dark: 8 },
    autoContrast: true,
    colors: {
      brand: virtualColor({ name: 'brand', light: 'circuitLight', dark: 'circuitDark' }),
      circuitLight: ['#e8f8ef', '#cef0dc', '#9ee2bb', '#6bd199', '#3ec07b', '#22ab64', '#169653', '#158f4f', '#128448', '#0e6e3b'],
      circuitDark: ['#d6ffe5', '#a6ffc8', '#75ffab', '#4dff94', '#2ef57f', '#17e36c', '#09c95a', '#0c6b36', '#00d964', '#1ce878'],
      gray: ['#e4f2ea', '#d3e8dc', '#bddcc9', '#9ec9b0', '#7ca88f', '#5a856d', '#426b54', '#2c5e43', '#193b28', '#092416'],
      dark: ['#d6ffe5', '#b0e6c4', '#7ac797', '#4e966a', '#246b42', '#174d2e', '#0a1c11', '#07140c', '#040a06', '#020503'],
    },
  } satisfies MantineThemeOverride,
  variables: {
    variables: {
      ...base.variables,
      '--mc-nav-image': `url("${chip}")`,
    },
    light: {
      '--mc-background': '#f1f7f3',
      '--mc-background-image': LIGHT_BG,
      '--mc-header-image': 'linear-gradient(to top, #00a859 0 2px, transparent 2px)',
      '--mc-surface': '#ffffff',
      '--mc-border': '#82b89a',
      '--mc-muted': '#2c5e43',
      '--mc-selected': '#d4f4e2',
      '--mc-on-accent': '#ffffff',
      '--mc-cover-shadow': '0 4px 12px #0924161f, 0 0 0 1px #12784426',
      '--mantine-color-text': '#092416',
      '--mantine-color-dimmed': '#2c5e43',
      '--mantine-color-body': '#ffffff',
      '--mantine-color-default': '#f7fcf9',
      '--mantine-color-default-hover': '#e4f2ea',
      '--mantine-color-default-color': '#092416',
      '--mantine-color-default-border': '#76b291',
      '--mantine-color-anchor': '#0f7540',
      ...chartColors.light,
      '--mc-chart-bar': '#0f7540',
      '--mc-chart-physical': '#0f7540',
      '--mc-chart-digital': '#b45309',
    },
    dark: {
      '--mc-background': '#040a06',
      '--mc-background-image': DARK_BG,
      '--mc-header-image': 'linear-gradient(to top, #00e676 0 2px, transparent 2px), linear-gradient(90deg, #040c07, #081c10)',
      '--mc-surface': '#07140c',
      '--mc-border': '#166337',
      '--mc-muted': '#7ac797',
      '--mc-selected': '#0c2e19',
      '--mc-on-accent': '#031208',
      '--mc-cover-shadow': '0 0 0 1px #00e67640, 0 6px 18px #00000099',
      '--mantine-color-text': '#d6ffe5',
      '--mantine-color-dimmed': '#7ac797',
      '--mantine-color-body': '#07140c',
      '--mantine-color-default-border': '#19733f',
      '--mantine-color-anchor': '#4dff94',
      '--mantine-color-brand-light': '#0c2e19',
      '--mantine-color-brand-light-hover': '#134225',
      '--mantine-color-brand-light-color': '#5effa3',
      '--mantine-primary-color-light': '#0c2e19',
      '--mantine-primary-color-light-hover': '#134225',
      '--mantine-primary-color-light-color': '#5effa3',
      '--mantine-color-gray-light': '#122b1c',
      '--mantine-color-gray-light-hover': '#1a3b27',
      '--mantine-color-gray-light-color': '#d6ffe5',
      ...chartColors.dark,
      '--mc-chart-bar': '#00d964',
      '--mc-chart-physical': '#00d964',
      '--mc-chart-digital': '#ffb300',
    },
  },
};
