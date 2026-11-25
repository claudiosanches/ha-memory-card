import { screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import en from '../locales/en.json';
import { mockApi } from '../test/api';
import { renderAt } from '../test/render';

class FakeEventSource {
  static last: FakeEventSource | undefined;
  url: string;
  listeners = new Map<string, () => void>();
  closed = false;
  constructor(url: string) {
    this.url = url;
    FakeEventSource.last = this;
  }
  addEventListener(type: string, listener: () => void) {
    this.listeners.set(type, listener);
  }
  close() {
    this.closed = true;
  }
}

describe('Live updates', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('refetches the page when the collection changes elsewhere', async () => {
    vi.stubGlobal('EventSource', FakeEventSource);
    const calls = mockApi({ 'GET /api/games': [], 'GET /api/platforms': [] });
    renderAt('/');
    await screen.findByText(en.library.empty);
    const gamesLoaded = () => calls.filter((c) => c.path === '/api/games').length;
    const before = gamesLoaded();

    expect(FakeEventSource.last?.url).toMatch(/\/api\/events$/);
    FakeEventSource.last?.listeners.get('changed')?.();
    await waitFor(() => expect(gamesLoaded()).toBeGreaterThan(before));
    expect(calls.filter((c) => c.path === '/api/me')).toHaveLength(1);
  });

  it('refetches after the stream reconnects, but not when it first opens', async () => {
    vi.stubGlobal('EventSource', FakeEventSource);
    const calls = mockApi({ 'GET /api/games': [], 'GET /api/platforms': [] });
    renderAt('/');
    await screen.findByText(en.library.empty);
    const gamesLoaded = () => calls.filter((c) => c.path === '/api/games').length;
    const before = gamesLoaded();

    const events = FakeEventSource.last;
    events?.listeners.get('open')?.();
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(gamesLoaded()).toBe(before);

    events?.listeners.get('error')?.();
    events?.listeners.get('open')?.();
    await waitFor(() => expect(gamesLoaded()).toBeGreaterThan(before));
    expect(calls.filter((c) => c.path === '/api/me')).toHaveLength(2);
  });

  it('refetches the IGDB title differences', async () => {
    vi.stubGlobal('EventSource', FakeEventSource);
    const calls = mockApi({
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true, sensors: 'off' },
      'GET /api/igdb/titles': [],
      'GET /api/igdb/sync': { running: false, done: 0, total: 0, linked: 0, updated: 0, unmatched: 0, failed: 0 },
    });
    renderAt('/cleanup?tab=igdb');
    await screen.findByText(en.igdb.titlesNone);
    const loaded = (path: string) => calls.filter((c) => c.path === path).length;
    const titles = loaded('/api/igdb/titles');

    FakeEventSource.last?.listeners.get('changed')?.();
    await waitFor(() => expect(loaded('/api/igdb/titles')).toBeGreaterThan(titles));
  });
});
