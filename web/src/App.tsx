import { Notifications } from '@mantine/notifications';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useState } from 'react';
import { createAppRouter } from './router';
import { AppearanceProvider } from './styles/themes/AppearanceProvider';
import './styles/base.css';

type Props = {
  router?: ReturnType<typeof createAppRouter>;
  queryClient?: QueryClient;
};

export function App({ router, queryClient: client }: Props) {
  const [queryClient] = useState(() => client ?? new QueryClient());
  const [appRouter] = useState(() => router ?? createAppRouter());

  return (
    <AppearanceProvider>
      <Notifications position="top-right" />
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={appRouter} />
      </QueryClientProvider>
    </AppearanceProvider>
  );
}
