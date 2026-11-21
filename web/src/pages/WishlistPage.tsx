import { ActionIcon, Badge, Button, Group, Loader, ScrollArea, Stack, Text, TextInput, Title, Tooltip } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconCheck, IconHeart, IconPlus, IconSearch, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { queryKeys, useCanManage, useCopies, useGame, useGames, usePlatforms, useStatus, useStores } from '../api/queries';
import type { CopyRow } from '../api/types';
import { ConfirmModal } from '../components/ConfirmModal';
import { Cover } from '../components/Cover';
import { FormModal } from '../components/FormModal';
import { PageHeader } from '../components/PageHeader';
import { notifyError, notifySuccess } from '../components/notify';
import { CopyForm, useGameSaved } from './game/forms';
import { IgdbHits, type IgdbHit, useIgdbSearch } from './game/IgdbLink';
import classes from './WishlistPage.module.css';

/** Where a new wishlist copy goes: a game in the collection, or one to create. */
type Target = { kind: 'existing'; id: number; title: string } | { kind: 'new'; title: string; igdbId: number | null };

type Dialog =
  | { kind: 'pickGame' }
  | { kind: 'addCopy'; target: Target }
  | { kind: 'gotIt'; row: CopyRow }
  | { kind: 'remove'; row: CopyRow }
  | null;

const WISHLIST = { ownership: 'wanted', sort: 'added' } as const;

export function WishlistPage() {
  const { t } = useTranslation();
  const canManage = useCanManage();
  const copies = useCopies(WISHLIST);
  const platforms = usePlatforms();
  const stores = useStores();
  const [dialog, setDialog] = useState<Dialog>(null);
  const close = () => setDialog(null);

  const platformName = useMemo(() => new Map((platforms.data ?? []).map((p) => [p.id, p.name])), [platforms.data]);
  const storeName = useMemo(() => new Map((stores.data ?? []).map((s) => [s.id, s.name])), [stores.data]);

  return (
    <Stack gap={24}>
      <PageHeader
        title={t('nav.wishlist')}
        description={t('wishlist.description')}
        count={copies.data && t('wishlist.count', { count: copies.data.length })}
        countTestId="wishlist-count"
      >
        {canManage && (
          <Button leftSection={<IconPlus size={18} />} onClick={() => setDialog({ kind: 'pickGame' })}>
            {t('wishlist.add')}
          </Button>
        )}
      </PageHeader>

      {copies.isPending ? (
        <Loader size="sm" />
      ) : copies.isError ? (
        <Text c="red" role="alert">
          {errorMessage(t, copies.error)}
        </Text>
      ) : copies.data.length === 0 ? (
        <Stack align="center" className={classes.empty}>
          <IconHeart size={48} stroke={1.2} />
          <Title order={3}>{t('wishlist.empty')}</Title>
          <Text c="dimmed" ta="center">
            {canManage ? t('wishlist.emptyHint') : t('wishlist.emptyViewer')}
          </Text>
        </Stack>
      ) : (
        <ul className={classes.list}>
          {copies.data.map((row) => {
            const details = [
              platformName.get(row.platform_id),
              t(`region.${row.region}`),
              t(`format.${row.format}`),
              row.condition ? t(`condition.${row.condition}`) : null,
            ].filter(Boolean);
            const store = row.store_id ? storeName.get(row.store_id) : null;
            return (
              <li key={row.copy_id} className={classes.item} data-testid={`wish-${row.copy_id}`}>
                <Link to="/games/$gameId" params={{ gameId: String(row.game_id) }} className={classes.cover} tabIndex={-1}>
                  <Cover path={row.cover_path} title={row.title} />
                </Link>
                <div className={classes.details}>
                  <Link to="/games/$gameId" params={{ gameId: String(row.game_id) }} className={classes.title}>
                    {row.title}
                  </Link>
                  <Text size="sm">{details.join(t('common.separator'))}</Text>
                  {store && (
                    <Text size="xs" c="dimmed">
                      {t('wishlist.store', { store })}
                    </Text>
                  )}
                  {row.notes && (
                    <Text size="xs" c="dimmed" lineClamp={2}>
                      {row.notes}
                    </Text>
                  )}
                </div>
                {canManage && (
                  <Group gap={4} wrap="nowrap" className={classes.actions}>
                    <Button
                      size="xs"
                      variant="light"
                      color="green"
                      leftSection={<IconCheck size={16} />}
                      onClick={() => setDialog({ kind: 'gotIt', row })}
                    >
                      {t('game.gotIt')}
                    </Button>
                    <Tooltip label={t('wishlist.remove')}>
                      <ActionIcon
                        variant="subtle"
                        color="red"
                        aria-label={t('wishlist.remove')}
                        onClick={() => setDialog({ kind: 'remove', row })}
                      >
                        <IconTrash size={16} />
                      </ActionIcon>
                    </Tooltip>
                  </Group>
                )}
              </li>
            );
          })}
        </ul>
      )}

      {dialog?.kind === 'pickGame' && (
        <PickGameModal onClose={close} onNext={(target) => setDialog({ kind: 'addCopy', target })} />
      )}
      {dialog?.kind === 'addCopy' && <AddWishCopy target={dialog.target} onClose={close} />}
      {dialog?.kind === 'gotIt' && (
        <CopyForm
          gameId={dialog.row.game_id}
          copy={{
            id: dialog.row.copy_id,
            status: dialog.row.status,
            format: dialog.row.format,
            device_id: dialog.row.device_id,
            store_id: dialog.row.store_id,
            condition: dialog.row.condition,
            notes: dialog.row.notes,
          }}
          release={dialog.row}
          status="owned"
          title={t('game.gotItTitle')}
          onClose={close}
        />
      )}
      {dialog?.kind === 'remove' && <RemoveWish row={dialog.row} onClose={close} />}
    </Stack>
  );
}

/** Step one of adding to the wishlist: which game, from IGDB, the collection or a new title. */
function PickGameModal({ onClose, onNext }: { onClose: () => void; onNext: (target: Target) => void }) {
  const { t } = useTranslation();
  const igdb = useStatus().data?.igdb_configured ?? false;
  const games = useGames({ ownership: 'any' });
  const { results, searched, search } = useIgdbSearch();
  const [picked, setPicked] = useState<IgdbHit | null>(null);
  const form = useForm({
    initialValues: { title: '' },
    validate: { title: (value) => (value.trim() ? null : t('errors.required')) },
  });

  const typed = form.values.title.trim().toLocaleLowerCase();
  const existing = picked
    ? null
    : (games.data ?? []).find((game) => game.title.trim().toLocaleLowerCase() === typed);

  const target = (title: string): Target => {
    if (picked?.game_id != null) return { kind: 'existing', id: picked.game_id, title: picked.name };
    if (existing) return { kind: 'existing', id: existing.id, title: existing.title };
    return { kind: 'new', title: title.trim(), igdbId: picked?.igdb_id ?? null };
  };

  const pick = (hit: IgdbHit) => {
    if (picked?.igdb_id === hit.igdb_id) {
      setPicked(null);
      return;
    }
    setPicked(hit);
    form.setFieldValue('title', hit.name);
  };

  return (
    <FormModal
      opened
      title={t('wishlist.add')}
      saving={false}
      submitLabel={t('wishlist.next')}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => onNext(target(values.title)))}
    >
      <TextInput
        label={t('game.title')}
        description={igdb ? t('library.addIgdbHint') : undefined}
        required
        data-autofocus
        {...form.getInputProps('title')}
        onChange={(event) => {
          form.setFieldValue('title', event.currentTarget.value);
          setPicked(null);
        }}
      />
      {existing && (
        <Text size="sm" c="dimmed" data-testid="wish-existing">
          {t('wishlist.existing', { title: existing.title })}
        </Text>
      )}
      {igdb && (
        <Button
          variant="light"
          style={{ alignSelf: 'flex-start' }}
          leftSection={<IconSearch size={16} />}
          loading={results.isFetching}
          disabled={!form.values.title.trim()}
          onClick={() => search(form.values.title)}
        >
          {t('igdb.search')}
        </Button>
      )}
      {searched && (
        <ScrollArea.Autosize mah={320} type="auto" offsetScrollbars>
          <Stack gap="xs">
            <IgdbHits
              results={results}
              onPick={pick}
              selectedId={picked?.igdb_id ?? null}
              badge={(hit) =>
                hit.igdb_id === picked?.igdb_id ? (
                  <Badge variant="light">{t('igdb.selected')}</Badge>
                ) : hit.game_id != null ? (
                  <Badge variant="light" color="gray">{t('igdb.inCollection')}</Badge>
                ) : null
              }
            />
          </Stack>
        </ScrollArea.Autosize>
      )}
    </FormModal>
  );
}

/** Step two: the copy you want. A new game is created only when the copy is saved. */
function AddWishCopy({ target, onClose }: { target: Target; onClose: () => void }) {
  const { t } = useTranslation();
  const created = useRef<number | null>(null);

  const resolveGame = async () => {
    if (target.kind === 'existing') return target.id;
    if (created.current != null) return created.current;
    const game = await call(api.POST('/api/games', { body: { title: target.title, play_status: null } }));
    created.current = game.id;
    if (target.igdbId != null) {
      // The game exists either way; a failed link leaves it to be linked from its page.
      try {
        await call(
          api.POST('/api/games/{id}/igdb', {
            params: { path: { id: game.id } },
            body: { igdb_id: target.igdbId, overwrite: false },
          }),
        );
      } catch (error) {
        notifyError(errorMessage(t, error));
      }
    }
    return game.id;
  };

  return (
    <CopyForm
      resolveGame={resolveGame}
      status="wanted"
      title={t('wishlist.addTitle', { title: target.title })}
      onClose={onClose}
    />
  );
}

/** Removes a wanted copy, offering to delete the game when nothing else is left of it. */
function RemoveWish({ row, onClose }: { row: CopyRow; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const saved = useGameSaved();
  const game = useGame(row.game_id);
  const copies = game.data?.releases.reduce((n, release) => n + release.copies.length, 0) ?? 0;
  const last = copies === 1;

  const removeCopy = useMutation({
    mutationFn: () => call(api.DELETE('/api/copies/{id}', { params: { path: { id: row.copy_id } } })),
    onSuccess: (updated) => {
      saved(updated, t('common.deleted'));
      onClose();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const removeGame = useMutation({
    mutationFn: () => call(api.DELETE('/api/games/{id}', { params: { path: { id: row.game_id } } })),
    onSuccess: () => {
      queryClient.removeQueries({ queryKey: queryKeys.game(row.game_id) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      notifySuccess(t('common.deleted'));
      onClose();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  return (
    <ConfirmModal
      opened
      title={t('wishlist.removeTitle')}
      message={last ? t('game.lastCopy', { title: row.title }) : undefined}
      confirmLabel={t('wishlist.remove')}
      busy={removeCopy.isPending}
      onConfirm={() => removeCopy.mutate()}
      onClose={onClose}
      extraAction={
        last && (
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
  );
}
