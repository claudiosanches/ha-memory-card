import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameDetail } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi, VIEWER } from '../../test/api';
import { today } from '../../components/status';
import { renderAt } from '../../test/render';

/** Matches the element whose whole text is `text`, across a bold label. */
const wholeText = (text: string) => (_: string, element: Element | null) =>
  element?.textContent === text && [...element.children].every((child) => child.textContent !== text);

const GAME: GameDetail = {
  id: 4,
  igdb_id: null,
  title: 'Metroid Dread',
  summary: null,
  cover_path: null,
  cover_uploaded: false,
  first_release_date: null,
  developers: [],
  publishers: [],
  play_status: 'backlog',
  local_max_players: null,
  online_max_players: null,
  local_coop: null,
  online_coop: null,
  time_to_beat: { main: 680, extras: 1115 },
  game_mode_ids: [],
  genre_ids: [],
  labels: [],
  created_at: '2026-11-01T10:00:00.000Z',
  sessions: [],
  household: [],
  versions: [],
  names: [],
  releases: [
    {
      id: 9,
      platform_id: 1,
      region: 'eu',
      title: null,
      product_code: 'HAC-AYL3A',
      release_date: null,
      cover_path: null,
      cover_uploaded: false,
      local_max_players: null,
      online_max_players: null,
      local_coop: null,
      online_coop: null,
      copies: [
        { id: 21, status: 'owned', format: 'physical', device_id: null, store_id: null, condition: 'cib', notes: null, source: 'manual', created_at: '2026-11-01T10:00:00.000Z', contents: [], photos: [], tags: [] },
      ],
    },
  ],
};

const SETUP = {
  'GET /api/platforms': [
    {
      id: 1,
      name: 'Nintendo Switch',
      slug: 'switch',
      abbreviation: 'Switch',
      family: null,
      generation: null,
      igdb_platform_id: 130,
      active: true,
      sort_order: 10,
      game_count: 1,
      owned_game_count: 1,
      wanted_game_count: 0,
      device_count: 0,
      account_count: 0,
    },
  ],
  'GET /api/devices': [],
  'GET /api/stores': [],
  'GET /api/game-modes': [],
};

/** Picks an item from a copy's menu. */
async function copyAction(copyId: number, name: string) {
  const copy = await screen.findByTestId(`copy-${copyId}`);
  fireEvent.click(within(copy).getByRole('button', { name: en.game.moreCopyActions }));
  fireEvent.click(await screen.findByRole('menuitem', { name }));
}

describe('Game page', () => {
  it('shows releases and copies', async () => {
    mockApi({ ...SETUP, 'GET /api/games/4': GAME });
    renderAt('/games/4');

    const release = await screen.findByTestId('release-9');
    await waitFor(() => expect(within(release).getByText('Nintendo Switch · Europe (PAL)')).toBeInTheDocument());
    expect(within(release).getByText('HAC-AYL3A')).toBeInTheDocument();
    expect(within(screen.getByTestId('copy-21')).getByText(en.condition.cib)).toBeInTheDocument();
  });

  it('says who added the game and who changed it last', async () => {
    mockApi({
      ...SETUP,
      'GET /api/games/4': {
        ...GAME,
        created_by: { id: 'a1', name: 'Sam' },
        updated_by: { id: 'b2', name: 'Alex' },
      },
    });
    renderAt('/games/4');

    const history = await screen.findByTestId('game-history');
    expect(history.textContent).toMatch(/^Added by Sam on .+\. Last changed by Alex\.$/);
  });

  it('leaves out the editor when it is who added the game', async () => {
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, created_by: { id: 'a1', name: null }, updated_by: { id: 'a1' } } });
    renderAt('/games/4');

    expect((await screen.findByTestId('game-history')).textContent).toMatch(/^Added by someone on [^.]+\.$/);
  });

  it('shows a viewer the game without edit controls', async () => {
    mockApi({ ...SETUP, 'GET /api/me': VIEWER, 'GET /api/games/4': GAME });
    renderAt('/games/4');

    const copy = await screen.findByTestId('copy-21');
    expect(within(copy).getByText(en.condition.cib)).toBeInTheDocument();
    expect(within(copy).queryByRole('button')).not.toBeInTheDocument();
    expect(within(screen.getByTestId('release-9')).queryByRole('button')).not.toBeInTheDocument();
    for (const name of [en.common.edit, en.game.delete, en.game.addCopy, en.game.playedToday, en.cover.changeGame]) {
      expect(screen.queryByRole('button', { name })).not.toBeInTheDocument();
    }
  });

  it('shows other names and edits the ones added by hand', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': {
        ...GAME,
        names: [
          { name: 'MD', from_igdb: false, comment: null },
          { name: 'メトロイド ドレッド', from_igdb: true, comment: 'Japanese title' },
        ],
      },
      'PUT /api/games/4': GAME,
    });
    renderAt('/games/4');

    expect(await screen.findByTestId('names')).toHaveTextContent('MD, メトロイド ドレッド (Japanese title)');
    fireEvent.click(screen.getByRole('button', { name: en.common.edit }));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByText(/IGDB also gives: メトロイド ドレッド/)).toBeInTheDocument();
    const names = within(dialog).getByLabelText(en.game.names);
    names.focus();
    fireEvent.change(names, { target: { value: 'Dread' } });
    fireEvent.keyDown(names, { key: 'Enter' });
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    expect(calls.find((c) => c.method === 'PUT')?.body).toMatchObject({ names: ['MD', 'Dread'] });
  });

  it('links other versions of the game', async () => {
    mockApi({
      ...SETUP,
      'GET /api/games/4': { ...GAME, versions: [{ game_id: 9, title: 'Metroid Prime Remastered', cover_path: null, relation: 'remaster' }] },
    });
    renderAt('/games/4');

    const versions = await screen.findByTestId('versions');
    expect(within(versions).getByRole('link', { name: 'Metroid Prime Remastered' })).toBeInTheDocument();
    expect(versions).toHaveTextContent('(remaster)');
  });

  it("shows IGDB's time to beat in hours", async () => {
    mockApi({ ...SETUP, 'GET /api/games/4': GAME });
    renderAt('/games/4');

    expect(await screen.findByTestId('time-to-beat')).toHaveTextContent(
      'Time to beat: 11 hours for the main story, 19 hours with extras',
    );
  });

  it('opens the library filtered by a game mode or genre', async () => {
    mockApi({
      ...SETUP,
      'GET /api/game-modes': [{ id: 3, name: 'Co-operative', slug: 'co-operative' }],
      'GET /api/genres': [{ id: 8, name: 'Platform', slug: 'platform' }],
      'GET /api/games/4': { ...GAME, game_mode_ids: [3], genre_ids: [8] },
      'GET /api/games': [],
    });
    const { router } = renderAt('/games/4');

    fireEvent.click(await screen.findByRole('link', { name: 'Co-operative' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/'));
    expect(router.state.location.searchStr).toBe('?game_mode=co-operative');

    await router.navigate({ to: '/games/$gameId', params: { gameId: '4' } });
    fireEvent.click(await screen.findByRole('link', { name: 'Platform' }));
    await waitFor(() => expect(router.state.location.searchStr).toBe('?genre=platform'));
  });

  it('shows labels after the summary only when the game has some, linked to the library', async () => {
    const labels = [{ id: 2, name: 'Couch co-op', slug: 'couch-co-op', color: 'teal' as const }];
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, summary: 'Samus returns.', labels }, 'GET /api/games': [] });
    const { router } = renderAt('/games/4');

    const row = await screen.findByTestId('labels');
    expect(row).toHaveTextContent(`${en.game.labels}:Couch co-op`);
    expect(screen.getByText('Samus returns.', { exact: false }).compareDocumentPosition(row)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    fireEvent.click(within(row).getByRole('link', { name: 'Couch co-op' }));
    await waitFor(() => expect(router.state.location.searchStr).toBe('?label=couch-co-op'));
  });

  it('hides the labels row when the game has none', async () => {
    mockApi({ ...SETUP, 'GET /api/games/4': GAME });
    renderAt('/games/4');

    await screen.findByRole('heading', { name: GAME.title });
    expect(screen.queryByTestId('labels')).not.toBeInTheDocument();
  });

  it('sets labels in the edit form', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'GET /api/labels': [
        { id: 2, name: 'Couch co-op', slug: 'couch-co-op', color: 'teal', game_count: 0 },
        { id: 3, name: 'Retro', slug: 'retro', color: 'orange', game_count: 0 },
      ],
      'PUT /api/games/4': GAME,
    });
    renderAt('/games/4');

    await screen.findByRole('heading', { name: GAME.title });
    fireEvent.click(screen.getByRole('button', { name: en.common.edit }));
    const dialog = await screen.findByRole('dialog');
    const labels = within(dialog).getByRole('combobox', { name: en.game.labels });
    await waitFor(() => expect(labels).toBeEnabled());
    fireEvent.click(labels);
    fireEvent.click(await screen.findByRole('option', { name: 'Retro', hidden: true }));
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    expect(calls.find((c) => c.method === 'PUT')?.body).toMatchObject({ label_ids: [3] });
  });

  it('lists each copy with its edition, purchase and where it came from', async () => {
    const copy = {
      id: 21,
      status: 'owned',
      format: 'physical',
      device_id: null,
      store_id: 3,
      condition: 'cib',
      notes: 'Sealed spare',
      edition: 'Steelbook',
      purchased_on: '2026-10-05',
      price: '59.90',
      currency: 'USD',
      estimated_value: '75.00',
      estimated_currency: 'USD',
      source: 'csv',
      created_at: '2026-11-01T10:00:00.000Z',
      contents: [],
      photos: [],
      tags: [],
      created_by: { id: 'a1', name: 'Sam' },
    };
    mockApi({
      ...SETUP,
      'GET /api/stores': [{ id: 3, name: 'Amazon', slug: 'amazon', notes: null, copy_count: 1 }],
      'GET /api/games/4': { ...GAME, releases: [{ ...GAME.releases[0], copies: [copy] }] },
    });
    renderAt('/games/4');

    const item = await screen.findByTestId('copy-21');
    expect(item.tagName).toBe('LI');
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
    for (const text of [en.format.physical, 'Steelbook', en.condition.cib, 'Sealed spare', '$59.90', 'worth $75.00']) {
      expect(within(item).getByText(text)).toBeInTheDocument();
    }
    expect(await within(item).findByText('Amazon')).toBeInTheDocument();
    expect(within(item).getByText(/^Bought /)).toBeInTheDocument();
    expect(within(item).getByText(/^Added by Sam on /)).toHaveTextContent(en.history.fromCsv);
    expect(within(item).getByRole('button', { name: en.game.editCopy })).toBeInTheDocument();
  });

  it('shows co-op next to player counts', async () => {
    const release = { ...GAME.releases[0], online_coop: true };
    mockApi({
      ...SETUP,
      'GET /api/games/4': { ...GAME, local_max_players: 2, local_coop: true, releases: [release] },
    });
    renderAt('/games/4');

    expect(
      await screen.findByText(wholeText(`${en.game.players}: 2 on the same screen (co-op), no online play`)),
    ).toBeInTheDocument();
    // The release adds online co-op to the game's values.
    const card = screen.getByTestId('release-9');
    expect(within(card).getByText(`${en.game.players}: 2 on the same screen (co-op), Unknown online (co-op)`)).toBeInTheDocument();
  });

  it('shows one player when single player is the only mode and counts are unknown', async () => {
    const modes = [
      { id: 1, name: 'Single player', slug: 'single-player' },
      { id: 2, name: 'Multiplayer', slug: 'multiplayer' },
    ];
    mockApi({ ...SETUP, 'GET /api/game-modes': modes, 'GET /api/games/4': { ...GAME, game_mode_ids: [1] } });
    renderAt('/games/4');
    expect(await screen.findByText(wholeText(`${en.game.players}: ${en.game.singlePlayer}`))).toBeInTheDocument();
  });

  it('keeps unknown counts for games with a multiplayer mode', async () => {
    const modes = [
      { id: 1, name: 'Single player', slug: 'single-player' },
      { id: 2, name: 'Multiplayer', slug: 'multiplayer' },
    ];
    mockApi({ ...SETUP, 'GET /api/game-modes': modes, 'GET /api/games/4': { ...GAME, game_mode_ids: [1, 2] } });
    renderAt('/games/4');
    expect(
      await screen.findByText(wholeText(`${en.game.players}: Unknown locally, Unknown online`)),
    ).toBeInTheDocument();
  });

  it('hides a release title equal to the game title', async () => {
    const release = { ...GAME.releases[0], title: 'Metroid Dread' };
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, releases: [release] } });
    renderAt('/games/4');

    const card = await screen.findByTestId('release-9');
    expect(within(card).getByText('HAC-AYL3A')).toBeInTheDocument();
    expect(within(card).queryByText(/Metroid Dread/)).toBeNull();
  });

  it('shows a release cover when the release has its own', async () => {
    const release = { ...GAME.releases[0], title: 'メトロイド ドレッド', cover_path: 'release-9/co5wrq.jpg' };
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, releases: [release] } });
    renderAt('/games/4');

    const card = await screen.findByTestId('release-9');
    expect(within(card).getByRole('img')).toHaveAttribute('src', 'covers/release-9/co5wrq.jpg');
  });

  it('opens the cover full size and changes it only from its button', async () => {
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, cover_path: 'game-4/co1.jpg' } });
    renderAt('/games/4');

    const [hero] = await screen.findAllByRole('button', { name: en.cover.open.replace('{{title}}', 'Metroid Dread') });
    fireEvent.click(hero!);
    expect(screen.queryByRole('menuitem', { name: en.cover.upload })).toBeNull();
    const dialog = await screen.findByRole('dialog', { name: 'Metroid Dread' });
    expect(within(dialog).getByRole('img')).toHaveAttribute('src', 'covers/game-4/co1.jpg');
  });

  it('uploads a release cover from the cover menu', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'PUT /api/releases/9/cover': {
        ...GAME,
        releases: [{ ...GAME.releases[0], cover_path: 'release-9/upload-1.png', cover_uploaded: true }],
      },
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.cover.changeRelease }));
    expect(await screen.findByRole('menuitem', { name: en.cover.upload })).toBeInTheDocument();
    expect(screen.queryByRole('menuitem', { name: en.cover.remove })).toBeNull();
    const file = new File(['png bytes'], 'box.png', { type: 'image/png' });
    fireEvent.change(screen.getByTestId('cover-file-release-9'), { target: { files: [file] } });

    const card = screen.getByTestId('release-9');
    await waitFor(() =>
      expect(within(card).getByRole('img')).toHaveAttribute('src', 'covers/release-9/upload-1.png'),
    );
    expect(calls.find((c) => c.method === 'PUT')?.body).toBe('png bytes');
  });

  it('keeps the release after saving a copy and adding another', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/stores': [{ id: 3, name: 'Amazon', notes: null, copy_count: 0 }],
      'GET /api/games/4': GAME,
      'POST /api/games/4/copies': GAME,
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.addCopy }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.region }));
    fireEvent.click(await screen.findByRole('option', { name: en.region.jp, hidden: true }));
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.platform }));
    fireEvent.click(await screen.findByRole('option', { name: 'Nintendo Switch', hidden: true }));
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.store }));
    fireEvent.click(await screen.findByRole('option', { name: 'Amazon', hidden: true }));
    fireEvent.click(within(dialog).getByRole('button', { name: en.game.saveAnother }));

    await waitFor(() => expect(calls.filter((c) => c.method === 'POST')).toHaveLength(1));
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({ platform_id: 1, region: 'jp', store_id: 3 });
    await waitFor(() => expect(within(dialog).getByRole('combobox', { name: en.game.store })).toHaveValue(''));
    expect(within(dialog).getByRole('combobox', { name: en.game.platform })).toHaveValue('Nintendo Switch');
    expect(within(dialog).getByRole('combobox', { name: en.game.region })).toHaveValue(en.region.jp);
  });

  it('adds a copy once while its save is running', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'POST /api/games/4/copies': () => new Promise(() => {}),
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.addCopy }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.region }));
    fireEvent.click(await screen.findByRole('option', { name: en.region.jp, hidden: true }));
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.platform }));
    fireEvent.click(await screen.findByRole('option', { name: 'Nintendo Switch', hidden: true }));
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.filter((c) => c.method === 'POST')).toHaveLength(1));

    fireEvent.click(within(dialog).getByRole('button', { name: en.game.saveAnother }));
    fireEvent.submit(dialog.querySelector('form') as HTMLFormElement);
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(calls.filter((c) => c.method === 'POST')).toHaveLength(1);
  });

  it('prices a copy in the currency used last', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: false, external_services: true, default_currency: 'BRL', sensors: 'off' },
      'GET /api/games/4': GAME,
      'POST /api/games/4/copies': GAME,
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.addCopy }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.region }));
    fireEvent.click(await screen.findByRole('option', { name: en.region.br, hidden: true }));
    fireEvent.click(within(dialog).getByRole('combobox', { name: en.game.platform }));
    fireEvent.click(await screen.findByRole('option', { name: 'Nintendo Switch', hidden: true }));
    fireEvent.change(within(dialog).getByLabelText(en.game.edition), { target: { value: 'Steelbook' } });
    fireEvent.change(within(dialog).getByLabelText(en.game.purchasedOn), { target: { value: '2026-11-20' } });
    fireEvent.change(within(dialog).getByLabelText(en.game.price), { target: { value: '1.299,90' } });
    expect((within(dialog).getByRole('combobox', { name: en.game.currency }) as HTMLInputElement).value).toMatch(/^BRL /);
    fireEvent.change(within(dialog).getByLabelText(en.game.estimatedValue), { target: { value: '80' } });
    expect((within(dialog).getByRole('combobox', { name: en.game.estimatedCurrency }) as HTMLInputElement).value).toMatch(/^BRL /);
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({
      edition: 'Steelbook',
      purchased_on: '2026-11-20',
      price: '1.299,90',
      currency: 'BRL',
      estimated_value: '80',
      estimated_currency: 'BRL',
    });
  });

  it('adds the games IGDB lists in a bundle', async () => {
    const bundle = { ...GAME, igdb_id: 50 };
    const withContents = {
      ...bundle,
      releases: [
        {
          ...GAME.releases[0]!,
          copies: [{ ...GAME.releases[0]!.copies[0]!, contents: [{ copy_id: 30, game_id: 7, title: 'Super Mario 64' }] }],
        },
      ],
    };
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true, sensors: 'off' },
      'GET /api/games/4': bundle,
      'GET /api/games/4/igdb-contents': [
        { igdb_id: 64, name: 'Super Mario 64', year: 1996, platforms: [], game_id: 7 },
        { igdb_id: 65, name: 'Super Mario Sunshine', year: 2002, platforms: [], game_id: null },
      ],
      'POST /api/games': { ...GAME, id: 8, title: 'Super Mario Sunshine' },
      'POST /api/games/8/igdb': { ...GAME, id: 8 },
      'POST /api/copies/21/contents': withContents,
    });
    renderAt('/games/4');

    await copyAction(21, en.bundle.add);
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByLabelText('Super Mario Sunshine (2002)')).toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add 2 games' }));

    await waitFor(() => expect(calls.filter((c) => c.path === '/api/copies/21/contents')).toHaveLength(2));
    expect(calls.filter((c) => c.path === '/api/copies/21/contents').map((c) => c.body)).toEqual([{ game_id: 7 }, { game_id: 8 }]);
    expect(calls.find((c) => c.path === '/api/games/8/igdb')?.body).toEqual({ igdb_id: 65, overwrite: false });
    const contents = await screen.findByTestId('contents-21');
    expect(within(contents).getByRole('link', { name: 'Super Mario 64' })).toBeInTheDocument();
  });

  it('retries only the bundle contents a failed run left out', async () => {
    let failed = false;
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true, sensors: 'off' },
      'GET /api/games/4': { ...GAME, igdb_id: 50 },
      'GET /api/games/4/igdb-contents': [
        { igdb_id: 64, name: 'Super Mario 64', year: 1996, platforms: [], game_id: 7 },
        { igdb_id: 65, name: 'Super Mario Sunshine', year: 2002, platforms: [], game_id: null },
      ],
      'POST /api/games': { ...GAME, id: 8, title: 'Super Mario Sunshine' },
      'POST /api/games/8/igdb': { ...GAME, id: 8 },
      'POST /api/copies/21/contents': (body: { game_id: number }) => {
        if (body.game_id !== 8 || failed) return GAME;
        failed = true;
        return new Response(JSON.stringify({ code: 'internal' }), { status: 500 });
      },
    });
    renderAt('/games/4');

    await copyAction(21, en.bundle.add);
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByLabelText('Super Mario Sunshine (2002)');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add 2 games' }));
    await waitFor(() => expect(failed).toBe(true));
    await waitFor(() => expect(within(dialog).getByRole('button', { name: 'Add 2 games' })).toBeEnabled());
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add 2 games' }));

    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
    expect(calls.filter((c) => c.path === '/api/games' && c.method === 'POST')).toHaveLength(1);
    expect(calls.filter((c) => c.path === '/api/copies/21/contents').map((c) => c.body)).toEqual([
      { game_id: 7 },
      { game_id: 8 },
      { game_id: 8 },
    ]);
  });

  it('shows a copy that came in a bundle with only a remove action', async () => {
    const inside = { ...GAME.releases[0]!.copies[0]!, bundle: { copy_id: 3, game_id: 9, title: 'Mario Pack' } };
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, releases: [{ ...GAME.releases[0]!, copies: [inside] }] } });
    renderAt('/games/4');

    const item = await screen.findByTestId('copy-21');
    expect(within(screen.getByTestId('bundle-21')).getByRole('link', { name: 'Mario Pack' })).toBeInTheDocument();
    expect(within(item).getByRole('button', { name: en.bundle.remove })).toBeInTheDocument();
    expect(within(item).queryByRole('button', { name: en.game.editCopy })).not.toBeInTheDocument();
  });

  it('uploads photos of a copy and shows them', async () => {
    const photo = { id: 3, path: 'copy-21/photo-1.jpg', created_at: '2026-11-21T10:00:00Z' };
    const withPhoto = {
      ...GAME,
      releases: [{ ...GAME.releases[0]!, copies: [{ ...GAME.releases[0]!.copies[0]!, photos: [photo] }] }],
    };
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'POST /api/copies/21/photos': withPhoto,
      'DELETE /api/photos/3': GAME,
    });
    renderAt('/games/4');

    const input = await screen.findByTestId('photo-input-21');
    const file = new File(['jpeg'], 'box.jpg', { type: 'image/jpeg' });
    fireEvent.change(input, { target: { files: [file] } });
    await waitFor(() => expect(calls.some((c) => c.path === '/api/copies/21/photos')).toBe(true));

    fireEvent.click(await screen.findByRole('button', { name: 'Open photo 1' }));
    const viewer = await screen.findByRole('dialog', { name: 'Photo 1 of 1' });
    fireEvent.click(within(viewer).getByRole('button', { name: en.photos.delete }));
    await waitFor(() => expect(calls.some((c) => c.method === 'DELETE' && c.path === '/api/photos/3')).toBe(true));
  });

  it('shows the photo before the last one once the last is deleted', async () => {
    const photos = [1, 2, 3].map((id) => ({ id, path: `copy-21/photo-${id}.jpg`, created_at: '2026-11-21T10:00:00Z' }));
    const withPhotos = (list: typeof photos) => ({
      ...GAME,
      releases: [{ ...GAME.releases[0]!, copies: [{ ...GAME.releases[0]!.copies[0]!, photos: list }] }],
    });
    mockApi({
      ...SETUP,
      'GET /api/games/4': withPhotos(photos),
      'DELETE /api/photos/3': withPhotos(photos.slice(0, 2)),
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: 'Open photo 3' }));
    const viewer = await screen.findByRole('dialog', { name: 'Photo 3 of 3' });
    fireEvent.click(within(viewer).getByRole('button', { name: en.photos.delete }));
    const after = await screen.findByRole('dialog', { name: 'Photo 2 of 2' });
    expect(within(after).getByRole('img')).toHaveAttribute('src', 'covers/copy-21/photo-2.jpg');
    fireEvent.click(within(after).getByRole('button', { name: en.photos.previous }));
    expect(within(await screen.findByRole('dialog', { name: 'Photo 1 of 2' })).getByRole('img')).toHaveAttribute(
      'src',
      'covers/copy-21/photo-1.jpg',
    );
  });

  it('links an NFC tag once Home Assistant reports a scan', async () => {
    const tagged = {
      ...GAME,
      releases: [
        { ...GAME.releases[0]!, copies: [{ ...GAME.releases[0]!.copies[0]!, tags: [{ tag_id: 'a1', created_at: '2026-11-21T10:00:00Z' }] }] },
      ],
    };
    let scanned = false;
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: false, external_services: true, sensors: 'off', tags: 'connected' },
      'GET /api/games/4': () => (scanned ? tagged : GAME),
      'POST /api/copies/21/tag-link': () => {
        scanned = true;
        return { expires_at: new Date(Date.now() + 60_000).toISOString() };
      },
      'DELETE /api/copies/21/tag-link': new Response(null, { status: 204 }),
    });
    renderAt('/games/4');

    await copyAction(21, en.tags.link);
    expect(await screen.findByTestId('tag-waiting')).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument(), { timeout: 4000 });
    expect(await screen.findByTestId('tags-21')).toHaveTextContent('1 NFC tag starts this copy');
    expect(calls.some((c) => c.method === 'DELETE' && c.path === '/api/copies/21/tag-link')).toBe(true);
  });

  it('cancels a tag link only after it started', async () => {
    let start = () => {};
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: false, external_services: true, sensors: 'off', tags: 'connected' },
      'GET /api/games/4': GAME,
      'POST /api/copies/21/tag-link': () =>
        new Promise((resolve) => (start = () => resolve({ expires_at: new Date(Date.now() + 60_000).toISOString() }))),
      'DELETE /api/copies/21/tag-link': new Response(null, { status: 204 }),
    });
    renderAt('/games/4');

    await copyAction(21, en.tags.link);
    await waitFor(() => expect(calls.some((c) => c.path === '/api/copies/21/tag-link')).toBe(true));
    fireEvent.click(within(await screen.findByRole('dialog')).getByRole('button', { name: en.common.cancel }));
    await new Promise((resolve) => setTimeout(resolve, 50));
    const links = () => calls.filter((c) => c.path === '/api/copies/21/tag-link').map((c) => c.method);
    expect(links()).toEqual(['POST']);
    start();
    await waitFor(() => expect(links()).toEqual(['POST', 'DELETE']));
  });

  it('offers to delete the game with its last copy', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'DELETE /api/games/4': new Response(null, { status: 204 }),
      'GET /api/games': [],
    });
    renderAt('/games/4');

    await copyAction(21, en.game.deleteCopyAction);
    const dialog = await screen.findByRole('dialog');
    expect(dialog).toHaveTextContent(en.game.lastCopy.replace('{{title}}', GAME.title));
    fireEvent.click(within(dialog).getByRole('button', { name: en.game.deleteGameToo }));

    await waitFor(() => expect(calls.some((c) => c.method === 'DELETE' && c.path === '/api/games/4')).toBe(true));
    expect(calls.some((c) => c.path.startsWith('/api/copies'))).toBe(false);
    expect(await screen.findByRole('heading', { name: en.nav.library })).toBeInTheDocument();
  });

  it('deletes only the copy when the game keeps others', async () => {
    const second = { ...GAME.releases[0]!.copies[0]!, id: 22 };
    const game = { ...GAME, releases: [{ ...GAME.releases[0]!, copies: [...GAME.releases[0]!.copies, second] }] };
    const calls = mockApi({ ...SETUP, 'GET /api/games/4': game, 'DELETE /api/copies/22': GAME });
    renderAt('/games/4');

    await copyAction(22, en.game.deleteCopyAction);
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).queryByRole('button', { name: en.game.deleteGameToo })).not.toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.delete }));
    await waitFor(() => expect(calls.some((c) => c.method === 'DELETE')).toBe(true));
    expect(calls.find((c) => c.method === 'DELETE')?.path).toBe('/api/copies/22');
  });

  it('plays only owned copies and moves a wanted one into the collection', async () => {
    const wanted = { ...GAME.releases[0]!.copies[0]!, id: 22, status: 'wanted' as const };
    const game = { ...GAME, releases: [{ ...GAME.releases[0]!, copies: [...GAME.releases[0]!.copies, wanted] }] };
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': game,
      'POST /api/games/4/sessions': game,
      'PUT /api/copies/22': GAME,
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.playedToday }));
    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/4/sessions')).toBe(true));
    expect((calls.find((c) => c.path === '/api/games/4/sessions')?.body as Record<string, unknown>).copy_id).toBe(21);

    expect(screen.getAllByRole('button', { name: en.game.gotIt })).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: en.game.gotIt }));
    const dialog = await screen.findByRole('dialog', { name: en.game.gotItTitle });
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.some((c) => c.path === '/api/copies/22')).toBe(true));
    expect(calls.find((c) => c.path === '/api/copies/22')?.body).toMatchObject({ status: 'owned' });
  });

  it('logs the only copy as played today', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'POST /api/games/4/sessions': (body: unknown) => ({
        ...GAME,
        sessions: [{ id: 1, ...(body as object), minutes: null, notes: null, created_at: '2026-11-21T10:00:00Z' }],
      }),
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.playedToday }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    const body = calls.find((c) => c.method === 'POST')?.body as Record<string, unknown>;
    expect(body.played_on).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(body.copy_id).toBe(21);
    expect(await screen.findByRole('button', { name: en.game.loggedToday })).toBeDisabled();
    expect(within(screen.getByTestId('session-1')).getByText(/Nintendo Switch/)).toBeInTheDocument();
  });

  it('keeps Played today off when a session dated ahead comes first', async () => {
    const session = (id: number, played_on: string) => ({ id, played_on, copy_id: 21, minutes: null, notes: null, created_at: '2026-11-02T20:00:00Z' });
    mockApi({ ...SETUP, 'GET /api/games/4': { ...GAME, sessions: [session(2, '2999-01-01'), session(1, today())] } });
    renderAt('/games/4');

    expect(await screen.findByRole('button', { name: en.game.loggedToday })).toBeDisabled();
  });

  it('writes dates in the format the app options name', async () => {
    const session = { id: 5, played_on: '2026-11-02', copy_id: 21, minutes: null, notes: null, created_at: '2026-11-02T20:00:00Z' };
    mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: false, external_services: true, sensors: 'off', date_format: 'dmy' },
      'GET /api/games/4': { ...GAME, sessions: [session] },
    });
    renderAt('/games/4');

    const row = await screen.findByTestId('session-5');
    expect(await within(row).findByText('02/11/2026')).toBeInTheDocument();
  });

  it('logs and edits a session with minutes and notes', async () => {
    const session = {
      id: 5,
      played_on: '2026-11-02',
      copy_id: 21,
      minutes: 90,
      notes: 'Boss fight',
      created_at: '2026-11-02T20:00:00Z',
    };
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': { ...GAME, sessions: [session] },
      'PUT /api/sessions/5': { ...GAME, sessions: [session] },
      'POST /api/games/4/sessions': { ...GAME, sessions: [session] },
    });
    renderAt('/games/4');

    const row = await screen.findByTestId('session-5');
    expect(within(row).getByText(/90 minutes/)).toBeInTheDocument();
    expect(within(row).getByText('Boss fight')).toBeInTheDocument();

    fireEvent.click(within(row).getByRole('button', { name: en.sessions.edit }));
    let dialog = await screen.findByRole('dialog', { name: en.sessions.edit });
    fireEvent.change(within(dialog).getByLabelText(en.sessions.minutesLabel), { target: { value: '120' } });
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.some((c) => c.path === '/api/sessions/5')).toBe(true));
    expect(calls.find((c) => c.path === '/api/sessions/5')?.body).toEqual({
      played_on: '2026-11-02',
      copy_id: 21,
      minutes: 120,
      notes: 'Boss fight',
    });

    fireEvent.click(screen.getByRole('button', { name: en.sessions.add }));
    dialog = await screen.findByRole('dialog', { name: en.sessions.add });
    fireEvent.change(within(dialog).getByLabelText(new RegExp(en.sessions.playedOn)), { target: { value: '2026-11-03' } });
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));
    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({
      played_on: '2026-11-03',
      copy_id: 21,
      minutes: null,
      notes: null,
    });
  });

  it('lets a viewer track progress when the option allows it', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/me': { ...VIEWER, can_track_progress: true },
      'GET /api/games/4': GAME,
      'POST /api/games/4/sessions': GAME,
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.game.playedToday }));
    await waitFor(() => expect(calls.some((c) => c.path === '/api/games/4/sessions')).toBe(true));
    expect(screen.queryByRole('button', { name: en.common.edit })).not.toBeInTheDocument();
  });

  it("shows everyone else's play without letting anyone change it", async () => {
    mockApi({
      ...SETUP,
      'GET /api/me': { ...VIEWER, can_track_progress: true },
      'GET /api/games/4': {
        ...GAME,
        household: [
          {
            person: { id: 'b2', name: 'Ana' },
            play_status: 'completed',
            sessions: 3,
            minutes: 120,
            last_played_on: '2026-11-01',
          },
        ],
      },
    });
    renderAt('/games/4');

    const household = await screen.findByTestId('household');
    expect(within(household).getByText(en.game.household)).toBeInTheDocument();
    expect(within(household).getByText('Ana')).toBeInTheDocument();
    expect(await within(household).findByText(en.playStatus.completed)).toBeInTheDocument();
    expect(within(household).getByText(/3 sessions/)).toBeInTheDocument();
    expect(within(household).queryByRole('button')).not.toBeInTheDocument();
  });

  it('links the game to an IGDB search result', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true },
      'GET /api/games/4': GAME,
      'GET /api/igdb/search': [{ igdb_id: 7, name: 'Metroid Dread', year: 2021, platforms: ['Switch'], game_id: null }],
      'POST /api/games/4/igdb': { ...GAME, igdb_id: 7, developers: ['MercurySteam'] },
    });
    renderAt('/games/4');

    fireEvent.click(await screen.findByRole('button', { name: en.igdb.link }));
    await screen.findByTestId('igdb-hit-7');
    const search = screen.getByRole('textbox', { name: en.igdb.searchLabel });
    fireEvent.change(search, { target: { value: 'Metroid Dread (2021)' } });
    expect(calls.filter((c) => c.path === '/api/igdb/search')).toHaveLength(1);
    fireEvent.submit(search);
    await waitFor(() => expect(calls.filter((c) => c.path === '/api/igdb/search')).toHaveLength(2));
    fireEvent.click(await screen.findByTestId('igdb-hit-7'));

    expect(await screen.findByText(`${en.game.developers}: MercurySteam`)).toBeInTheDocument();
    expect(calls.find((c) => c.path === '/api/igdb/search')?.query.get('q')).toBe('Metroid Dread');
    expect(calls.find((c) => c.method === 'POST')?.body).toEqual({ igdb_id: 7, overwrite: false });
  });

  it('says when the game does not exist', async () => {
    mockApi({ ...SETUP, 'GET /api/games/77': json({ code: 'not_found' }, 404) });
    renderAt('/games/77');
    expect(await screen.findByText(en.game.notFound)).toBeInTheDocument();
  });
});
