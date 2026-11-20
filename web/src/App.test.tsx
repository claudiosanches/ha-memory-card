import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import en from './locales/en.json';
import { renderAt } from './test/render';

function stubStatus(body: unknown, status = 200) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => new Response(JSON.stringify(body), {
      status,
      headers: { 'Content-Type': 'application/json' },
    })),
  );
}

describe('App', () => {
  it('shows every navigation entry from the locale file', async () => {
    renderAt('/');
    for (const label of Object.values(en.nav)) {
      expect(await screen.findByRole('link', { name: label })).toBeInTheDocument();
    }
    expect(screen.getByRole('heading', { name: en.nav.library })).toBeInTheDocument();
  });

  it('shows server status on the settings screen', async () => {
    stubStatus({ version: '9.9.9', igdb_configured: false, games: 3, owned_copies: 5 });
    renderAt('/settings');
    expect(await screen.findByTestId('status-version')).toHaveTextContent('9.9.9');
    expect(screen.getByTestId('igdb-missing')).toHaveTextContent(en.settings.igdbMissing);
  });

  it('reports a failed status request', async () => {
    stubStatus({ code: 'internal' }, 500);
    renderAt('/settings');
    expect(await screen.findByTestId('status-error')).toHaveTextContent(en.settings.loadError);
  });
});
