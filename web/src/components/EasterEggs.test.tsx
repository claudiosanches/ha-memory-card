import { fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { GameSummary } from '../api/types';
import en from '../locales/en.json';
import { mockApi } from '../test/api';
import { renderAt } from '../test/render';

const STATUS = {
  version: '0.4.2',
  igdb_configured: false,
  external_services: true,
  sensors: 'connected',
  tags: 'unavailable',
};

const GAMES: GameSummary[] = [
  {
    id: 42,
    title: 'Metal Gear Solid',
    cover_path: null,
    play_status: 'finished',
    last_played_at: null,
    platforms: [{ platform_id: 1, copies: 1 }],
    owned_copies: 1,
    wanted_copies: 0,
  },
];

describe('Slot 1 Memory Card browser', () => {
  afterEach(() => localStorage.clear());

  it('opens after 5 clicks on the header logo and navigates to a game', async () => {
    mockApi({
      'GET /api/games': GAMES,
      'GET /api/platforms': [
        { id: 1, name: 'PlayStation', abbreviation: 'PS1', slug: 'ps1', active: true, sort_order: 1, game_count: 1, owned_game_count: 1, wanted_game_count: 0, device_count: 0, account_count: 0 },
      ],
    });
    renderAt('/');

    await screen.findByRole('heading', { name: en.nav.library });
    const logo = screen.getByTestId('app-logo');
    for (let i = 0; i < 4; i++) fireEvent.click(logo);
    expect(screen.queryByTestId('slot-one-modal')).not.toBeInTheDocument();

    fireEvent.click(logo);
    const modal = await screen.findByTestId('slot-one-modal');
    expect(within(modal).getByText('1/15 blocks used')).toBeInTheDocument();

    const block = within(modal).getByRole('listitem', { name: 'Metal Gear Solid' });
    fireEvent.mouseEnter(block);
    expect(within(modal).getByText('Metal Gear Solid')).toBeInTheDocument();
    expect(within(modal).getByText('PS1')).toBeInTheDocument();
  });
});

describe('CRT diagnostic terminal and hidden Cursed 90s theme', () => {
  afterEach(() => localStorage.clear());

  it('hides Cursed 90s from the theme picker until unlocked via the CRT terminal', async () => {
    mockApi({ 'GET /api/status': STATUS });
    renderAt('/settings');

    // Theme picker does not list Cursed 90's initially.
    fireEvent.click(await screen.findByRole('combobox', { name: en.settings.theme }));
    expect(screen.queryByRole('option', { name: en.settings.themes['cursed-90s'], hidden: true })).not.toBeInTheDocument();

    // Switch to Status and click Version 3 times to open the CRT terminal.
    fireEvent.click(screen.getByRole('tab', { name: en.settings.status }));
    const version = await screen.findByTestId('status-version');
    fireEvent.click(version);
    fireEvent.click(version);
    expect(screen.queryByTestId('diagnostic-terminal')).not.toBeInTheDocument();
    fireEvent.click(version);

    const terminal = await screen.findByTestId('diagnostic-terminal');
    // Wrong guess shows error and hint.
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    expect(within(terminal).getByTestId('terminal-log')).toHaveTextContent(en.easterEggs.crt.hint1);

    // Unrelated key does not clear valid progress.
    fireEvent.keyDown(window, { key: 'ArrowUp' });
    fireEvent.keyDown(window, { key: 'x' });
    expect(within(terminal).getByTestId('code-slot-0')).toHaveTextContent('↑');

    // Finish the secret sequence via keyboard + on-screen buttons.
    fireEvent.keyDown(window, { key: 'ArrowUp' });
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    fireEvent.keyDown(window, { key: 'ArrowLeft' });
    fireEvent.keyDown(window, { key: 'ArrowRight' });
    fireEvent.keyDown(window, { key: 'ArrowLeft' });
    fireEvent.keyDown(window, { key: 'ArrowRight' });
    fireEvent.click(within(terminal).getByRole('button', { name: 'B' }));
    fireEvent.click(within(terminal).getByRole('button', { name: 'A' }));

    expect(within(terminal).getByTestId('terminal-log')).toHaveTextContent(en.easterEggs.crt.accepted);
    expect(document.documentElement.dataset.mcTheme).toBe('cursed-90s');
    expect(localStorage.getItem('memory-card-unlocked-themes')).toContain('cursed-90s');

    // Close, switch back to default, re-open terminal, and verify entering the code applies Cursed 90's again.
    fireEvent.click(within(terminal).getByRole('button', { name: en.easterEggs.crt.close }));
    fireEvent.click(screen.getByRole('tab', { name: en.settings.appearance }));
    fireEvent.click(await screen.findByRole('combobox', { name: en.settings.theme }));
    fireEvent.click(await screen.findByRole('option', { name: en.settings.themes.default, hidden: true }));
    expect(document.documentElement.dataset.mcTheme).toBe('default');

    fireEvent.click(screen.getByRole('tab', { name: en.settings.status }));
    const versionAgain = await screen.findByTestId('status-version');
    fireEvent.click(versionAgain);
    fireEvent.click(versionAgain);
    fireEvent.click(versionAgain);
    await screen.findByTestId('diagnostic-terminal');

    for (const key of ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a']) {
      fireEvent.keyDown(window, { key });
    }
    expect(document.documentElement.dataset.mcTheme).toBe('cursed-90s');
  });

  it('shows a mercy override button only after more than 30 misses', async () => {
    mockApi({ 'GET /api/status': STATUS });
    renderAt('/settings?tab=status');

    const version = await screen.findByTestId('status-version');
    fireEvent.click(version);
    fireEvent.click(version);
    fireEvent.click(version);

    const terminal = await screen.findByTestId('diagnostic-terminal');
    for (let i = 0; i < 30; i++) fireEvent.keyDown(window, { key: 'ArrowDown' });
    expect(within(terminal).queryByRole('button', { name: en.easterEggs.crt.mercyButton })).not.toBeInTheDocument();

    fireEvent.keyDown(window, { key: 'ArrowDown' });
    const mercy = within(terminal).getByRole('button', { name: en.easterEggs.crt.mercyButton });
    fireEvent.click(mercy);

    expect(document.documentElement.dataset.mcTheme).toBe('cursed-90s');
    expect(localStorage.getItem('memory-card-unlocked-themes')).toContain('cursed-90s');
  });
});
