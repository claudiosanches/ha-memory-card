import createClient from 'openapi-fetch';
import type { paths } from './schema';

/**
 * API paths in the spec start with `/api/`. Resolving the base against the
 * document keeps every request under the ingress prefix.
 */
export function apiBaseUrl(baseURI: string = document.baseURI): string {
  return new URL('.', baseURI).href.replace(/\/$/, '');
}

/** True when served through Home Assistant ingress (`<base href>` injected). */
export function inIngress(baseURI: string = document.baseURI): boolean {
  return new URL('.', baseURI).pathname !== '/';
}

export const api = createClient<paths>({
  baseUrl: apiBaseUrl(),
  // The server refuses changes without it; another site cannot send it.
  headers: { 'X-Requested-With': 'memory-card' },
  // Late-bound so tests can stub fetch.
  fetch: (request) => globalThis.fetch(request),
});
