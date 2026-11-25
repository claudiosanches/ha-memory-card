import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { apiBaseUrl } from './client';

/** Queries that ask IGDB itself, which a refetch would ask again. */
const IGDB_LOOKUPS = new Set(['igdb-search', 'igdb-matches', 'igdb-contents']);

/**
 * Refetches what the page shows when the collection changes elsewhere, such
 * as a tag scan, another user or the IGDB sync. Who is signed in only changes
 * on restart, and IGDB lookups would ask IGDB again, so those stay. After the
 * stream drops and the browser reconnects, it refetches too, for the changes
 * made while it was away.
 */
export function useLiveUpdates() {
  const queryClient = useQueryClient();
  useEffect(() => {
    if (typeof EventSource === 'undefined') return;
    const events = new EventSource(`${apiBaseUrl()}/api/events`);
    let timer: number | undefined;
    let dropped = false;
    const refetch = () => {
      // Changes in a row, such as an import, refetch once.
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        void queryClient.invalidateQueries({
          predicate: ({ queryKey: [key] }) => key !== 'me' && !IGDB_LOOKUPS.has(String(key)),
        });
      }, 300);
    };
    events.addEventListener('changed', refetch);
    events.addEventListener('error', () => {
      dropped = true;
    });
    events.addEventListener('open', () => {
      if (dropped) refetch();
      dropped = false;
    });
    return () => {
      window.clearTimeout(timer);
      events.close();
    };
  }, [queryClient]);
}
