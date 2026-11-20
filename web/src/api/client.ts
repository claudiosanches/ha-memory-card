import createClient from 'openapi-fetch';
import type { paths } from './schema';

/**
 * API paths in the spec start with `/api/`. Resolving the base against the
 * document keeps every request under the ingress prefix.
 */
export function apiBaseUrl(baseURI: string = document.baseURI): string {
  return new URL('.', baseURI).href.replace(/\/$/, '');
}

export const api = createClient<paths>({
  baseUrl: apiBaseUrl(),
  // Late-bound so tests can stub fetch.
  fetch: (request) => globalThis.fetch(request),
});
