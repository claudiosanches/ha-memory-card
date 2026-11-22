import {
  ActionIcon,
  Anchor,
  Badge,
  Button,
  Card,
  Group,
  Loader,
  Menu,
  Select,
  Stack,
  Text,
  Title,
  Tooltip,
} from '@mantine/core';
import {
  IconArrowLeft,
  IconCalendarCheck,
  IconChevronDown,
  IconLink,
  IconPencil,
  IconPlus,
  IconRefresh,
  IconTrash,
} from '@tabler/icons-react';
import { useMediaQuery } from '@mantine/hooks';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { getRouteApi, Link, useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { ApiError, call, errorMessage } from '../../api/errors';
import {
  queryKeys,
  useCanManage,
  useCanTrackProgress,
  useDevices,
  useGame,
  useGameModes,
  useGenres,
  usePlatforms,
  useStatus,
  useStores,
} from '../../api/queries';
import {
  type CopyDetail,
  type GameDetail,
  type ReleaseDetail,
} from '../../api/types';
import { SectionHeading } from '../../components/SectionHeading';
import { ConfirmModal } from '../../components/ConfirmModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { usePlayStatusList } from '../../components/playStatuses';
import { history } from '../../components/history';
import { formatDate, today } from '../../components/status';
import { BundleContentsModal } from './BundleContents';
import { Copies } from './Copies';
import { TagLinkModal } from './TagLink';
import { defaultCopy, Sessions } from './Sessions';
import { CoverPicker } from './CoverPicker';
import { describePlayers, effectiveCounts, hasPlayerData } from './players';
import { CopyForm, GameForm, ReleaseForm, useGameSaved } from './forms';
import { IgdbLinkModal, useIgdbLink } from './IgdbLink';
import classes from './GamePage.module.css';

const route = getRouteApi('/games/$gameId');

type Dialog =
  | { kind: 'editGame' }
  | { kind: 'linkIgdb' }
  | { kind: 'deleteGame' }
  | { kind: 'addCopy'; release?: ReleaseDetail }
  | { kind: 'editCopy'; copy: CopyDetail; release: ReleaseDetail }
  | { kind: 'gotIt'; copy: CopyDetail; release: ReleaseDetail }
  | { kind: 'deleteCopy'; copy: CopyDetail }
  | { kind: 'contents'; copy: CopyDetail }
  | { kind: 'linkTag'; copy: CopyDetail }
  | { kind: 'editRelease'; release: ReleaseDetail }
  | { kind: 'deleteRelease'; release: ReleaseDetail }
  | null;

function copyCount(game: GameDetail) {
  return game.releases.reduce((sum, release) => sum + release.copies.length, 0);
}

export function GamePage() {
  const { t } = useTranslation();
  const { gameId } = route.useParams();
  const id = Number(gameId);
  const game = useGame(id);

  if (game.isPending) return <Loader size="sm" />;
  if (game.isError) {
    const missing = game.error instanceof ApiError && game.error.code === 'not_found';
    return (
      <Stack>
        <BackLink />
        <Text c={missing ? 'dimmed' : 'red'}>{missing ? t('game.notFound') : errorMessage(t, game.error)}</Text>
      </Stack>
    );
  }
  return <GameView game={game.data} />;
}

function BackLink() {
  const { t } = useTranslation();
  return (
    <Anchor
      size="sm"
      renderRoot={(props) => <Link to="/" {...props} />}
    >
      <Group gap={4}>
        <IconArrowLeft size={14} />
        {t('game.back')}
      </Group>
    </Anchor>
  );
}

function GameView({ game }: { game: GameDetail }) {
  const { t, i18n } = useTranslation();
  const statuses = usePlayStatusList();
  const [dialog, setDialog] = useState<Dialog>(null);
  const phone = useMediaQuery('(max-width: 48em)');
  const close = () => setDialog(null);
  const saved = useGameSaved();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const gameModes = useGameModes();
  const genres = useGenres();
  const status = useStatus();
  const refresh = useIgdbLink(game);
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const canManage = useCanManage();
  const canTrack = useCanTrackProgress();

  const names = useMemo(
    () => ({
      platform: new Map((platforms.data ?? []).map((p) => [p.id, p.name])),
      device: new Map((devices.data ?? []).map((d) => [d.id, d.name])),
      store: new Map((stores.data ?? []).map((s) => [s.id, s.name])),
      mode: new Map((gameModes.data ?? []).map((m) => [m.id, m.name])),
      genre: new Map((genres.data ?? []).map((g) => [g.id, g.name])),
    }),
    [platforms.data, devices.data, stores.data, gameModes.data, genres.data],
  );

  const allCopies = game.releases.flatMap((release) => release.copies.map((copy) => ({ copy, release })));
  // Only copies you have can be played.
  const ownedCopies = allCopies.filter(({ copy }) => copy.status === 'owned');
  const wishlistOnly = ownedCopies.length === 0 && allCopies.length > 0;
  const lastCopy = dialog?.kind === 'deleteCopy' && allCopies.length === 1;

  // Progress has its own routes, which viewers may be allowed to use.
  const setStatus = useMutation({
    mutationFn: (play_status: string | null) =>
      call(api.PUT('/api/games/{id}/play-status', { params: { path: { id: game.id } }, body: { play_status } })),
    onSuccess: (updated) => saved(updated),
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const logToday = useMutation({
    mutationFn: (copy_id: number | null) =>
      call(
        api.POST('/api/games/{id}/sessions', {
          params: { path: { id: game.id } },
          body: { played_on: today(), copy_id },
        }),
      ),
    onSuccess: (updated) => saved(updated),
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const saving = setStatus.isPending || logToday.isPending;

  const removeGame = useMutation({
    mutationFn: () => call(api.DELETE('/api/games/{id}', { params: { path: { id: game.id } } })),
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: queryKeys.game(game.id) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      notifySuccess(t('common.deleted'));
      void navigate({ to: '/' });
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const removeCopy = useMutation({
    mutationFn: (copyId: number) => call(api.DELETE('/api/copies/{id}', { params: { path: { id: copyId } } })),
    onSuccess: (updated) => {
      saved(updated, t('common.deleted'));
      close();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const removeRelease = useMutation({
    mutationFn: (releaseId: number) =>
      call(api.DELETE('/api/releases/{id}', { params: { path: { id: releaseId } } })),
    onSuccess: (updated) => {
      saved(updated, t('common.deleted'));
      close();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const copyLabel = (copy: CopyDetail, release: ReleaseDetail) =>
    [
      names.platform.get(release.platform_id),
      t(`format.${copy.format}`),
      copy.device_id ? names.device.get(copy.device_id) : null,
    ]
      .filter(Boolean)
      .join(', ');
  const copyNames = new Map(ownedCopies.map(({ copy, release }) => [copy.id, copyLabel(copy, release)]));
  const lastSession = game.sessions[0];
  const playedToday = lastSession?.played_on === today();

  // The release title only says something when it differs from the game's.
  const regionalTitle = (release: ReleaseDetail) => (release.title === game.title ? null : release.title);

  // IGDB has no multiplayer data for single player games, only the mode.
  const singlePlayerOnly =
    game.local_max_players == null &&
    game.online_max_players == null &&
    !game.local_coop &&
    !game.online_coop &&
    game.game_mode_ids.length > 0 &&
    game.game_mode_ids.every((id) => gameModes.data?.find((mode) => mode.id === id)?.slug === 'single-player');

  return (
    <Stack gap="section" className={classes.page}>
      <BackLink />
      <Stack gap="card">
        <section className={classes.hero}>
          <Group className={classes.identity} align="flex-start" wrap="nowrap">
            <CoverPicker
              target={{ kind: 'game', id: game.id }}
              path={game.cover_path}
              uploaded={game.cover_uploaded}
              title={game.title}
              width={phone ? 92 : 160}
              readOnly={!canManage}
            />
            <Stack className={classes.identityText} gap={4}>
              <Title order={1} className={classes.gameTitle}>{game.title}</Title>
              <Group gap={6}>
                {wishlistOnly && (
                  <Badge size="xs" variant="light" color="pink" w="fit-content">
                    {t('game.onWishlist')}
                  </Badge>
                )}
                {game.igdb_id && (
                  <Badge size="xs" variant="light" w="fit-content">
                    {t('game.igdbLinked')}
                  </Badge>
                )}
              </Group>
              {game.first_release_date && (
                <Text size="sm" c="dimmed">
                  {t('game.firstRelease')}: {formatDate(game.first_release_date, i18n.language)}
                </Text>
              )}
              {game.developers.length > 0 && (
                <Text size="sm" c="dimmed">
                  {t('game.developers')}: {game.developers.join(', ')}
                </Text>
              )}
              {game.publishers.length > 0 && (
                <Text size="sm" c="dimmed">
                  {t('game.publishers')}: {game.publishers.join(', ')}
                </Text>
              )}
              <Text size="xs" c="dimmed" data-testid="game-history">
                {history(game, t, i18n.language)}
              </Text>
            </Stack>
          </Group>
          {canManage && (
            <Group className={classes.gameActions} gap="xs">
              {status.data?.igdb_configured && game.igdb_id && (
                <Button
                  variant="default"
                  leftSection={<IconRefresh size={16} />}
                  loading={refresh.isPending}
                  onClick={() => game.igdb_id && refresh.mutate({ igdbId: game.igdb_id, overwrite: false })}
                >
                  {t('igdb.refresh')}
                </Button>
              )}
              {status.data?.igdb_configured && (
                <Button variant="default" leftSection={<IconLink size={16} />} onClick={() => setDialog({ kind: 'linkIgdb' })}>
                  {game.igdb_id ? t('igdb.relink') : t('igdb.link')}
                </Button>
              )}
              <Group className={classes.manageActions} gap="xs" wrap="nowrap">
                <Button variant="default" leftSection={<IconPencil size={16} />} onClick={() => setDialog({ kind: 'editGame' })}>
                  {t('common.edit')}
                </Button>
                <ActionIcon
                  variant="subtle"
                  color="red"
                  size="lg"
                  aria-label={t('game.delete')}
                  onClick={() => setDialog({ kind: 'deleteGame' })}
                >
                  <IconTrash size={18} />
                </ActionIcon>
              </Group>
            </Group>
          )}
        </section>

        <Card withBorder className={classes.metadata}>
          <Stack gap="sm">
            <Title order={4}>{t('game.progressTitle')}</Title>
            <Group className={classes.playControls} align="flex-end" gap="md">
              <Select
                label={t('game.playStatus')}
                placeholder={t('common.notSet')}
                className={classes.playSelect}
                data={statuses.options}
                value={game.play_status ?? null}
                onChange={(value) => setStatus.mutate(value ?? null)}
                clearable={canTrack}
                readOnly={!canTrack}
                rightSection={canTrack ? undefined : null}
                disabled={saving}
                leftSection={
                  game.play_status ? <Badge size="xs" circle color={statuses.color(game.play_status)} /> : null
                }
              />
              {canTrack && (
                <Button
                  variant="light"
                  leftSection={<IconCalendarCheck size={16} />}
                  onClick={() => logToday.mutate(defaultCopy(game, copyNames))}
                  loading={saving}
                  disabled={playedToday}
                >
                  {playedToday ? t('game.loggedToday') : t('game.playedToday')}
                </Button>
              )}
            </Group>
            <div className={classes.lastPlayed} data-testid="last-played">
              <Text size="xs" c="dimmed">{t('game.lastPlayed')}</Text>
              <Text size="sm" fw={600}>{lastSession ? formatDate(lastSession.played_on, i18n.language) : t('game.never')}</Text>
            </div>
            <Sessions game={game} copies={copyNames} canTrack={canTrack} />
          </Stack>
        </Card>

        <Card withBorder className={classes.metadata}>
          <Stack gap="sm">
            <Title order={4}>{t('game.detailsTitle')}</Title>
            <Group gap="xs" className={classes.tagRow}>
              <Text size="sm"><strong>{t('game.gameModes')}:</strong></Text>
              {game.game_mode_ids.length === 0 ? (
                <Text size="sm" c="dimmed">{t('common.notSet')}</Text>
              ) : game.game_mode_ids.map((modeId) => (
                <Badge
                  key={modeId}
                  size="sm"
                  variant="light"
                  tt="none"
                  className={classes.tagLink}
                  renderRoot={(props) => <Link to="/" search={{ game_mode: [modeId] }} {...props} />}
                >
                  {names.mode.get(modeId)}
                </Badge>
              ))}
            </Group>
            {game.genre_ids.length > 0 && (
              <Group gap="xs" className={classes.tagRow}>
                <Text size="sm"><strong>{t('game.genres')}:</strong></Text>
                {game.genre_ids.map((genreId) => (
                  <Badge
                    key={genreId}
                    size="sm"
                    variant="outline"
                    tt="none"
                    className={classes.tagLink}
                    renderRoot={(props) => <Link to="/" search={{ genre: [genreId] }} {...props} />}
                  >
                    {names.genre.get(genreId)}
                  </Badge>
                ))}
              </Group>
            )}
            <Text size="sm">
              <strong>{t('game.players')}:</strong> {singlePlayerOnly ? t('game.singlePlayer') : describePlayers(t, game)}
            </Text>
            {game.names.length > 0 && (
              <Text size="sm" data-testid="names">
                <strong>{t('game.alsoKnownAs')}:</strong>{' '}
                {game.names
                  .map((n) => (n.comment ? `${n.name} (${n.comment})` : n.name))
                  .join(', ')}
              </Text>
            )}
            {game.versions.length > 0 && (
              <Text size="sm" data-testid="versions">
                <strong>{t('game.otherVersions')}:</strong>{' '}
                {game.versions.map((version, i) => (
                  <span key={version.game_id}>
                    {i > 0 && ', '}
                    <Anchor
                      inherit
                      renderRoot={(props) => (
                        <Link to="/games/$gameId" params={{ gameId: String(version.game_id) }} {...props} />
                      )}
                    >
                      {version.title}
                    </Anchor>{' '}
                    <Text span c="dimmed" inherit>
                      ({t(`versionRelation.${version.relation}`)})
                    </Text>
                  </span>
                ))}
              </Text>
            )}
            {game.summary?.trim() && (
              <Text size="sm" className={classes.summaryText}>
                <strong>{t('game.summary')}:</strong> {game.summary}
              </Text>
            )}
          </Stack>
        </Card>
      </Stack>

      <Stack gap="card">
        <Group className={classes.releasesHeading} justify="space-between">
          <SectionHeading title={t('game.releasesTitle')} description={canManage ? t('game.releasesDescription') : undefined} />
          {canManage && (
            <Button leftSection={<IconPlus size={16} />} onClick={() => setDialog({ kind: 'addCopy' })}>
              {t('game.addCopy')}
            </Button>
          )}
        </Group>

        {game.releases.length === 0 && <Text c="dimmed">{t('game.noReleases')}</Text>}

        {game.releases.map((release) => (
          <Card key={release.id} withBorder className={classes.release} data-testid={`release-${release.id}`}>
            <Stack gap="xs">
              <Group className={classes.releaseHeader} justify="space-between" wrap="nowrap" align="flex-start">
                <Group className={classes.releaseIdentity} gap="sm" wrap="nowrap" align="flex-start">
                  <CoverPicker
                    target={{ kind: 'release', id: release.id }}
                    path={release.cover_path ?? game.cover_path}
                    uploaded={release.cover_uploaded}
                    title={release.title ?? game.title}
                    width={48}
                    readOnly={!canManage}
                  />
                  <Stack gap={0}>
                    <Text fw={600}>
                      {t('game.releaseHeading', {
                        platform: names.platform.get(release.platform_id) ?? '',
                        region: t(`region.${release.region}`),
                      })}
                    </Text>
                    {(regionalTitle(release) || release.product_code) && (
                      <Text size="sm" c="dimmed">
                        {[regionalTitle(release), release.product_code].filter(Boolean).join(', ')}
                      </Text>
                    )}
                    {hasPlayerData(release) && (
                      <Text size="xs" c="dimmed">
                        {t('game.players')}: {describePlayers(t, effectiveCounts(release, game))}
                      </Text>
                    )}
                  </Stack>
                </Group>
                {!canManage ? null : phone ? (
                  <Menu position="bottom-end">
                    <Menu.Target>
                      <Button variant="default" size="xs" className={classes.releaseMenuButton} rightSection={<IconChevronDown size={14} />}>
                        {t('game.manageRelease')}
                      </Button>
                    </Menu.Target>
                    <Menu.Dropdown>
                      <Menu.Item leftSection={<IconPlus size={16} />} onClick={() => setDialog({ kind: 'addCopy', release })}>
                        {t('game.addCopyHere')}
                      </Menu.Item>
                      <Menu.Item leftSection={<IconPencil size={16} />} onClick={() => setDialog({ kind: 'editRelease', release })}>
                        {t('game.editRelease')}
                      </Menu.Item>
                      <Menu.Item color="red" leftSection={<IconTrash size={16} />} onClick={() => setDialog({ kind: 'deleteRelease', release })}>
                        {t('game.deleteRelease')}
                      </Menu.Item>
                    </Menu.Dropdown>
                  </Menu>
                ) : (
                  <Group className={classes.releaseActions} gap={4} wrap="nowrap">
                    <Tooltip label={t('game.addCopyHere')}>
                      <ActionIcon
                        variant="subtle"
                        aria-label={t('game.addCopyHere')}
                        onClick={() => setDialog({ kind: 'addCopy', release })}
                      >
                        <IconPlus size={16} />
                      </ActionIcon>
                    </Tooltip>
                    <ActionIcon
                      variant="subtle"
                      aria-label={t('game.editRelease')}
                      onClick={() => setDialog({ kind: 'editRelease', release })}
                    >
                      <IconPencil size={16} />
                    </ActionIcon>
                    <ActionIcon
                      variant="subtle"
                      color="red"
                      aria-label={t('game.deleteRelease')}
                      onClick={() => setDialog({ kind: 'deleteRelease', release })}
                    >
                      <IconTrash size={16} />
                    </ActionIcon>
                  </Group>
                )}
              </Group>

              {release.copies.length > 0 && (
                <Copies
                  copies={release.copies}
                  devices={names.device}
                  stores={names.store}
                  onEdit={canManage ? (copy) => setDialog({ kind: 'editCopy', copy, release }) : undefined}
                  onDelete={canManage ? (copy) => setDialog({ kind: 'deleteCopy', copy }) : undefined}
                  onGotIt={canManage ? (copy) => setDialog({ kind: 'gotIt', copy, release }) : undefined}
                  onAddContents={canManage ? (copy) => setDialog({ kind: 'contents', copy }) : undefined}
                  onLinkTag={
                    canManage && status.data?.tags === 'connected' ? (copy) => setDialog({ kind: 'linkTag', copy }) : undefined
                  }
                />
              )}
            </Stack>
          </Card>
        ))}
      </Stack>

      {dialog?.kind === 'editGame' && <GameForm game={game} onClose={close} />}
      {dialog?.kind === 'linkIgdb' && <IgdbLinkModal game={game} onClose={close} />}
      {dialog?.kind === 'addCopy' && <CopyForm gameId={game.id} release={dialog.release} onClose={close} />}
      {dialog?.kind === 'editCopy' && (
        <CopyForm gameId={game.id} copy={dialog.copy} release={dialog.release} onClose={close} />
      )}
      {dialog?.kind === 'gotIt' && (
        <CopyForm
          gameId={game.id}
          copy={dialog.copy}
          release={dialog.release}
          status="owned"
          title={t('game.gotItTitle')}
          onClose={close}
        />
      )}
      {dialog?.kind === 'editRelease' && <ReleaseForm release={dialog.release} onClose={close} />}

      <ConfirmModal
        opened={dialog?.kind === 'deleteGame'}
        title={t('game.deleteTitle', { title: game.title })}
        message={copyCount(game) ? t('game.deleteMessage', { count: copyCount(game) }) : undefined}
        confirmLabel={t('common.delete')}
        busy={removeGame.isPending}
        onConfirm={() => removeGame.mutate()}
        onClose={close}
      />
      {dialog?.kind === 'contents' && <BundleContentsModal game={game} copy={dialog.copy} onClose={close} />}
      {dialog?.kind === 'linkTag' && <TagLinkModal gameId={game.id} copy={dialog.copy} onClose={close} />}
      <ConfirmModal
        opened={dialog?.kind === 'deleteCopy'}
        title={dialog?.kind === 'deleteCopy' && dialog.copy.bundle ? t('bundle.removeTitle') : t('game.deleteCopy')}
        message={lastCopy ? t('game.lastCopy', { title: game.title }) : undefined}
        confirmLabel={t('common.delete')}
        busy={removeCopy.isPending}
        onConfirm={() => dialog?.kind === 'deleteCopy' && removeCopy.mutate(dialog.copy.id)}
        onClose={close}
        extraAction={
          lastCopy && (
            <Button
              variant="outline"
              color="red"
              loading={removeGame.isPending}
              disabled={removeCopy.isPending}
              onClick={() => removeGame.mutate()}
            >
              {t('game.deleteGameToo')}
            </Button>
          )
        }
      />
      <ConfirmModal
        opened={dialog?.kind === 'deleteRelease'}
        title={t('game.deleteRelease')}
        message={
          dialog?.kind === 'deleteRelease' && dialog.release.copies.length
            ? t('game.deleteReleaseCopies', { count: dialog.release.copies.length })
            : undefined
        }
        confirmLabel={t('common.delete')}
        busy={removeRelease.isPending}
        onConfirm={() => dialog?.kind === 'deleteRelease' && removeRelease.mutate(dialog.release.id)}
        onClose={close}
      />
    </Stack>
  );
}
