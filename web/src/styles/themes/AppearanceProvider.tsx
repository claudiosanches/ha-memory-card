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
  hidden?: boolean;
  palette: Pick<MantineThemeOverride, 'colors' | 'primaryColor' | 'primaryShade' | 'autoContrast'>;
  variables: {
    variables: typeof variables.variables;
    light: typeof variables.light;
    dark: typeof variables.light;
  };
};

export const SECRET_THEME_ID = 'cursed-90s';

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
  { ...cursed90s, hidden: true },
] as const satisfies readonly VisualTheme[];

export type AppTheme = (typeof themes)[number];

type AppearanceValue = {
  themeId: string;
  selectTheme: (id: string) => void;
  availableThemes: readonly AppTheme[];
  isUnlocked: (id: string) => boolean;
  unlockTheme: (id: string) => void;
};

const AppearanceContext = createContext<AppearanceValue | null>(null);

export function useAppearance() {
  const value = useContext(AppearanceContext);
  if (!value) throw new Error('AppearanceProvider is required');
  return value;
}

export function AppearanceProvider({ children }: { children: ReactNode }) {
  const [savedId, selectTheme] = useLocalStorage({ key: 'memory-card-theme', defaultValue: 'default' });
  const [unlockedIds, setUnlockedIds] = useLocalStorage<string[]>({
    key: 'memory-card-unlocked-themes',
    defaultValue: [],
  });
  const selected = themes.find(({ id }) => id === savedId) ?? themes[0];
  const isUnlocked = (id: string) => unlockedIds.includes(id) || savedId === id;
  const unlockTheme = (id: string) => {
    setUnlockedIds((current) => (current.includes(id) ? current : [...current, id]));
  };
  const availableThemes = themes.filter((theme) => !('hidden' in theme && theme.hidden) || isUnlocked(theme.id));

  // Portaled menus and dialogs share the same theme as the app shell.
  useLayoutEffect(() => {
    document.documentElement.dataset.mcTheme = selected.id;
    return () => { delete document.documentElement.dataset.mcTheme; };
  }, [selected.id]);

  return (
    <AppearanceContext.Provider value={{ themeId: selected.id, selectTheme, availableThemes, isUnlocked, unlockTheme }}>
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
