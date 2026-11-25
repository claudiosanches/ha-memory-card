import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { apiBaseUrl } from './client';

/** Queries that ask IGDB itself, which a refetch would ask again. */
const IGDB_LOOKUPS = new Set(['igdb-search', 'igdb-matches', 'igdb-contents']);

/**
 * Refetches what the page shows when the collection changes elsewhere, such
 * as a tag scan, another user or the IGDB sync. IGDB lookups stay. After the
 * stream drops and the browser reconnects, it refetches too, for the changes
 * made while it was away, and who is signed in, since a restart may have
 * changed the roles.
 */
export function useLiveUpdates() {
  const queryClient = useQueryClient();
  useEffect(() => {
    if (typeof EventSource === 'undefined') return;
    const events = new EventSource(`${apiBaseUrl()}/api/events`);
    let timer: number | undefined;
    let dropped = false;
    let withMe = false;
    const refetch = (reconnected: boolean) => {
      withMe ||= reconnected;
      // Changes in a row, such as an import, refetch once.
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        const me = withMe;
        withMe = false;
        void queryClient.invalidateQueries({
          predicate: ({ queryKey: [key] }) => (me || key !== 'me') && !IGDB_LOOKUPS.has(String(key)),
        });
      }, 300);
    };
    events.addEventListener('changed', () => refetch(false));
    events.addEventListener('error', () => {
      dropped = true;
    });
    events.addEventListener('open', () => {
      if (dropped) refetch(true);
      dropped = false;
    });
    return () => {
      window.clearTimeout(timer);
      events.close();
    };
  }, [queryClient]);
}
