import {
  Badge,
  Button,
  Card,
  Group,
  Loader,
  Select,
  SimpleGrid,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core';
import { useForm } from '@mantine/form';
import { useDebouncedCallback, useDisclosure } from '@mantine/hooks';
import { IconAdjustments, IconPlus, IconSearch } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { getRouteApi, Link, useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useGames, usePlatforms } from '../../api/queries';
import { type GameSummary, PLAY_STATUSES, type Platform, type PlayStatus } from '../../api/types';
import { FormModal } from '../../components/FormModal';
import { STATUS_COLORS } from '../../components/status';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { FiltersDrawer } from './FiltersDrawer';
import { activeFilterCount, type LibrarySearch, toApiQuery } from './search';

const route = getRouteApi('/');

export function LibraryPage() {
  const { t } = useTranslation();
  const search = route.useSearch();
  const navigate = route.useNavigate();
  const [filtersOpen, filters] = useDisclosure();
  const [adding, setAdding] = useState(false);
  const [text, setText] = useState(search.q ?? '');

  const games = useGames(toApiQuery(search));
  const platforms = usePlatforms();
  const platformsById = useMemo(
    () => new Map((platforms.data ?? []).map((p) => [p.id, p])),
    [platforms.data],
  );

  const setSearch = (patch: Partial<LibrarySearch>) =>
    void navigate({ to: '/', search: (prev: LibrarySearch) => ({ ...prev, ...patch }), replace: true });
  const setQuery = useDebouncedCallback((value: string) => setSearch({ q: value.trim() || undefined }), 300);

  const filterCount = activeFilterCount(search);
  const filtered = filterCount > 0 || !!search.q;

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={2}>{t('nav.library')}</Title>
        <Button leftSection={<IconPlus size={16} />} onClick={() => setAdding(true)}>
          {t('library.addGame')}
        </Button>
      </Group>

      <Group gap="sm" wrap="nowrap">
        <TextInput
          flex={1}
          leftSection={<IconSearch size={16} />}
          placeholder={t('library.search')}
          aria-label={t('library.search')}
          value={text}
          onChange={(event) => {
            setText(event.currentTarget.value);
            setQuery(event.currentTarget.value);
          }}
        />
        <Button variant="default" leftSection={<IconAdjustments size={16} />} onClick={filters.open}>
          {filterCount ? t('library.filtersCount', { count: filterCount }) : t('library.filters')}
        </Button>
      </Group>

      {games.isPending ? (
        <Loader size="sm" />
      ) : games.isError ? (
        <Text c="red">{errorMessage(t, games.error)}</Text>
      ) : games.data.length === 0 ? (
        <Text c="dimmed">{filtered ? t('library.noMatch') : t('library.empty')}</Text>
      ) : (
        <>
          <Text size="sm" c="dimmed" data-testid="library-count">
            {t('library.count', { count: games.data.length })}
          </Text>
          <SimpleGrid cols={{ base: 1, xs: 2, md: 3, xl: 4 }}>
            {games.data.map((game) => (
              <GameCard key={game.id} game={game} platformsById={platformsById} />
            ))}
          </SimpleGrid>
        </>
      )}

      <FiltersDrawer
        opened={filtersOpen}
        search={search}
        onChange={setSearch}
        onClear={() => {
          setText('');
          void navigate({ to: '/', search: {}, replace: true });
        }}
        onClose={filters.close}
      />
      {adding && <AddGameModal onClose={() => setAdding(false)} />}
    </Stack>
  );
}

function GameCard({ game, platformsById }: { game: GameSummary; platformsById: Map<number, Platform> }) {
  const { t } = useTranslation();
  return (
    <Card
      withBorder
      padding="md"
      data-testid={`game-${game.title}`}
      renderRoot={(props) => <Link to="/games/$gameId" params={{ gameId: String(game.id) }} {...props} />}
    >
      <Stack gap="xs">
        <Text fw={600} lineClamp={2}>
          {game.title}
        </Text>
        <Group gap={4}>
          {game.play_status && (
            <Badge size="sm" color={STATUS_COLORS[game.play_status]} variant="light">
              {t(`playStatus.${game.play_status}`)}
            </Badge>
          )}
          {game.platforms.map(({ platform_id, copies }) => {
            const platform = platformsById.get(platform_id);
            const label = platform?.abbreviation ?? platform?.name ?? '';
            return (
              <Badge key={platform_id} size="sm" variant="outline" tt="none">
                {copies > 1 ? `${label} ×${copies}` : label}
              </Badge>
            );
          })}
        </Group>
        <Text size="xs" c="dimmed">
          {game.owned_copies ? t('library.copies', { count: game.owned_copies }) : t('library.noCopies')}
        </Text>
      </Stack>
    </Card>
  );
}

function AddGameModal({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const form = useForm({
    initialValues: { title: '', play_status: '' },
    validate: { title: (value) => (value.trim() ? null : t('errors.required')) },
  });
  const onApiError = useApiErrorHandler(form);

  const create = useMutation({
    mutationFn: (values: { title: string; play_status: string }) =>
      call(
        api.POST('/api/games', {
          body: {
            title: values.title,
            play_status: (values.play_status || null) as PlayStatus | null,
          },
        }),
      ),
    onSuccess: (game) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      queryClient.setQueryData(queryKeys.game(game.id), game);
      onClose();
      void navigate({ to: '/games/$gameId', params: { gameId: String(game.id) } });
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={t('library.addGame')}
      saving={create.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => create.mutate(values))}
    >
      <TextInput label={t('game.title')} required data-autofocus {...form.getInputProps('title')} />
      <Select
        label={t('game.playStatus')}
        data={PLAY_STATUSES.map((s) => ({ value: s, label: t(`playStatus.${s}`) }))}
        clearable
        {...form.getInputProps('play_status')}
      />
    </FormModal>
  );
}
