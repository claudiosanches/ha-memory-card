import {
  ActionIcon,
  Anchor,
  Badge,
  Button,
  Card,
  Group,
  Loader,
  Select,
  Stack,
  Table,
  Text,
  Title,
  Tooltip,
} from '@mantine/core';
import {
  IconArrowLeft,
  IconCalendarCheck,
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
  PLAY_STATUSES,
  type PlayStatus,
  type ReleaseDetail,
} from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { Cover } from '../../components/Cover';
import { notifyError, notifySuccess } from '../../components/notify';
import { formatDate, STATUS_COLORS, today } from '../../components/status';
import { CopyForm, GameForm, gameInput, ReleaseForm, useGameSaved } from './forms';
import { IgdbLinkModal, useIgdbLink } from './IgdbLink';

const route = getRouteApi('/games/$gameId');

type Dialog =
  | { kind: 'editGame' }
  | { kind: 'linkIgdb' }
  | { kind: 'deleteGame' }
  | { kind: 'addCopy'; release?: ReleaseDetail }
  | { kind: 'editCopy'; copy: CopyDetail; release: ReleaseDetail }
  | { kind: 'deleteCopy'; copy: CopyDetail }
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

  const update = useMutation({
    mutationFn: (patch: Partial<ReturnType<typeof gameInput>>) =>
      call(api.PUT('/api/games/{id}', { params: { path: { id: game.id } }, body: { ...gameInput(game), ...patch } })),
    onSuccess: (updated) => saved(updated),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

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

  const playedToday = () => {
    const copyId = game.last_played_copy_id ?? (allCopies.length === 1 ? allCopies[0]?.copy.id : null) ?? null;
    update.mutate({ last_played_at: today(), last_played_copy_id: copyId });
  };

  const copyLabel = (copy: CopyDetail, release: ReleaseDetail) =>
    [
      names.platform.get(release.platform_id),
      t(`format.${copy.format}`),
      copy.device_id ? names.device.get(copy.device_id) : null,
    ]
      .filter(Boolean)
      .join(', ');

  const players = (local?: number | null, online?: number | null) =>
    t('game.playersSummary', {
      local: local ?? t('game.playersUnknown'),
      online: online ?? t('game.playersUnknown'),
    });

  return (
    <Stack>
      <BackLink />
      <Group justify="space-between" align="flex-start">
        <Group align="flex-start" wrap="nowrap">
          <Cover path={game.cover_path} title={game.title} width={phone ? 72 : 120} />
          <Stack gap={4}>
            <Title order={2}>{game.title}</Title>
            {game.igdb_id && (
              <Badge size="xs" variant="light" w="fit-content">
                {t('game.igdbLinked')}
              </Badge>
            )}
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
          </Stack>
        </Group>
        <Group gap="xs">
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

      <Card withBorder>
        <Stack gap="sm">
          <Group align="flex-end" gap="md">
            <Select
              label={t('game.playStatus')}
              placeholder={t('common.notSet')}
              w={220}
              data={PLAY_STATUSES.map((s) => ({ value: s, label: t(`playStatus.${s}`) }))}
              value={game.play_status ?? null}
              onChange={(value) => update.mutate({ play_status: (value as PlayStatus | null) ?? null })}
              clearable
              disabled={update.isPending}
              leftSection={
                game.play_status ? <Badge size="xs" circle color={STATUS_COLORS[game.play_status]} /> : null
              }
            />
            <Select
              label={t('game.lastPlayedCopy')}
              placeholder={t('common.notSet')}
              w={260}
              data={allCopies.map(({ copy, release }) => ({ value: String(copy.id), label: copyLabel(copy, release) }))}
              value={game.last_played_copy_id ? String(game.last_played_copy_id) : null}
              onChange={(value) => update.mutate({ last_played_copy_id: value ? Number(value) : null })}
              clearable
              disabled={update.isPending || allCopies.length === 0}
            />
            <Button
              variant="light"
              leftSection={<IconCalendarCheck size={16} />}
              onClick={playedToday}
              loading={update.isPending}
            >
              {t('game.playedToday')}
            </Button>
          </Group>
          <Text size="sm" data-testid="last-played">
            {t('game.lastPlayed')}:{' '}
            {game.last_played_at ? formatDate(game.last_played_at, i18n.language) : t('game.never')}
          </Text>
          <Group gap="xs">
            <Text size="sm">{t('game.gameModes')}:</Text>
            {game.game_mode_ids.length === 0 && (
              <Text size="sm" c="dimmed">
                {t('common.notSet')}
              </Text>
            )}
            {game.game_mode_ids.map((modeId) => (
              <Badge key={modeId} size="sm" variant="light" tt="none">
                {names.mode.get(modeId)}
              </Badge>
            ))}
          </Group>
          {game.genre_ids.length > 0 && (
            <Group gap="xs">
              <Text size="sm">{t('game.genres')}:</Text>
              {game.genre_ids.map((genreId) => (
                <Badge key={genreId} size="sm" variant="outline" tt="none">
                  {names.genre.get(genreId)}
                </Badge>
              ))}
            </Group>
          )}
          <Text size="sm">
            {t('game.players')}: {players(game.local_max_players, game.online_max_players)}
          </Text>
          {game.summary && (
            <Text size="sm" c="dimmed">
              {game.summary}
            </Text>
          )}
        </Stack>
      </Card>

      <Group justify="space-between">
        <Title order={4}>{t('game.releasesTitle')}</Title>
        <Button leftSection={<IconPlus size={16} />} onClick={() => setDialog({ kind: 'addCopy' })}>
          {t('game.addCopy')}
        </Button>
      </Group>

      {game.releases.length === 0 && <Text c="dimmed">{t('game.noReleases')}</Text>}

      {game.releases.map((release) => (
        <Card key={release.id} withBorder data-testid={`release-${release.id}`}>
          <Stack gap="xs">
            <Group justify="space-between" wrap="nowrap" align="flex-start">
              <Group gap="sm" wrap="nowrap" align="flex-start">
                {release.cover_path && (
                  <Cover path={release.cover_path} title={release.title ?? game.title} width={48} />
                )}
                <Stack gap={0}>
                  <Text fw={600}>
                    {t('game.releaseHeading', {
                      platform: names.platform.get(release.platform_id) ?? '',
                      region: t(`region.${release.region}`),
                    })}
                  </Text>
                  {(release.title || release.product_code) && (
                    <Text size="sm" c="dimmed">
                      {[release.title, release.product_code].filter(Boolean).join(', ')}
                    </Text>
                  )}
                  {(release.local_max_players || release.online_max_players) && (
                    <Text size="xs" c="dimmed">
                      {t('game.players')}: {players(release.local_max_players, release.online_max_players)}
                    </Text>
                  )}
                </Stack>
              </Group>
              <Group gap={4} wrap="nowrap">
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
            </Group>

            {release.copies.length > 0 && (
              <Table.ScrollContainer minWidth={560}>
                <Table>
                  <Table.Thead>
                    <Table.Tr>
                      <Table.Th>{t('game.format')}</Table.Th>
                      <Table.Th>{t('game.device')}</Table.Th>
                      <Table.Th>{t('game.store')}</Table.Th>
                      <Table.Th>{t('game.condition')}</Table.Th>
                      <Table.Th>{t('common.notes')}</Table.Th>
                      <Table.Th w={80} />
                    </Table.Tr>
                  </Table.Thead>
                  <Table.Tbody>
                    {release.copies.map((copy) => (
                      <Table.Tr key={copy.id} data-testid={`copy-${copy.id}`}>
                        <Table.Td>{t(`format.${copy.format}`)}</Table.Td>
                        <Table.Td>{copy.device_id ? names.device.get(copy.device_id) : null}</Table.Td>
                        <Table.Td>{copy.store_id ? names.store.get(copy.store_id) : null}</Table.Td>
                        <Table.Td>{copy.condition ? t(`condition.${copy.condition}`) : null}</Table.Td>
                        <Table.Td>
                          <Text size="sm" lineClamp={2}>
                            {copy.notes}
                          </Text>
                        </Table.Td>
                        <Table.Td>
                          <Group gap={4} wrap="nowrap">
                            <ActionIcon
                              variant="subtle"
                              aria-label={t('game.editCopy')}
                              onClick={() => setDialog({ kind: 'editCopy', copy, release })}
                            >
                              <IconPencil size={16} />
                            </ActionIcon>
                            <ActionIcon
                              variant="subtle"
                              color="red"
                              aria-label={t('game.deleteCopy')}
                              onClick={() => setDialog({ kind: 'deleteCopy', copy })}
                            >
                              <IconTrash size={16} />
                            </ActionIcon>
                          </Group>
                        </Table.Td>
                      </Table.Tr>
                    ))}
                  </Table.Tbody>
                </Table>
              </Table.ScrollContainer>
            )}
          </Stack>
        </Card>
      ))}

      {dialog?.kind === 'editGame' && <GameForm game={game} onClose={close} />}
      {dialog?.kind === 'linkIgdb' && <IgdbLinkModal game={game} onClose={close} />}
      {dialog?.kind === 'addCopy' && <CopyForm gameId={game.id} release={dialog.release} onClose={close} />}
      {dialog?.kind === 'editCopy' && (
        <CopyForm gameId={game.id} copy={dialog.copy} release={dialog.release} onClose={close} />
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
      <ConfirmModal
        opened={dialog?.kind === 'deleteCopy'}
        title={t('game.deleteCopy')}
        confirmLabel={t('common.delete')}
        busy={removeCopy.isPending}
        onConfirm={() => dialog?.kind === 'deleteCopy' && removeCopy.mutate(dialog.copy.id)}
        onClose={close}
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
