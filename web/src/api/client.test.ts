import { describe, expect, it } from 'vitest';
import { apiBaseUrl } from './client';

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
