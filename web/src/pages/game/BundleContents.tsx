import { Button, Checkbox, Loader, Stack, Text } from '@mantine/core';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { useStatus } from '../../api/queries';
import type { CopyDetail, GameDetail } from '../../api/types';
import { FormModal } from '../../components/FormModal';
import { notifyError } from '../../components/notify';
import { createGame, PickGameModal, type Target } from '../../components/PickGame';
import { useGameSaved } from './forms';

type Props = { game: GameDetail; copy: CopyDetail; onClose: () => void };

/** Adds games that came inside a copy: IGDB's list for the bundle when there is one, or any game. */
export function BundleContentsModal({ game, copy, onClose }: Props) {
  const { t } = useTranslation();
  const igdb = useStatus().data?.igdb_configured ?? false;
  const listed = useQuery({
    queryKey: ['igdb-contents', game.id],
    queryFn: () => call(api.GET('/api/games/{id}/igdb-contents', { params: { path: { id: game.id } } })),
    enabled: igdb && game.igdb_id != null,
  });
  const inside = new Set(copy.contents.map((item) => item.game_id));
  const offered = (listed.data ?? []).filter((hit) => hit.game_id !== game.id && !(hit.game_id != null && inside.has(hit.game_id)));
  const [checked, setChecked] = useState<Set<number> | null>(null);
  const picked = checked ?? new Set(offered.map((hit) => hit.igdb_id));
  const [picking, setPicking] = useState(false);
  const saved = useGameSaved();

  const add = useMutation({
    mutationFn: async (targets: Target[]) => {
      let latest: GameDetail | null = null;
      for (const target of targets) {
        const id = target.kind === 'existing' ? target.id : await createGame(target, t);
        latest = await call(
          api.POST('/api/copies/{id}/contents', { params: { path: { id: copy.id } }, body: { game_id: id } }),
        );
      }
      return latest;
    },
    onSuccess: (latest) => {
      if (latest) saved(latest);
      onClose();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const pending = igdb && game.igdb_id != null && listed.isPending;
  if (picking || (!pending && offered.length === 0)) {
    return (
      <PickGameModal
        title={t('bundle.addTitle', { title: game.title })}
        submitLabel={t('bundle.addOne')}
        onClose={onClose}
        onNext={(target) => add.mutate([target])}
      />
    );
  }

  const toggle = (igdbId: number) => {
    const next = new Set(picked);
    if (next.has(igdbId)) next.delete(igdbId);
    else next.add(igdbId);
    setChecked(next);
  };
  const targets = (): Target[] =>
    offered
      .filter((hit) => picked.has(hit.igdb_id))
      .map((hit) =>
        hit.game_id != null
          ? { kind: 'existing', id: hit.game_id, title: hit.name }
          : { kind: 'new', title: hit.name, igdbId: hit.igdb_id },
      );

  return (
    <FormModal
      opened
      title={t('bundle.addTitle', { title: game.title })}
      saving={add.isPending}
      submitLabel={t('bundle.addPicked', { count: picked.size })}
      onClose={onClose}
      onSubmit={(event) => {
        event?.preventDefault();
        if (picked.size > 0) add.mutate(targets());
      }}
      extraAction={
        <Button variant="light" onClick={() => setPicking(true)}>
          {t('bundle.other')}
        </Button>
      }
    >
      {pending ? (
        <Loader size="sm" />
      ) : (
        <Stack gap="xs">
          <Text size="sm" c="dimmed">
            {t('bundle.igdbListed')}
          </Text>
          {offered.map((hit) => (
            <Checkbox
              key={hit.igdb_id}
              label={hit.year ? `${hit.name} (${hit.year})` : hit.name}
              checked={picked.has(hit.igdb_id)}
              onChange={() => toggle(hit.igdb_id)}
            />
          ))}
        </Stack>
      )}
    </FormModal>
  );
}
