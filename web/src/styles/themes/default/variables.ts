/**
 * Every color the stats charts draw with. A theme spreads these into its
 * light and dark maps and overrides any it wants.
 *
 * The numbered slots color donut slices in fixed order and were checked for
 * color vision deficiency between neighbours, including the last slice
 * against the first. Recheck when changing them.
 */
export const chartColors = {
  light: {
    '--mc-chart-1': '#2a78d6',
    '--mc-chart-2': '#eb6834',
    '--mc-chart-3': '#1baf7a',
    '--mc-chart-4': '#eda100',
    '--mc-chart-5': '#e87ba4',
    '--mc-chart-6': '#008300',
    '--mc-chart-other': '#5f5e5a',
    // Single-series bars and columns, and the track behind a bar.
    '--mc-chart-bar': 'var(--mantine-primary-color-filled)',
    '--mc-chart-track': 'var(--mantine-primary-color-light)',
    '--mc-chart-axis': 'var(--mc-border)',
    // Outline of a legend or list row under the pointer, or tied to the hovered
    // part of a chart, matching the stat tiles.
    '--mc-chart-row-hover': 'var(--mantine-primary-color-filled)',
    // Backlog progress. Not counted sits after not set in the bar.
    '--mc-chart-finished': 'var(--mc-chart-3)',
    '--mc-chart-in-progress': 'var(--mc-chart-4)',
    '--mc-chart-not-started': 'var(--mc-chart-1)',
    '--mc-chart-not-set': 'var(--mc-chart-other)',
    '--mc-chart-not-counted': 'var(--mc-chart-5)',
    '--mc-chart-physical': 'var(--mantine-primary-color-filled)',
    '--mc-chart-digital': 'var(--mantine-primary-color-4)',
    // Play calendar, from no play to the most sessions in a day.
    '--mc-chart-heat-0': 'color-mix(in srgb, var(--mc-border) 50%, transparent)',
    '--mc-chart-heat-1': 'var(--mantine-primary-color-3)',
    '--mc-chart-heat-2': 'var(--mantine-primary-color-5)',
    '--mc-chart-heat-3': 'var(--mantine-primary-color-7)',
    '--mc-chart-heat-4': 'var(--mantine-primary-color-9)',
  },
  dark: {
    '--mc-chart-1': '#3987e5',
    '--mc-chart-2': '#d95926',
    '--mc-chart-3': '#199e70',
    '--mc-chart-4': '#c98500',
    '--mc-chart-5': '#d55181',
    '--mc-chart-6': '#0ca30c',
    '--mc-chart-other': '#c3c2b7',
    '--mc-chart-bar': 'var(--mantine-primary-color-filled)',
    '--mc-chart-track': 'var(--mantine-primary-color-light)',
    '--mc-chart-axis': 'var(--mc-border)',
    '--mc-chart-row-hover': 'var(--mantine-primary-color-filled)',
    '--mc-chart-finished': 'var(--mc-chart-3)',
    '--mc-chart-in-progress': 'var(--mc-chart-4)',
    '--mc-chart-not-started': 'var(--mc-chart-1)',
    '--mc-chart-not-set': 'var(--mc-chart-other)',
    '--mc-chart-not-counted': 'var(--mc-chart-5)',
    '--mc-chart-physical': 'var(--mantine-primary-color-filled)',
    '--mc-chart-digital': 'var(--mantine-primary-color-7)',
    // Brighter means more on a dark surface.
    '--mc-chart-heat-0': 'color-mix(in srgb, var(--mc-border) 50%, transparent)',
    '--mc-chart-heat-1': 'var(--mantine-primary-color-7)',
    '--mc-chart-heat-2': 'var(--mantine-primary-color-5)',
    '--mc-chart-heat-3': 'var(--mantine-primary-color-3)',
    '--mc-chart-heat-4': 'var(--mantine-primary-color-1)',
  },
};

/** Shared contrast adjustments applied beneath each theme's light/dark tokens. */
export const sharedVariables = {
  variables: {},
  light: {
    '--mantine-color-placeholder': 'var(--mc-muted)',
    // Mantine's stock light-variant text colors for warm/green hues fall below 4.5:1 on light tints.
    '--mantine-color-yellow-light-color': '#8f4700',
    '--mantine-color-orange-light-color': '#a83200',
    '--mantine-color-green-light-color': '#1f6b2e',
    '--mantine-color-lime-light-color': '#3f6a00',
    '--mantine-color-teal-light-color': '#06694b',
  },
  dark: {
    '--mantine-color-placeholder': 'var(--mc-muted)',
  },
};

export const variables = {
  variables: {
    '--mc-background-image': 'none',
    '--mc-raised-shadow': '0 8px 20px #00000030',
    '--mc-cover-action-shadow': '0 1px 3px #00000040',
    '--mc-selected-text': 'var(--mantine-color-text)',
  },
  light: {
    '--mc-background': '#f4f6f8',
    '--mc-surface': '#ffffff',
    '--mc-border': '#dfe5eb',
    '--mc-muted': '#576575',
    '--mc-selected': '#edf1f5',
    '--mc-on-accent': '#ffffff',
    '--mc-cover-shadow': '0 4px 12px #14233518, 0 1px 3px #14233512',
    '--mantine-color-text': '#243344',
    '--mantine-color-dimmed': '#576575',
    ...chartColors.light,
    // Teal shades sit too close together here, so these are violet and aqua.
    '--mc-chart-physical': '#6250d6',
    '--mc-chart-digital': '#1baf7a',
  },
  dark: {
    '--mc-background': '#141a22',
    '--mc-surface': '#1b222b',
    '--mc-border': '#303b48',
    '--mc-muted': '#a1acba',
    '--mc-selected': '#2a3440',
    '--mc-on-accent': '#ffffff',
    '--mc-cover-shadow': '0 5px 15px #00000040, 0 1px 3px #00000030',
    '--mantine-color-dimmed': '#a1acba',
    '--mantine-color-text': 'var(--mantine-color-dark-0)',
    '--mantine-color-gray-light': '#2c3542',
    '--mantine-color-gray-light-hover': '#364151',
    '--mantine-color-gray-light-color': '#e7ebf0',
    ...chartColors.dark,
    '--mc-chart-bar': '#4dabf7',
    '--mc-chart-physical': '#9085e9',
    '--mc-chart-digital': '#199e70',
  },
};
