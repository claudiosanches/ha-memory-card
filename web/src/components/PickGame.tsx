import { Anchor, Badge, Button, ScrollArea, Stack, Text, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconSearch } from '@tabler/icons-react';
import type { TFunction } from 'i18next';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { useGames, useStatus } from '../api/queries';
import { IgdbHits, type IgdbHit, useIgdbSearch } from '../pages/game/IgdbLink';
import { FormModal } from './FormModal';
import { notifyError } from './notify';

/** A game in the collection, or one to create. */
export type Target = ({ kind: 'existing'; id: number; title: string } | { kind: 'new'; title: string; igdbId: number | null }) & {
  /** IGDB's ids of the platforms the game came out on, when it was picked from IGDB. */
  igdbPlatforms?: number[];
};

/** Creates a new target's game and links it to IGDB when picked from there. */
export async function createGame(target: Extract<Target, { kind: 'new' }>, t: TFunction): Promise<number> {
  const game = await call(api.POST('/api/games', { body: { title: target.title, play_status: null } }));
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
}

/** Which game: one in the collection, an IGDB result or a new title. */
export function PickGameModal({
  title,
  submitLabel,
  initialTitle = '',
  saving = false,
  onClose,
  onNext,
}: {
  title: string;
  submitLabel: string;
  /** Set while `onNext` saves in place, so the picker takes no second submit. */
  saving?: boolean;
  /** Starts the title field with this, such as a name read from a barcode. */
  initialTitle?: string;
  onClose: () => void;
  onNext: (target: Target) => void;
}) {
  const { t } = useTranslation();
  const igdb = useStatus().data?.igdb_configured ?? false;
  const games = useGames({ ownership: 'any' });
  const { results, searched, search } = useIgdbSearch();
  const [picked, setPicked] = useState<IgdbHit | null>(null);
  const form = useForm({
    initialValues: { title: initialTitle },
    validate: { title: (value) => (value.trim() ? null : t('errors.required')) },
  });

  const typed = form.values.title.trim().toLocaleLowerCase();
  const existing = picked
    ? null
    : (games.data ?? []).find((game) => game.title.trim().toLocaleLowerCase() === typed);

  const target = (title: string): Target => {
    const igdbPlatforms = picked?.platform_ids ?? [];
    if (picked?.game_id != null) return { kind: 'existing', id: picked.game_id, title: picked.name, igdbPlatforms };
    if (existing) return { kind: 'existing', id: existing.id, title: existing.title, igdbPlatforms };
    return { kind: 'new', title: title.trim(), igdbId: picked?.igdb_id ?? null, igdbPlatforms };
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
      title={title}
      saving={saving}
      // Until the collection loads, a title in it would read as a new game.
      submitDisabled={!games.isSuccess}
      submitLabel={submitLabel}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => {
        if (!saving && games.isSuccess) onNext(target(values.title));
      })}
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
      {games.isError && (
        <Text size="sm" c="red" role="alert">
          {errorMessage(t, games.error)}{' '}
          <Anchor component="button" type="button" size="sm" onClick={() => void games.refetch()}>
            {t('common.retry')}
          </Anchor>
        </Text>
      )}
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
