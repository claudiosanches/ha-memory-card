import { vi } from 'vitest';
import type { Me, PlayStatus } from '../api/types';

type Handler = (body: unknown) => unknown;
export type ApiCall = { method: string; path: string; query: URLSearchParams; body: unknown };

export function json(body: unknown, status = 200): Response {
  return new Response(status === 204 ? null : JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

const builtin = (key: string, color: PlayStatus['color'], counts_as: PlayStatus['counts_as']): PlayStatus => ({
  key,
  label: null,
  color,
  counts_as,
  builtin: true,
  game_count: 0,
});

/** The statuses a fresh install has. */
export const BUILTIN_STATUSES: PlayStatus[] = [
  builtin('backlog', 'gray', 'backlog'),
  builtin('playing', 'blue', 'playing'),
  builtin('paused', 'yellow', 'paused'),
  builtin('completed', 'green', 'finished'),
  builtin('dropped', 'red', 'finished'),
  builtin('mastered', 'grape', 'finished'),
];

/** A viewer once managers are configured; the default user is a manager. */
export const VIEWER: Me = {
  user: { id: 'viewer-id', name: 'sam', display_name: 'Sam' },
  role: 'viewer',
  enforced: true,
};

/**
 * Stubs fetch with routes keyed by "METHOD /path". A route is a value to
 * return as JSON, a Response, or a function of the request body (which may
 * return a promise of either). Unknown
 * routes answer 404, except the play status list, which defaults to the
 * built-in statuses, and the current user, who defaults to a manager.
 * Returns the list of calls made.
 */
export function mockApi(given: Record<string, unknown>): ApiCall[] {
  const routes: Record<string, unknown> = {
    'GET /api/play-statuses': BUILTIN_STATUSES,
    'GET /api/me': { user: null, role: 'manager', enforced: false } satisfies Me,
    ...given,
  };
  const calls: ApiCall[] = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (request: Request) => {
      const url = new URL(request.url);
      const path = url.pathname;
      const text = await request.clone().text();
      const isJson = request.headers.get('Content-Type')?.includes('json') ?? false;
      const body: unknown = text ? (isJson ? JSON.parse(text) : text) : undefined;
      calls.push({ method: request.method, path, query: url.searchParams, body });

      const route = routes[`${request.method} ${path}`];
      if (route === undefined) return json({ code: 'not_found' }, 404);
      const result = await (typeof route === 'function' ? (route as Handler)(body) : route);
      return result instanceof Response ? result : json(result);
    }),
  );
  return calls;
}
