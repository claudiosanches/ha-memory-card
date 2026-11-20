import { AppShell, Burger, Group, NavLink, Title } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import {
  IconCopy,
  IconDeviceGamepad2,
  IconFileImport,
  IconLibrary,
  IconSettings,
} from '@tabler/icons-react';
import { Link, Outlet, useRouterState } from '@tanstack/react-router';
import type { ComponentType } from 'react';
import { useTranslation } from 'react-i18next';

type NavKey = 'library' | 'duplicates' | 'import' | 'manage' | 'settings';

const NAV: { key: NavKey; to: string; icon: ComponentType<{ size?: number }> }[] = [
  { key: 'library', to: '/', icon: IconLibrary },
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
      header={{ height: 56 }}
      navbar={{ width: 240, breakpoint: 'sm', collapsed: { mobile: !opened } }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md">
          <Burger
            opened={opened}
            onClick={toggle}
            hiddenFrom="sm"
            size="sm"
            aria-label={t('app.toggleNavigation')}
          />
          <Title order={3}>{t('app.title')}</Title>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="xs">
        {NAV.map(({ key, to, icon: Icon }) => (
          <NavLink
            key={key}
            label={t(`nav.${key}`)}
            leftSection={<Icon size={18} />}
            active={pathname === to}
            onClick={close}
            renderRoot={(props) => <Link to={to} {...props} />}
          />
        ))}
      </AppShell.Navbar>

      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
}
