import { AppShell, Burger, Group, Loader, NavLink, Text } from '@mantine/core';
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
import { ApiError, errorMessage } from '../api/errors';
import { useMe } from '../api/queries';
import { ThemePicker } from './ThemePicker';
import classes from './Layout.module.css';

type NavKey = 'library' | 'backlog' | 'duplicates' | 'import' | 'manage' | 'settings';
type NavItem = { key: NavKey; to: string; icon: ComponentType<{ size?: number }>; manager?: boolean };

const COLLECTION: NavItem[] = [
  { key: 'library', to: '/', icon: IconLibrary },
  { key: 'backlog', to: '/backlog', icon: IconStack2 },
  { key: 'duplicates', to: '/duplicates', icon: IconCopy },
];

const TOOLS: NavItem[] = [
  { key: 'import', to: '/import', icon: IconFileImport, manager: true },
  { key: 'manage', to: '/manage', icon: IconDeviceGamepad2, manager: true },
  { key: 'settings', to: '/settings', icon: IconSettings },
];

export function Layout() {
  const { t } = useTranslation();
  const [opened, { toggle, close }] = useDisclosure();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const me = useMe();
  const canManage = me.data?.role === 'manager';

  const link = ({ key, to, icon: Icon }: NavItem) => (
    <NavLink
      key={key}
      className={classes.navLink}
      label={t(`nav.${key}`)}
      leftSection={<Icon size={20} />}
      active={pathname === to || (key === 'library' && pathname.startsWith('/games/'))}
      onClick={close}
      renderRoot={(props) => <Link to={to} {...props} aria-current={pathname === to ? 'page' : undefined} />}
    />
  );

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
        {me.data && (
          <>
            <Text className={classes.sectionLabel}>{t('app.collection')}</Text>
            {COLLECTION.map(link)}
            <Text className={classes.sectionLabel} mt={32}>{t('app.tools')}</Text>
            {TOOLS.filter((item) => canManage || !item.manager).map(link)}
          </>
        )}
      </AppShell.Navbar>

      <AppShell.Main className={classes.main}>
        <div id="main-content" tabIndex={-1} className={classes.content}>
          {/* Pages read the role, so they wait for it. */}
          {me.data ? (
            <Outlet />
          ) : me.isError ? (
            <Text c="red" role="alert">
              {me.error instanceof ApiError && me.error.code === 'identity_required'
                ? t('app.unidentified')
                : errorMessage(t, me.error)}
            </Text>
          ) : (
            <Loader size="sm" />
          )}
        </div>
      </AppShell.Main>
    </AppShell>
  );
}
