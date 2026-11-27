import { afterEach, describe, expect, it, vi } from 'vitest';
import { api, apiBaseUrl, inIngress } from './client';

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

describe('inIngress', () => {
  it('is true when served under a path prefix', () => {
    expect(inIngress('http://ha.local:8123/api/hassio_ingress/abc/')).toBe(true);
    expect(inIngress('http://ha.local/api/hassio_ingress/abc/#/manage')).toBe(true);
  });

  it('is false at the root', () => {
    expect(inIngress('http://127.0.0.1:8099/')).toBe(false);
    expect(inIngress('http://127.0.0.1:8099/#/manage')).toBe(false);
    expect(inIngress('http://localhost:5173/')).toBe(false);
  });
});
