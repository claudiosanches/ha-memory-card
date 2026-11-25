import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import en from '../locales/en.json';
import ptBR from '../locales/pt-BR.json';
import { MANAGER, mockApi, VIEWER } from '../test/api';
import { renderAt } from '../test/render';

const STATUS = {
  version: '0.1.0',
  igdb_configured: true,
  external_services: true,
  sensors: 'connected',
  tags: 'unavailable',
};

describe('Settings defaults', () => {
  it('opens Appearance first once a manager is configured', async () => {
    mockApi({ 'GET /api/me': MANAGER, 'GET /api/status': STATUS });
    renderAt('/settings');

    expect(await screen.findByRole('tab', { name: en.settings.appearance })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('combobox', { name: en.settings.theme })).toBeInTheDocument();
  });

  it('opens Status first when no manager is configured yet', async () => {
    mockApi({ 'GET /api/me': { ...VIEWER, managers_configured: false }, 'GET /api/status': STATUS });
    renderAt('/settings');

    expect(await screen.findByText(en.settings.accessNoManagers)).toBeInTheDocument();
    expect(screen.getByTestId('no-managers')).toHaveTextContent(en.app.noManagers);
    expect(screen.queryByRole('link', { name: en.app.openSettings })).not.toBeInTheDocument();
  });
});

describe('Settings access', () => {
  it('shows the user ID to add to the managers option', async () => {
    mockApi({
      'GET /api/me': { ...MANAGER, user: { id: 'abc123', name: 'sam', display_name: null } },
      'GET /api/status': STATUS,
    });
    renderAt('/settings?tab=status');

    expect(await screen.findByTestId('user-id')).toHaveTextContent('abc123');
    expect(screen.getByText('sam')).toBeInTheDocument();
    expect(screen.getByText(en.settings.accessManager)).toBeInTheDocument();
  });

  it('shows viewer role on the status tab', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': STATUS });
    renderAt('/settings?tab=status');

    expect(await screen.findByTestId('status-version')).toBeInTheDocument();
    expect(screen.getByTestId('access-role')).toHaveTextContent(en.settings.roles.viewer);
    expect(screen.getByText(en.settings.accessViewer)).toBeInTheDocument();
  });
});

describe('Settings about', () => {
  it('links to documentation and the support issue form', async () => {
    mockApi({});
    renderAt('/settings?tab=status');
    expect(await screen.findByRole('link', { name: en.settings.documentation })).toHaveAttribute(
      'href',
      'https://ha-memory-card.pages.dev/docs/',
    );
    expect(screen.getByRole('link', { name: en.settings.getSupport })).toHaveAttribute(
      'href',
      'https://github.com/claudiosanches/ha-memory-card/issues/new?template=support.yml',
    );
  });

  it('links both ways to support development', async () => {
    mockApi({});
    renderAt('/settings?tab=status');
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
    renderAt('/settings?tab=status');
    expect(await screen.findByTestId('sensors-status')).toHaveTextContent(en.settings.sensorsStatus.connected);
  });

  it('says what to install when there is no broker', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': { ...STATUS, sensors: 'no_broker' } });
    renderAt('/settings?tab=status');
    expect(await screen.findByTestId('sensors-status')).toHaveTextContent(en.settings.sensorsStatus.no_broker);
  });

  it('says the integration still has to be added', async () => {
    mockApi({ 'GET /api/me': VIEWER, 'GET /api/status': { ...STATUS, sensors: 'waiting' } });
    renderAt('/settings?tab=status');
    expect(await screen.findByTestId('sensors-status')).toHaveTextContent(en.settings.sensorsStatus.waiting);
  });
});

describe('Settings language', () => {
  it('switches the app to Portuguese and back to the browser language', async () => {
    mockApi({ 'GET /api/status': STATUS });
    renderAt('/settings');

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
