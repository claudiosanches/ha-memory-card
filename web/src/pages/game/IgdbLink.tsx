import { Badge, Button, Card, Checkbox, Group, Loader, Modal, Stack, Text, TextInput } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { IconSearch } from '@tabler/icons-react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { type ReactNode, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import type { components } from '../../api/schema';
import type { GameDetail } from '../../api/types';
import { notifyError } from '../../components/notify';
import { useGameSaved } from './forms';

export type IgdbHit = components['schemas']['SearchHit'];

type LinkRequest = { igdbId: number; overwrite: boolean };

/** Links a game to IGDB, or refreshes it when `igdbId` is its current link. */
export function useIgdbLink(game: GameDetail, onDone?: () => void) {
  const { t } = useTranslation();
  const saved = useGameSaved();
  return useMutation({
    mutationFn: ({ igdbId, overwrite }: LinkRequest) =>
      call(
        api.POST('/api/games/{id}/igdb', {
          params: { path: { id: game.id } },
          body: { igdb_id: igdbId, overwrite },
        }),
      ),
    onSuccess: (updated, { igdbId }) => {
      saved(updated, igdbId === game.igdb_id ? t('igdb.refreshed') : t('igdb.linked'));
      onDone?.();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });
}

/**
 * Searches IGDB only when asked, never while typing, since every search
 * counts against the IGDB rate limit.
 */
export function useIgdbSearch(initial = '') {
  const [term, setTerm] = useState(initial.trim());
  const results = useQuery({
    queryKey: ['igdb-search', term],
    queryFn: () => call(api.GET('/api/igdb/search', { params: { query: { q: term } } })),
    enabled: term.length > 0,
    staleTime: 5 * 60_000,
  });
  return { results, searched: term.length > 0, search: (text: string) => setTerm(text.trim()) };
}

type HitsProps = {
  results: ReturnType<typeof useIgdbSearch>['results'];
  disabled?: boolean;
  onPick: (hit: IgdbHit) => void;
  /** A badge for a hit, such as "Current" or "In your collection". */
  badge?: (hit: IgdbHit) => ReactNode;
  selectedId?: number | null;
};

export function IgdbHits({ results, disabled, onPick, badge, selectedId }: HitsProps) {
  const { t } = useTranslation();
  return (
    <>
      {results.isFetching && <Loader size="sm" />}
      {results.isError && <Text c="red">{errorMessage(t, results.error)}</Text>}
      {results.data?.length === 0 && <Text c="dimmed">{t('igdb.noResults')}</Text>}
      {results.data?.map((hit) => (
        <Card
          key={hit.igdb_id}
          withBorder
          padding="sm"
          component="button"
          type="button"
          disabled={disabled}
          onClick={() => onPick(hit)}
          aria-pressed={selectedId === undefined ? undefined : selectedId === hit.igdb_id}
          style={{
            textAlign: 'left',
            cursor: 'pointer',
            borderColor: selectedId === hit.igdb_id ? 'var(--mantine-primary-color-filled)' : undefined,
          }}
          data-testid={`igdb-hit-${hit.igdb_id}`}
        >
          <Group justify="space-between" wrap="nowrap">
            <Stack gap={2}>
              <Text fw={600}>{hit.name}</Text>
              <Text size="xs" c="dimmed">
                {t('igdb.hitDetails', { year: hit.year ?? '?', platforms: hit.platforms.join(', ') })}
              </Text>
            </Stack>
            {badge?.(hit)}
          </Group>
        </Card>
      ))}
    </>
  );
}

export function IgdbLinkModal({ game, onClose }: { game: GameDetail; onClose: () => void }) {
  const { t } = useTranslation();
  const phone = useMediaQuery('(max-width: 48em)');
  const [text, setText] = useState(game.title);
  // A new match usually means the data from the old one was wrong.
  const [overwrite, setOverwrite] = useState(game.igdb_id != null);
  const link = useIgdbLink(game, onClose);
  const { results, search } = useIgdbSearch(game.title);

  return (
    <Modal opened onClose={onClose} title={t('igdb.link')} fullScreen={phone} centered={!phone} size="lg">
      <Stack>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            search(text);
          }}
        >
          <Group align="flex-end" wrap="nowrap">
            <TextInput
              label={t('igdb.searchLabel')}
              leftSection={<IconSearch size={16} />}
              value={text}
              onChange={(event) => setText(event.currentTarget.value)}
              style={{ flex: 1 }}
              data-autofocus
            />
            <Button type="submit" variant="light" loading={results.isFetching}>
              {t('igdb.search')}
            </Button>
          </Group>
        </form>
        <Checkbox
          label={t('igdb.overwrite')}
          checked={overwrite}
          onChange={(event) => setOverwrite(event.currentTarget.checked)}
        />
        <IgdbHits
          results={results}
          disabled={link.isPending}
          onPick={(hit) => link.mutate({ igdbId: hit.igdb_id, overwrite })}
          badge={(hit) =>
            hit.igdb_id === game.igdb_id ? (
              <Badge variant="light">{t('igdb.current')}</Badge>
            ) : hit.game_id != null ? (
              <Badge variant="light" color="gray">{t('igdb.inCollection')}</Badge>
            ) : null
          }
        />
      </Stack>
    </Modal>
  );
}
