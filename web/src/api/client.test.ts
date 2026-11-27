import { afterEach, describe, expect, it, vi } from 'vitest';
import { api, apiBaseUrl } from './client';

describe('api', () => {
  afterEach(() => vi.restoreAllMocks());

  it('marks requests as coming from the app', async () => {
    const fetch = vi.spyOn(globalThis, 'fetch').mockResolvedValue(new Response(null, { status: 204 }));
    await api.POST('/api/igdb/sync');
    const request = fetch.mock.calls[0]?.[0] as Request;
    expect(request.headers.get('X-Requested-With')).toBe('memory-card');
  });
});

describe('apiBaseUrl', () => {
  it('keeps the ingress prefix', () => {
    expect(apiBaseUrl('http://ha.local:8123/api/hassio_ingress/abc/')).toBe(
      'http://ha.local:8123/api/hassio_ingress/abc',
    );
  });

  it('ignores the hash route', () => {
    expect(apiBaseUrl('http://ha.local/api/hassio_ingress/abc/#/settings')).toBe(
      'http://ha.local/api/hassio_ingress/abc',
    );
  });

  it('works at the root in development', () => {
    expect(apiBaseUrl('http://localhost:5173/')).toBe('http://localhost:5173');
  });
});
