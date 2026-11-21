import { Badge, Card, Checkbox, Group, Loader, Modal, Stack, Text, TextInput } from '@mantine/core';
import { useDebouncedValue, useMediaQuery } from '@mantine/hooks';
import { IconSearch } from '@tabler/icons-react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import type { GameDetail } from '../../api/types';
import { notifyError } from '../../components/notify';
import { useGameSaved } from './forms';

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

export function IgdbLinkModal({ game, onClose }: { game: GameDetail; onClose: () => void }) {
  const { t } = useTranslation();
  const phone = useMediaQuery('(max-width: 48em)');
  const [text, setText] = useState(game.title);
  const [query] = useDebouncedValue(text.trim(), 400);
  // A new match usually means the data from the old one was wrong.
  const [overwrite, setOverwrite] = useState(game.igdb_id != null);
  const link = useIgdbLink(game, onClose);

  const results = useQuery({
    queryKey: ['igdb-search', query],
    queryFn: () => call(api.GET('/api/igdb/search', { params: { query: { q: query } } })),
    enabled: query.length > 0,
    staleTime: 5 * 60_000,
  });

  return (
    <Modal opened onClose={onClose} title={t('igdb.link')} fullScreen={phone} centered={!phone} size="lg">
      <Stack>
        <TextInput
          label={t('igdb.search')}
          leftSection={<IconSearch size={16} />}
          value={text}
          onChange={(event) => setText(event.currentTarget.value)}
          data-autofocus
        />
        <Checkbox
          label={t('igdb.overwrite')}
          checked={overwrite}
          onChange={(event) => setOverwrite(event.currentTarget.checked)}
        />
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
            disabled={link.isPending}
            onClick={() => link.mutate({ igdbId: hit.igdb_id, overwrite })}
            style={{ textAlign: 'left', cursor: 'pointer' }}
            data-testid={`igdb-hit-${hit.igdb_id}`}
          >
            <Group justify="space-between" wrap="nowrap">
              <Stack gap={2}>
                <Text fw={600}>{hit.name}</Text>
                <Text size="xs" c="dimmed">
                  {t('igdb.hitDetails', { year: hit.year ?? '?', platforms: hit.platforms.join(', ') })}
                </Text>
              </Stack>
              {hit.igdb_id === game.igdb_id && <Badge variant="light">{t('igdb.current')}</Badge>}
            </Group>
          </Card>
        ))}
      </Stack>
    </Modal>
  );
}
