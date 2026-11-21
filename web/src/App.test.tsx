import { fireEvent, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import en from './locales/en.json';
import { json, mockApi, VIEWER } from './test/api';
import { renderAt } from './test/render';

describe('App', () => {
  afterEach(() => localStorage.clear());

  it('shows every navigation entry from the locale file', async () => {
    mockApi({ 'GET /api/games': [] });
    renderAt('/');
    for (const label of Object.values(en.nav)) {
      expect(await screen.findByRole('link', { name: label })).toBeInTheDocument();
    }
    expect(screen.getByRole('heading', { name: en.nav.library })).toBeInTheDocument();
  });

  it('collapses the sidebar to icons and remembers it', async () => {
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: query.includes('min-width'),
      media: query,
      addEventListener: () => {},
      removeEventListener: () => {},
    }));
    mockApi({ 'GET /api/games': [] });
    renderAt('/');

    fireEvent.click(await screen.findByRole('button', { name: en.app.collapseSidebar }));
    expect(screen.getByRole('button', { name: en.app.expandSidebar })).toHaveAttribute('aria-expanded', 'false');
    expect(screen.getByRole('navigation')).toHaveAttribute('data-rail');
    expect(screen.getByRole('link', { name: en.nav.library })).toBeInTheDocument();
    expect(localStorage.getItem('memory-card-sidebar-collapsed')).toBe('true');
  });

  it('leaves manager tools out of a viewer navigation', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/games': [] });
    renderAt('/');
    expect(await screen.findByRole('link', { name: en.nav.settings })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: en.nav.import })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: en.nav.manage })).not.toBeInTheDocument();
  });

  it('keeps viewers off manager pages opened by URL', async () => {
    mockApi({ 'GET /api/me': VIEWER });
    renderAt('/manage');
    expect(await screen.findByText(en.app.managersOnly)).toBeInTheDocument();
    expect(screen.queryByRole('tab')).not.toBeInTheDocument();
  });

  it('explains a missing Home Assistant identity', async () => {
    mockApi({ 'GET /api/me': json({ code: 'identity_required' }, 401) });
    renderAt('/');
    expect(await screen.findByRole('alert')).toHaveTextContent(en.app.unidentified);
    expect(screen.queryByRole('link', { name: en.nav.library })).not.toBeInTheDocument();
  });

  it('shows server status on the settings screen', async () => {
    mockApi({ 'GET /api/status': { version: '9.9.9', igdb_configured: false, games: 3, owned_copies: 5 } });
    renderAt('/settings');
    expect(await screen.findByTestId('status-version')).toHaveTextContent('9.9.9');
    expect(screen.getByTestId('igdb-missing')).toHaveTextContent(en.settings.igdbMissing);
  });

  it('reports a failed status request', async () => {
    mockApi({ 'GET /api/status': json({ code: 'internal' }, 500) });
    renderAt('/settings');
    expect(await screen.findByTestId('status-error')).toHaveTextContent(en.settings.loadError);
  });
});
