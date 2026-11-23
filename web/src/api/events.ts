import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { apiBaseUrl } from './client';

/**
 * Refetches what the page shows when the collection changes elsewhere, such
 * as a tag scan, another user or the IGDB sync. Who is signed in only changes
 * on restart, and IGDB lookups would ask IGDB again, so those stay.
 */
export function useLiveUpdates() {
  const queryClient = useQueryClient();
  useEffect(() => {
    if (typeof EventSource === 'undefined') return;
    const events = new EventSource(`${apiBaseUrl()}/api/events`);
    let timer: number | undefined;
    events.addEventListener('changed', () => {
      // Changes in a row, such as an import, refetch once.
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        void queryClient.invalidateQueries({
          predicate: ({ queryKey: [key] }) => key !== 'me' && !(typeof key === 'string' && key.startsWith('igdb')),
        });
      }, 300);
    });
    return () => {
      window.clearTimeout(timer);
      events.close();
    };
  }, [queryClient]);
}
