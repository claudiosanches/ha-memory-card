import { MantineProvider } from '@mantine/core';
import { Notifications } from '@mantine/notifications';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from '@tanstack/react-router';
import { useState } from 'react';
import { createAppRouter } from './router';
import { theme } from './theme';
import './styles.css';

type Props = {
  router?: ReturnType<typeof createAppRouter>;
  queryClient?: QueryClient;
};

export function App({ router, queryClient: client }: Props) {
  const [queryClient] = useState(() => client ?? new QueryClient());
  const [appRouter] = useState(() => router ?? createAppRouter());

  return (
    <MantineProvider theme={theme} defaultColorScheme="auto">
      <Notifications position="top-right" />
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={appRouter} />
      </QueryClientProvider>
    </MantineProvider>
  );
}
