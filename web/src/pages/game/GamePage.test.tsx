import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { GameDetail } from '../../api/types';
import en from '../../locales/en.json';
import { json, mockApi, VIEWER } from '../../test/api';
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
  game_mode_ids: [],
  genre_ids: [],
  created_at: '2026-11-01T10:00:00.000Z',
  sessions: [],
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
        { id: 21, status: 'owned', format: 'physical', device_id: null, store_id: null, condition: 'cib', notes: null, created_at: '2026-11-01T10:00:00.000Z' },
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
    },
  ],
  'GET /api/devices': [],
  'GET /api/stores': [],
  'GET /api/game-modes': [],
};

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

  it('opens the library filtered by a game mode or genre', async () => {
    mockApi({
      ...SETUP,
      'GET /api/game-modes': [{ id: 3, name: 'Co-operative', slug: 'co-operative' }],
      'GET /api/genres': [{ id: 8, name: 'Platform' }],
      'GET /api/games/4': { ...GAME, game_mode_ids: [3], genre_ids: [8] },
      'GET /api/games': [],
    });
    const { router } = renderAt('/games/4');

    fireEvent.click(await screen.findByRole('link', { name: 'Co-operative' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/'));
    expect(router.state.location.search).toEqual({ game_mode: [3] });

    await router.navigate({ to: '/games/$gameId', params: { gameId: '4' } });
    fireEvent.click(await screen.findByRole('link', { name: 'Platform' }));
    await waitFor(() => expect(router.state.location.search).toEqual({ genre: [8] }));
  });

  it('lists each copy with its edition, purchase and who added it', async () => {
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
      created_at: '2026-11-01T10:00:00.000Z',
      created_by: { id: 'a1', name: 'Sam' },
    };
    mockApi({
      ...SETUP,
      'GET /api/stores': [{ id: 3, name: 'Amazon', notes: null, copy_count: 1 }],
      'GET /api/games/4': { ...GAME, releases: [{ ...GAME.releases[0], copies: [copy] }] },
    });
    renderAt('/games/4');

    const item = await screen.findByTestId('copy-21');
    expect(item.tagName).toBe('LI');
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
    for (const text of [en.format.physical, 'Steelbook', en.condition.cib, 'Sealed spare', '$59.90']) {
      expect(within(item).getByText(text)).toBeInTheDocument();
    }
    expect(await within(item).findByText('Amazon')).toBeInTheDocument();
    expect(within(item).getByText(/^Bought /)).toBeInTheDocument();
    expect(within(item).getByText(/^Added by Sam on /)).toBeInTheDocument();
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

  it('prices a copy in the currency used last', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: false, external_services: true, games: 1, owned_copies: 1, last_currency: 'BRL', sensors: 'off' },
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
    fireEvent.click(within(dialog).getByRole('button', { name: en.common.save }));

    await waitFor(() => expect(calls.some((c) => c.method === 'POST')).toBe(true));
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({
      edition: 'Steelbook',
      purchased_on: '2026-11-20',
      price: '1.299,90',
      currency: 'BRL',
    });
  });

  it('offers to delete the game with its last copy', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': GAME,
      'DELETE /api/games/4': new Response(null, { status: 204 }),
      'GET /api/games': [],
    });
    renderAt('/games/4');

    fireEvent.click((await screen.findAllByRole('button', { name: en.game.deleteCopy }))[0]!);
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

    const buttons = await screen.findAllByRole('button', { name: en.game.deleteCopy });
    fireEvent.click(buttons[1]!);
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

  it('logs and edits a session with minutes and notes', async () => {
    const session = {
      id: 5,
      played_on: '2026-11-02',
      copy_id: 21,
      minutes: 90,
      notes: 'Boss fight',
      created_at: '2026-11-02T20:00:00Z',
      created_by: { id: 'a1', name: 'Sam' },
    };
    const calls = mockApi({
      ...SETUP,
      'GET /api/games/4': { ...GAME, sessions: [session] },
      'PUT /api/sessions/5': { ...GAME, sessions: [session] },
      'POST /api/games/4/sessions': { ...GAME, sessions: [session] },
    });
    renderAt('/games/4');

    const row = await screen.findByTestId('session-5');
    expect(within(row).getByText(/90 minutes · Sam/)).toBeInTheDocument();
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

  it('links the game to an IGDB search result', async () => {
    const calls = mockApi({
      ...SETUP,
      'GET /api/status': { version: '0.1.0', igdb_configured: true, external_services: true, games: 1, owned_copies: 1 },
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
