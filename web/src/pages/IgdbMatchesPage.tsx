import { Anchor, Badge, Button, Card, Group, Loader, Radio, Stack, Text } from '@mantine/core';
import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { queryKeys } from '../api/queries';
import type { MatchGroup } from '../api/types';
import { notifyError } from '../components/notify';
import { PageHeader } from '../components/PageHeader';

/** Scores at or above this are picked for you; you still confirm. */
const CONFIDENT = 80;
const PAGE = 5;

/** Unlinked games with IGDB's closest names, to link or skip one by one. */
export function IgdbMatchesPage() {
  const { t } = useTranslation();
  const [handled, setHandled] = useState<Set<number>>(() => new Set());
  const pages = useInfiniteQuery({
    queryKey: ['igdb-matches'],
    queryFn: ({ pageParam }) =>
      call(api.GET('/api/igdb/matches', { params: { query: { after: pageParam, limit: PAGE } } })),
    initialPageParam: 0,
    getNextPageParam: (page) => (page.length < PAGE ? undefined : page.at(-1)?.game_id),
    // Linking changes the list; a refetch would bring back games already handled here.
    staleTime: Infinity,
    gcTime: 0,
  });

  const groups = (pages.data?.pages.flat() ?? []).filter((group) => !handled.has(group.game_id));
  const done = (gameId: number) => setHandled((current) => new Set(current).add(gameId));
  const loading = pages.isFetching;

  return (
    <Stack gap={24}>
      <PageHeader title={t('matches.title')} description={t('matches.intro')} />
      {pages.isError && <Text c="red">{errorMessage(t, pages.error)}</Text>}
      {groups.map((group) => (
        <MatchCard key={group.game_id} group={group} onDone={() => done(group.game_id)} />
      ))}
      {loading && (
        <Group gap="sm">
          <Loader size="sm" />
          <Text size="sm">{t('matches.searching')}</Text>
        </Group>
      )}
      {!loading && pages.hasNextPage && (
        <Button variant="default" w="fit-content" onClick={() => void pages.fetchNextPage()}>
          {t('matches.more')}
        </Button>
      )}
      {!loading && !pages.hasNextPage && groups.length === 0 && !pages.isError && (
        <Text c="dimmed">{t('matches.none')}</Text>
      )}
    </Stack>
  );
}

function MatchCard({ group, onDone }: { group: MatchGroup; onDone: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const top = group.candidates[0];
  const [picked, setPicked] = useState<string | null>(
    top && top.score >= CONFIDENT && top.game_id == null ? String(top.igdb_id) : null,
  );

  const link = useMutation({
    mutationFn: (igdbId: number) =>
      call(
        api.POST('/api/games/{id}/igdb', {
          params: { path: { id: group.game_id } },
          body: { igdb_id: igdbId, overwrite: false },
        }),
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      onDone();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const skip = useMutation({
    mutationFn: () => call(api.POST('/api/games/{id}/igdb-skip', { params: { path: { id: group.game_id } } })),
    onSuccess: onDone,
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  return (
    <Card withBorder data-testid={`match-${group.game_id}`}>
      <Stack gap="sm">
        <div>
          <Anchor
            fw={600}
            renderRoot={(props) => <Link to="/games/$gameId" params={{ gameId: String(group.game_id) }} {...props} />}
          >
            {group.title}
          </Anchor>
          {group.platforms.length > 0 && (
            <Text size="xs" c="dimmed">
              {group.platforms.join(', ')}
            </Text>
          )}
        </div>
        {group.candidates.length === 0 ? (
          <Text size="sm" c="dimmed">
            {t('matches.noCandidates')}
          </Text>
        ) : (
          <Radio.Group value={picked} onChange={setPicked} aria-label={t('matches.pick', { title: group.title })}>
            <Stack gap="xs">
              {group.candidates.map((candidate) => (
                <Radio
                  key={candidate.igdb_id}
                  value={String(candidate.igdb_id)}
                  disabled={candidate.game_id != null}
                  label={
                    <Group gap={6} wrap="wrap">
                      <Text size="sm">{candidate.name}</Text>
                      <Text size="xs" c="dimmed">
                        {t('igdb.hitDetails', { year: candidate.year ?? '?', platforms: candidate.platforms.join(', ') })}
                      </Text>
                      <Badge size="xs" variant="light" color={candidate.score >= CONFIDENT ? 'green' : 'gray'}>
                        {t('matches.score', { score: candidate.score })}
                      </Badge>
                      {candidate.game_id != null && (
                        <Badge size="xs" variant="light" color="gray">
                          {t('igdb.inCollection')}
                        </Badge>
                      )}
                    </Group>
                  }
                />
              ))}
            </Stack>
          </Radio.Group>
        )}
        <Group gap="xs">
          <Button size="xs" disabled={!picked} loading={link.isPending} onClick={() => picked && link.mutate(Number(picked))}>
            {t('matches.link')}
          </Button>
          <Button size="xs" variant="default" onClick={onDone}>
            {t('matches.later')}
          </Button>
          <Button size="xs" variant="subtle" color="gray" loading={skip.isPending} onClick={() => skip.mutate()}>
            {t('matches.notOnIgdb')}
          </Button>
        </Group>
      </Stack>
    </Card>
  );
}
