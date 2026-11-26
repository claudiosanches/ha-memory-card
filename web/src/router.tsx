import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
  type RouterHistory,
} from '@tanstack/react-router';
import { Layout } from './components/Layout';
import { ManagerOnly } from './components/ManagerOnly';
import { WishlistPage } from './pages/WishlistPage';
import { BacklogPage } from './pages/backlog/BacklogPage';
import { CLEANUP_TABS, CleanupPage, type CleanupTab } from './pages/cleanup/CleanupPage';
import { ImportPage } from './pages/ImportPage';
import { GamePage } from './pages/game/GamePage';
import { LibraryPage } from './pages/library/LibraryPage';
import { validateLibrarySearch } from './pages/library/search';
import { MANAGE_TABS, ManagePage, type ManageTab } from './pages/manage/ManagePage';
import { SETTINGS_TABS, SettingsPage, type SettingsTab } from './pages/SettingsPage';
import { IgdbMatchesPage } from './pages/IgdbMatchesPage';
import { StatsPage } from './pages/stats/StatsPage';
import { HouseholdPage } from './pages/household/HouseholdPage';
import { HardwarePage } from './pages/hardware/HardwarePage';
import { parseSearch, stringifySearch } from './searchParams';

const rootRoute = createRootRoute({ component: Layout });

const routeTree = rootRoute.addChildren([
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: LibraryPage,
    validateSearch: validateLibrarySearch,
  }),
  createRoute({ getParentRoute: () => rootRoute, path: '/games/$gameId', component: GamePage }),
  createRoute({ getParentRoute: () => rootRoute, path: '/wishlist', component: WishlistPage }),
  createRoute({ getParentRoute: () => rootRoute, path: '/hardware', component: HardwarePage }),
  createRoute({ getParentRoute: () => rootRoute, path: '/backlog', component: BacklogPage }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/cleanup',
    component: CleanupPage,
    validateSearch: (search: Record<string, unknown>): { tab?: CleanupTab } =>
      CLEANUP_TABS.includes(search.tab as CleanupTab) ? { tab: search.tab as CleanupTab } : {},
  }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/duplicates',
    beforeLoad: () => {
      throw redirect({ to: '/cleanup', search: { tab: 'duplicates' }, replace: true });
    },
  }),
  createRoute({ getParentRoute: () => rootRoute, path: '/stats', component: StatsPage }),
  createRoute({ getParentRoute: () => rootRoute, path: '/household', component: HouseholdPage }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/igdb-matches',
    component: () => (
      <ManagerOnly>
        <IgdbMatchesPage />
      </ManagerOnly>
    ),
  }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/setup',
    beforeLoad: () => {
      throw redirect({ to: '/cleanup', search: { tab: 'missing' }, replace: true });
    },
  }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/import',
    component: () => (
      <ManagerOnly>
        <ImportPage />
      </ManagerOnly>
    ),
  }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/manage',
    component: () => (
      <ManagerOnly>
        <ManagePage />
      </ManagerOnly>
    ),
    validateSearch: (search: Record<string, unknown>): { tab?: ManageTab } =>
      MANAGE_TABS.includes(search.tab as ManageTab) ? { tab: search.tab as ManageTab } : {},
  }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/settings',
    component: SettingsPage,
    beforeLoad: ({ location }) => {
      if (new URLSearchParams(location.searchStr).get('tab') === 'igdb') {
        throw redirect({ to: '/cleanup', search: { tab: 'igdb' }, replace: true });
      }
    },
    validateSearch: (search: Record<string, unknown>): { tab?: SettingsTab } =>
      SETTINGS_TABS.includes(search.tab as SettingsTab) ? { tab: search.tab as SettingsTab } : {},
  }),
]);

/** Hash history: the server never sees client routes, so deep links survive a reload under ingress. */
export function createAppRouter(history: RouterHistory = createHashHistory()) {
  return createRouter({ routeTree, history, parseSearch, stringifySearch });
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
