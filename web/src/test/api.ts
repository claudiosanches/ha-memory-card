import { vi } from 'vitest';

type Handler = (body: unknown) => unknown;
export type ApiCall = { method: string; path: string; body: unknown };

export function json(body: unknown, status = 200): Response {
  return new Response(status === 204 ? null : JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

/**
 * Stubs fetch with routes keyed by "METHOD /path". A route is a value to
 * return as JSON, a Response, or a function of the request body. Unknown
 * routes answer 404. Returns the list of calls made.
 */
export function mockApi(routes: Record<string, unknown>): ApiCall[] {
  const calls: ApiCall[] = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (request: Request) => {
      const path = new URL(request.url).pathname;
      const text = await request.clone().text();
      const body: unknown = text ? JSON.parse(text) : undefined;
      calls.push({ method: request.method, path, body });

      const route = routes[`${request.method} ${path}`];
      if (route === undefined) return json({ code: 'not_found' }, 404);
      const result = typeof route === 'function' ? (route as Handler)(body) : route;
      return result instanceof Response ? result : json(result);
    }),
  );
  return calls;
}
