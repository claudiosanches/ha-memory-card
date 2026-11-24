import { MantineProvider, mergeThemeOverrides, type MantineThemeOverride } from '@mantine/core';
import { useLocalStorage } from '@mantine/hooks';
import { createContext, useContext, useLayoutEffect, type ReactNode } from 'react';
import { sharedTheme } from '../mantine';
import { palette } from './default/palette';
import { sharedVariables, variables } from './default/variables';
import { circuit } from './circuit';
import { cursed90s } from './cursed-90s';
import { cyberpunk } from './cyberpunk';
import { dieselpunk } from './dieselpunk';
import { internet2000s } from './internet-2000s';
import { monochrome } from './monochrome';
import { pastelPlush } from './pastel-plush';
import { retro } from './retro';
import { sage } from './sage';
import { slate } from './slate';
import { steampunk } from './steampunk';
import { warmStone } from './warm-stone';

type VisualTheme = {
  id: string;
  palette: Pick<MantineThemeOverride, 'colors' | 'primaryColor' | 'primaryShade' | 'autoContrast'>;
  variables: {
    variables: typeof variables.variables;
    light: typeof variables.light;
    dark: typeof variables.light;
  };
};

export const themes = [
  { id: 'default', palette, variables },
  monochrome,
  warmStone,
  slate,
  sage,
  retro,
  internet2000s,
  steampunk,
  dieselpunk,
  cyberpunk,
  circuit,
  pastelPlush,
  cursed90s,
] as const satisfies readonly VisualTheme[];

const AppearanceContext = createContext<{ themeId: string; selectTheme: (id: string) => void } | null>(null);

export function useAppearance() {
  const value = useContext(AppearanceContext);
  if (!value) throw new Error('AppearanceProvider is required');
  return value;
}

export function AppearanceProvider({ children }: { children: ReactNode }) {
  const [savedId, selectTheme] = useLocalStorage({ key: 'memory-card-theme', defaultValue: 'default' });
  const selected = themes.find(({ id }) => id === savedId) ?? themes[0];

  // Portaled menus and dialogs share the same theme as the app shell.
  useLayoutEffect(() => {
    document.documentElement.dataset.mcTheme = selected.id;
    return () => { delete document.documentElement.dataset.mcTheme; };
  }, [selected.id]);

  return (
    <AppearanceContext.Provider value={{ themeId: selected.id, selectTheme }}>
      <MantineProvider
        theme={mergeThemeOverrides(sharedTheme, selected.palette)}
        cssVariablesResolver={() => ({
          variables: { ...sharedVariables.variables, ...selected.variables.variables },
          light: { ...sharedVariables.light, ...selected.variables.light },
          dark: { ...sharedVariables.dark, ...selected.variables.dark },
        })}
        defaultColorScheme="auto"
      >
        {children}
      </MantineProvider>
    </AppearanceContext.Provider>
  );
}
