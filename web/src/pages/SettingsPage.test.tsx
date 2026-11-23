import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { TitleDifference } from '../api/types';
import en from '../locales/en.json';
import ptBR from '../locales/pt-BR.json';
import { MANAGER, mockApi, VIEWER } from '../test/api';
import { renderAt } from '../test/render';

const DIFFERENCES: TitleDifference[] = [
  { game_id: 3, title: 'Bomberman 93', igdb_title: "Bomberman '93", cover_path: null },
  { game_id: 8, title: 'de Blob 2', igdb_title: 'De Blob 2', cover_path: null },
];

const STATUS = {
  version: '0.1.0',
  igdb_configured: true,
  external_services: true,
  sensors: 'connected',
  tags: 'unavailable',
};

describe('Settings access', () => {
  it('shows the user ID to add to the managers option', async () => {
    mockApi({
      'GET /api/me': { ...MANAGER, user: { id: 'abc123', name: 'sam', display_name: null } },
      'GET /api/status': STATUS,
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': [],
    });
    renderAt('/settings');

    expect(await screen.findByTestId('user-id')).toHaveTextContent('abc123');
    expect(screen.getByText('sam')).toBeInTheDocument();
    expect(screen.getByText(en.settings.accessManager)).toBeInTheDocument();
    fireEvent.click(await screen.findByRole('tab', { name: en.settings.igdb }));
    expect(await screen.findByRole('button', { name: en.igdb.syncButton })).toBeInTheDocument();
  });

  it('explains how to name the first manager', async () => {
    mockApi({ 'GET /api/me': { ...VIEWER, managers_configured: false }, 'GET /api/status': STATUS });
    renderAt('/settings');

    expect(await screen.findByText(en.settings.accessNoManagers)).toBeInTheDocument();
    expect(screen.getByTestId('no-managers')).toHaveTextContent(en.app.noManagers);
    expect(screen.queryByRole('link', { name: en.app.openSettings })).not.toBeInTheDocument();
  });

  it('keeps IGDB tools away from viewers', async () => {
    const calls = mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': STATUS });
    renderAt('/settings');

    expect(await screen.findByTestId('status-version')).toBeInTheDocument();
    expect(screen.getByTestId('access-role')).toHaveTextContent(en.settings.roles.viewer);
    expect(screen.getByText(en.settings.accessViewer)).toBeInTheDocument();
    expect(screen.queryByRole('tab', { name: en.settings.igdb })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: en.igdb.syncButton })).not.toBeInTheDocument();
    expect(calls.some((c) => c.path.startsWith('/api/igdb'))).toBe(false);
  });
});

describe('Settings about', () => {
  it('links both ways to support development', async () => {
    mockApi({});
    renderAt('/settings');
    expect(await screen.findByRole('link', { name: en.settings.githubSponsors })).toHaveAttribute(
      'href',
      'https://github.com/sponsors/claudiosanches',
    );
    expect(screen.getByRole('link', { name: en.settings.kofi })).toHaveAttribute('href', 'https://ko-fi.com/claudiosanches');
  });
});

describe('Settings sensors', () => {
  it('shows whether Home Assistant sensors are published', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': STATUS });
    renderAt('/settings');
    expect(await screen.findByTestId('sensors-status')).toHaveTextContent(en.settings.sensorsStatus.connected);
  });

  it('says what to install when there is no broker', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': { ...STATUS, sensors: 'no_broker' } });
    renderAt('/settings');
    expect(await screen.findByTestId('sensors-status')).toHaveTextContent(en.settings.sensorsStatus.no_broker);
  });

  it('says the integration still has to be added', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': { ...STATUS, sensors: 'waiting' } });
    renderAt('/settings');
    expect(await screen.findByTestId('sensors-status')).toHaveTextContent(en.settings.sensorsStatus.waiting);
  });
});

describe('Settings titles', () => {
  it('renames a game to its IGDB title', async () => {
    const calls = mockApi({
      'GET /api/status': STATUS,
      'GET /api/igdb/sync': { running: false, total: 0, done: 0, linked: 0, unmatched: 0, updated: 0, failed: 0 },
      'GET /api/igdb/titles': DIFFERENCES,
      'POST /api/igdb/titles': [DIFFERENCES[1]],
    });
    renderAt('/settings?tab=igdb');

    const row = await screen.findByTestId('title-3');
    expect(within(row).getByText("IGDB: Bomberman '93")).toBeInTheDocument();
    expect(screen.getByText(en.igdb.titlesText_other.split('{{count}}').join('2'))).toBeInTheDocument();
    fireEvent.click(within(row).getByRole('button', { name: en.igdb.useIgdbTitle }));

    await waitFor(() => expect(screen.queryByTestId('title-3')).toBeNull());
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ use_igdb: [3] });
    expect(screen.getByTestId('title-8')).toBeInTheDocument();
  });
});

describe('Settings language', () => {
  it('switches the app to Portuguese and back to the browser language', async () => {
    mockApi({ 'GET /api/status': STATUS });
    renderAt('/settings?tab=appearance');

    fireEvent.click(await screen.findByRole('combobox', { name: en.settings.language }));
    fireEvent.click(await screen.findByRole('option', { name: 'Português (Brasil)', hidden: true }));
    expect(await screen.findByRole('link', { name: ptBR.nav.library })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe('pt-BR');
    expect(localStorage.getItem('memory-card-language')).toBe('pt-BR');

    fireEvent.click(screen.getByRole('combobox', { name: ptBR.settings.language }));
    fireEvent.click(await screen.findByRole('option', { name: 'Automático (English)', hidden: true }));
    expect(await screen.findByRole('link', { name: en.nav.library })).toBeInTheDocument();
    expect(localStorage.getItem('memory-card-language')).toBeNull();
  });
});
