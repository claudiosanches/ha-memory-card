/**
 * Categorical chart slots, in fixed order, checked for color vision
 * deficiency between neighbours (including the last donut slice against the
 * first). Themes share these unless they validate their own.
 */
export const chartSeries = {
  light: {
    '--mc-chart-1': '#2a78d6',
    '--mc-chart-2': '#eb6834',
    '--mc-chart-3': '#1baf7a',
    '--mc-chart-4': '#eda100',
    '--mc-chart-5': '#e87ba4',
    '--mc-chart-6': '#008300',
    '--mc-chart-other': '#5f5e5a',
  },
  dark: {
    '--mc-chart-1': '#3987e5',
    '--mc-chart-2': '#d95926',
    '--mc-chart-3': '#199e70',
    '--mc-chart-4': '#c98500',
    '--mc-chart-5': '#d55181',
    '--mc-chart-6': '#0ca30c',
    '--mc-chart-other': '#c3c2b7',
  },
};

/** A strong and a soft shade of the theme color, for charts that split one thing in two. */
export const themePair = {
  light: { '--mc-chart-pair-1': 'var(--mantine-primary-color-filled)', '--mc-chart-pair-2': 'var(--mantine-primary-color-4)' },
  dark: { '--mc-chart-pair-1': 'var(--mantine-primary-color-filled)', '--mc-chart-pair-2': 'var(--mantine-primary-color-7)' },
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
    ...chartSeries.light,
    // Teal shades sit too close together here, so the pair is violet and aqua.
    '--mc-chart-pair-1': '#6250d6',
    '--mc-chart-pair-2': '#1baf7a',
  },
  dark: {
    '--mc-background': '#141a22',
    '--mc-surface': '#1b222b',
    '--mc-border': '#303b48',
    '--mc-muted': '#a1acba',
    '--mc-selected': '#2a3440',
    '--mc-on-accent': '#11243b',
    '--mc-cover-shadow': '0 5px 15px #00000040, 0 1px 3px #00000030',
    '--mantine-color-dimmed': '#a1acba',
    '--mantine-color-text': 'var(--mantine-color-dark-0)',
    ...chartSeries.dark,
    '--mc-chart-pair-1': '#9085e9',
    '--mc-chart-pair-2': '#199e70',
  },
};
