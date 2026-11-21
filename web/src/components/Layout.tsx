import { AppShell, Burger, Group, NavLink, Text } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import {
  IconCopy,
  IconStack2,
  IconDeviceGamepad2,
  IconFileImport,
  IconLibrary,
  IconSettings,
  IconDeviceSdCard,
} from '@tabler/icons-react';
import { Link, Outlet, useRouterState } from '@tanstack/react-router';
import type { ComponentType } from 'react';
import { useTranslation } from 'react-i18next';
import { ThemePicker } from './ThemePicker';
import classes from './Layout.module.css';

type NavKey = 'library' | 'backlog' | 'duplicates' | 'import' | 'manage' | 'settings';

const NAV: { key: NavKey; to: string; icon: ComponentType<{ size?: number }> }[] = [
  { key: 'library', to: '/', icon: IconLibrary },
  { key: 'backlog', to: '/backlog', icon: IconStack2 },
  { key: 'duplicates', to: '/duplicates', icon: IconCopy },
  { key: 'import', to: '/import', icon: IconFileImport },
  { key: 'manage', to: '/manage', icon: IconDeviceGamepad2 },
  { key: 'settings', to: '/settings', icon: IconSettings },
];

export function Layout() {
  const { t } = useTranslation();
  const [opened, { toggle, close }] = useDisclosure();
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return (
    <AppShell
      header={{ height: 68 }}
      navbar={{ width: 224, breakpoint: 'sm', collapsed: { mobile: !opened } }}
      padding={0}
    >
      <a href="#main-content" className={classes.skipLink} onClick={(event) => {
        // The app uses hash routing; focusing content must not change the route.
        event.preventDefault();
        document.getElementById('main-content')?.focus();
      }}>{t('app.skipNavigation')}</a>
      <AppShell.Header className={classes.header}>
        <Group h="100%" justify="space-between" px="lg">
          <Group gap="sm">
            <Burger
              opened={opened}
              onClick={toggle}
              hiddenFrom="sm"
              size="sm"
              aria-label={t('app.toggleNavigation')}
            />
            <span className={classes.logo}><IconDeviceSdCard size={25} stroke={1.7} /></span>
            <Text className={classes.brand}>{t('app.title')}</Text>
          </Group>
          <ThemePicker compact />
        </Group>
      </AppShell.Header>

      <AppShell.Navbar className={classes.navbar} p="md">
        <Text className={classes.sectionLabel}>{t('app.collection')}</Text>
        {NAV.map(({ key, to, icon: Icon }) => (
          <div key={key}>
            {key === 'import' && <Text className={classes.sectionLabel} mt={32}>{t('app.tools')}</Text>}
            <NavLink
              className={classes.navLink}
              label={t(`nav.${key}`)}
              leftSection={<Icon size={20} />}
              active={pathname === to || (key === 'library' && pathname.startsWith('/games/'))}
              onClick={close}
              renderRoot={(props) => <Link to={to} {...props} aria-current={pathname === to ? 'page' : undefined} />}
            />
          </div>
        ))}
      </AppShell.Navbar>

      <AppShell.Main className={classes.main}>
        <div id="main-content" tabIndex={-1} className={classes.content}>
          <Outlet />
        </div>
      </AppShell.Main>
    </AppShell>
  );
}
