import { ActionIcon, AppShell, Burger, Group, Loader, NavLink, Text, Tooltip } from '@mantine/core';
import { useDisclosure, useLocalStorage, useMediaQuery } from '@mantine/hooks';
import {
  IconChecklist,
  IconCopy,
  IconStack2,
  IconDeviceGamepad2,
  IconFileImport,
  IconLibrary,
  IconSettings,
  IconDeviceSdCard,
  IconLayoutSidebarLeftCollapse,
  IconLayoutSidebarLeftExpand,
} from '@tabler/icons-react';
import { Link, Outlet, useRouterState } from '@tanstack/react-router';
import type { ComponentType } from 'react';
import { useTranslation } from 'react-i18next';
import { ApiError, errorMessage } from '../api/errors';
import { useMe } from '../api/queries';
import { ThemePicker } from './ThemePicker';
import classes from './Layout.module.css';

type NavKey = 'library' | 'backlog' | 'duplicates' | 'setup' | 'import' | 'manage' | 'settings';
type NavItem = { key: NavKey; to: string; icon: ComponentType<{ size?: number }>; manager?: boolean };

const COLLECTION: NavItem[] = [
  { key: 'library', to: '/', icon: IconLibrary },
  { key: 'backlog', to: '/backlog', icon: IconStack2 },
  { key: 'duplicates', to: '/duplicates', icon: IconCopy },
];

const TOOLS: NavItem[] = [
  { key: 'setup', to: '/setup', icon: IconChecklist, manager: true },
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
  const [collapsed, setCollapsed] = useLocalStorage({ key: 'memory-card-sidebar-collapsed', defaultValue: false });
  // Phones open the full menu from the burger, so only wider screens get the icon rail.
  const wide = useMediaQuery('(min-width: 48em)');
  const rail = collapsed && wide;

  const link = ({ key, to, icon: Icon }: NavItem) => (
    <Tooltip key={key} label={t(`nav.${key}`)} position="right" disabled={!rail}>
      <NavLink
        className={classes.navLink}
        classNames={{ body: classes.navBody, section: classes.navSection }}
        label={t(`nav.${key}`)}
        leftSection={<Icon size={20} />}
        active={pathname === to || (key === 'library' && pathname.startsWith('/games/'))}
        onClick={close}
        renderRoot={(props) => <Link to={to} {...props} aria-current={pathname === to ? 'page' : undefined} />}
      />
    </Tooltip>
  );

  return (
    <AppShell
      header={{ height: 68 }}
      navbar={{ width: rail ? 80 : 224, breakpoint: 'sm', collapsed: { mobile: !opened } }}
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
            <ActionIcon
              visibleFrom="sm"
              variant="subtle"
              color="gray"
              size="lg"
              aria-label={collapsed ? t('app.expandSidebar') : t('app.collapseSidebar')}
              aria-expanded={!collapsed}
              onClick={() => setCollapsed((value) => !value)}
            >
              {collapsed ? <IconLayoutSidebarLeftExpand size={20} /> : <IconLayoutSidebarLeftCollapse size={20} />}
            </ActionIcon>
            <span className={classes.logo}><IconDeviceSdCard size={25} stroke={1.7} /></span>
            <Text className={classes.brand}>{t('app.title')}</Text>
          </Group>
          <ThemePicker compact />
        </Group>
      </AppShell.Header>

      <AppShell.Navbar className={classes.navbar} p="md" data-rail={rail || undefined}>
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
