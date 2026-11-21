import { MantineProvider, mergeThemeOverrides, type MantineThemeOverride } from '@mantine/core';
import { useLocalStorage } from '@mantine/hooks';
import { createContext, useContext, type ReactNode } from 'react';
import { sharedTheme } from '../mantine';
import { palette } from './default/palette';
import { variables } from './default/variables';

type VisualTheme = {
  id: string;
  palette: Pick<MantineThemeOverride, 'colors' | 'primaryColor' | 'primaryShade' | 'autoContrast'>;
  variables: {
    variables: typeof variables.variables;
    light: typeof variables.light;
    dark: typeof variables.light;
  };
};

export const themes = [{ id: 'default', palette, variables }] as const satisfies readonly VisualTheme[];

const AppearanceContext = createContext<{ themeId: string; selectTheme: (id: string) => void } | null>(null);

export function useAppearance() {
  const value = useContext(AppearanceContext);
  if (!value) throw new Error('AppearanceProvider is required');
  return value;
}

export function AppearanceProvider({ children }: { children: ReactNode }) {
  const [savedId, selectTheme] = useLocalStorage({ key: 'memory-card-theme', defaultValue: 'default' });
  const selected = themes.find(({ id }) => id === savedId) ?? themes[0];

  return (
    <AppearanceContext.Provider value={{ themeId: selected.id, selectTheme }}>
      <MantineProvider
        theme={mergeThemeOverrides(sharedTheme, selected.palette)}
        cssVariablesResolver={() => selected.variables}
        defaultColorScheme="auto"
      >
        {children}
      </MantineProvider>
    </AppearanceContext.Provider>
  );
}
