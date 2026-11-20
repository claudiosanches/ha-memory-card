import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  type RouterHistory,
} from '@tanstack/react-router';
import { Layout } from './components/Layout';
import { DuplicatesPage } from './pages/DuplicatesPage';
import { ImportPage } from './pages/ImportPage';
import { LibraryPage } from './pages/LibraryPage';
import { MANAGE_TABS, ManagePage, type ManageTab } from './pages/manage/ManagePage';
import { SettingsPage } from './pages/SettingsPage';

const rootRoute = createRootRoute({ component: Layout });

const routeTree = rootRoute.addChildren([
  createRoute({ getParentRoute: () => rootRoute, path: '/', component: LibraryPage }),
  createRoute({ getParentRoute: () => rootRoute, path: '/duplicates', component: DuplicatesPage }),
  createRoute({ getParentRoute: () => rootRoute, path: '/import', component: ImportPage }),
  createRoute({
    getParentRoute: () => rootRoute,
    path: '/manage',
    component: ManagePage,
    validateSearch: (search: Record<string, unknown>): { tab?: ManageTab } =>
      MANAGE_TABS.includes(search.tab as ManageTab) ? { tab: search.tab as ManageTab } : {},
  }),
  createRoute({ getParentRoute: () => rootRoute, path: '/settings', component: SettingsPage }),
]);

/** Hash history: the server never sees client routes, so deep links survive a reload under ingress. */
export function createAppRouter(history: RouterHistory = createHashHistory()) {
  return createRouter({ routeTree, history });
}

declare module '@tanstack/react-router' {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
