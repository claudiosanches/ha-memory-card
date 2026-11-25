import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { components } from '../api/schema';
import en from '../locales/en.json';
import { json, mockApi } from '../test/api';
import { renderAt } from '../test/render';

type Preview = components['schemas']['ImportPreview'];

const CSV = 'game_title,platform,region,format\nZelda,Switch,eu,physical\n';

function summary(extra: Partial<Preview['summary']> = {}): Preview['summary'] {
  return {
    rows: 1,
    new_games: 1,
    updated_games: 0,
    new_releases: 1,
    updated_releases: 0,
    new_copies: 1,
    updated_copies: 0,
    unchanged_copies: 0,
    game_only_rows: 0,
    error_rows: 0,
    warning_rows: 0,
    ...extra,
  };
}

async function chooseFile(content: string) {
  await screen.findByRole('heading', { name: en.import.importTitle });
  const input = document.querySelector('input[type="file"]') as HTMLInputElement;
  fireEvent.change(input, { target: { files: [new File([content], 'games.csv', { type: 'text/csv' })] } });
}

describe('Import page', () => {
  it('links the export and the template under the ingress prefix', async () => {
    mockApi({});
    renderAt('/import');
    expect(await screen.findByRole('link', { name: en.import.exportButton })).toHaveAttribute(
      'href',
      'api/export.csv',
    );
    expect(screen.getByRole('link', { name: en.import.templateButton })).toHaveAttribute(
      'href',
      'api/import/template.csv',
    );
  });

  it('previews a file, then imports it', async () => {
    const preview: Preview = {
      file_issue: null,
      headers: [],
      delimiter: ',',
      summary: summary(),
      new_platforms: [],
      new_devices: [{ name: 'Switch OLED', kind: 'console', platforms: ['Nintendo Switch'] }],
      new_stores: ['Amazon'],
      new_labels: [],
      rows: [{ line: 2, game_title: 'Zelda', platform: 'Switch', action: 'new_copy', issues: [] }],
    };
    const calls = mockApi({
      'POST /api/import/preview': preview,
      'POST /api/import': { summary: summary(), unlinked_games: 0 },
    });
    renderAt('/import');

    await chooseFile(CSV);
    const result = await screen.findByTestId('import-preview');
    expect(within(result).getByTestId('summary-new_copies')).toHaveTextContent('1');
    expect(within(result).getByText('Switch OLED (Console): Nintendo Switch')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: en.import.confirm }));
    expect(await screen.findByTestId('import-done')).toHaveTextContent(
      'Imported 1 new games and 1 new copies, updated 0 copies.',
    );
    const sent = calls.filter((c) => c.method === 'POST').map((c) => [c.path, c.body]);
    expect(sent).toEqual([
      ['/api/import/preview', CSV],
      ['/api/import', CSV],
    ]);
    expect(screen.queryByTestId('import-unlinked')).not.toBeInTheDocument();
  });

  it('points to the IGDB sync when imported games are not linked', async () => {
    mockApi({
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true },
      'POST /api/import/preview': {
        file_issue: null,
        headers: [],
        delimiter: ',',
        summary: summary(),
        new_platforms: [],
        new_devices: [],
        new_stores: [],
        new_labels: [],
        rows: [{ line: 2, game_title: 'Zelda', platform: 'Switch', action: 'new_copy', issues: [] }],
      },
      'POST /api/import': { summary: summary(), unlinked_games: 3 },
    });
    renderAt('/import');

    await chooseFile(CSV);
    fireEvent.click(await screen.findByRole('button', { name: en.import.confirm }));
    expect(await screen.findByTestId('import-unlinked')).toHaveTextContent(en.import.unlinked_other.replace('{{count}}', '3'));
    expect(screen.getByRole('link', { name: en.import.reviewMatches })).toHaveAttribute('href', expect.stringContaining('/igdb-matches'));
  });

  it('lists row problems and blocks the import', async () => {
    mockApi({
      'POST /api/import/preview': {
        file_issue: null,
        headers: [],
        delimiter: ',',
        summary: summary({ new_games: 0, new_releases: 0, new_copies: 0, error_rows: 1 }),
        new_platforms: [],
        new_devices: [],
        new_stores: [],
        new_labels: [],
        rows: [
          {
            line: 2,
            game_title: 'Zelda',
            platform: 'Switch',
            action: 'error',
            issues: [{ column: 'region', code: 'required', error: true }],
          },
        ],
      } satisfies Preview,
    });
    renderAt('/import');

    await chooseFile(CSV);
    const issues = await screen.findByTestId('import-issues');
    expect(within(issues).getByText('region')).toBeInTheDocument();
    expect(within(issues).getByText(en.importIssues.required)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: en.import.confirm })).toBeDisabled();
  });

  it('refuses a file larger than the server reads without reading it', async () => {
    const calls = mockApi({});
    renderAt('/import');
    await screen.findByRole('heading', { name: en.import.importTitle });
    const file = new File([CSV], 'games.csv', { type: 'text/csv' });
    Object.defineProperty(file, 'size', { value: 16 * 1024 * 1024 + 1 });
    file.arrayBuffer = () => Promise.reject(new Error('read'));
    fireEvent.change(document.querySelector('input[type="file"]') as HTMLInputElement, { target: { files: [file] } });

    expect(await screen.findByText(en.errors.too_large)).toBeInTheDocument();
    expect(calls.filter((c) => c.path === '/api/import/preview')).toHaveLength(0);
  });

  it('refuses a file that is not UTF-8 before the preview', async () => {
    const calls = mockApi({});
    renderAt('/import');
    await screen.findByRole('heading', { name: en.import.importTitle });
    const file = new File([new Uint8Array([0x67, 0x61, 0x6d, 0x65, 0x0a, 0xe9, 0x0a])], 'games.csv', { type: 'text/csv' });
    fireEvent.change(document.querySelector('input[type="file"]') as HTMLInputElement, { target: { files: [file] } });

    expect(await screen.findByText(en.fileIssues.not_utf8)).toBeInTheDocument();
    expect(calls.filter((c) => c.path === '/api/import/preview')).toHaveLength(0);
  });

  it('imports the file chosen last when an earlier one reads slower', async () => {
    const calls = mockApi({
      'POST /api/import/preview': {
        file_issue: null,
        headers: [],
        delimiter: ',',
        summary: summary(),
        new_platforms: [],
        new_devices: [],
        new_stores: [],
        new_labels: [],
        rows: [],
      } satisfies Preview,
      'POST /api/import': { summary: summary(), unlinked_games: 0 },
    });
    renderAt('/import');
    await screen.findByRole('heading', { name: en.import.importTitle });
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    const slow = new File(['game_title\nOld\n'], 'old.csv', { type: 'text/csv' });
    let finish = () => {};
    const read = slow.arrayBuffer();
    slow.arrayBuffer = () => new Promise((resolve) => (finish = () => resolve(read)));
    fireEvent.change(input, { target: { files: [slow] } });
    fireEvent.change(input, { target: { files: [new File([CSV], 'games.csv', { type: 'text/csv' })] } });

    fireEvent.click(await screen.findByRole('button', { name: en.import.confirm }));
    await screen.findByTestId('import-done');
    finish();
    await new Promise((resolve) => setTimeout(resolve, 0));
    const sent = calls.filter((c) => c.method === 'POST').map((c) => [c.path, c.body]);
    expect(sent).toEqual([
      ['/api/import/preview', CSV],
      ['/api/import', CSV],
    ]);
    expect(screen.queryByTestId('import-preview')).not.toBeInTheDocument();
  });

  it('explains a file it cannot read and lets its columns be picked', async () => {
    const calls = mockApi({
      'POST /api/import/preview': (body: unknown) => json({
        file_issue: { code: 'missing_column', column: 'game_title' },
        headers: String(body).startsWith('game_title')
          ? [
              { header: 'game_title', column: 'game_title' },
              { header: 'ignored 2', column: null },
            ]
          : [
              { header: 'Jogo', column: null },
              { header: 'Nota', column: null },
            ],
        delimiter: ',',
        summary: summary({ rows: 0, new_games: 0, new_releases: 0, new_copies: 0 }),
        new_platforms: [],
        new_devices: [],
        new_stores: [],
        new_labels: [],
        rows: [],
      }),
    });
    renderAt('/import');

    await chooseFile('Jogo,Nota\nZelda,5\n');
    expect(await screen.findByTestId('file-issue')).toHaveTextContent('The file needs a game_title column.');
    const jogo = screen.getByRole('combobox', { name: 'Read Jogo as' });
    fireEvent.click(jogo);
    const options = await waitFor(() => {
      const list = document.getElementById(jogo.getAttribute('aria-controls') ?? '');
      expect(list).not.toBeNull();
      return list as HTMLElement;
    });
    fireEvent.click(within(options).getByRole('option', { name: 'game_title', hidden: true }));
    await waitFor(() => expect(calls.filter((c) => c.path === '/api/import/preview')).toHaveLength(2));
    expect(calls.filter((c) => c.path === '/api/import/preview')[1]?.body).toBe('game_title,ignored 2\nZelda,5\n');
  });

  it('keeps the confirm button off when nothing changes', async () => {
    mockApi({
      'POST /api/import/preview': {
        file_issue: null,
        headers: [],
        delimiter: ',',
        summary: summary({ new_games: 0, new_releases: 0, new_copies: 0, unchanged_copies: 1 }),
        new_platforms: [],
        new_devices: [],
        new_stores: [],
        new_labels: [],
        rows: [],
      },
    });
    renderAt('/import');

    await chooseFile(CSV);
    await waitFor(() => expect(screen.getByText(en.import.nothingToDo)).toBeInTheDocument());
    expect(screen.getByRole('button', { name: en.import.confirm })).toBeDisabled();
  });
});
