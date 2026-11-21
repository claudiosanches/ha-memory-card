import {
  ActionIcon,
  Badge,
  Button,
  Group,
  Loader,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core';
import { useForm } from '@mantine/form';
import { useDebouncedCallback, useDisclosure, useLocalStorage } from '@mantine/hooks';
import { IconAdjustments, IconDeviceGamepad2, IconLayoutGrid, IconList, IconPlus, IconSearch, IconX } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { getRouteApi, Link, useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useCanManage, useGames, usePlatforms, useStatus } from '../../api/queries';
import type { GameSummary, Platform } from '../../api/types';
import { FilterBar, FilterButton } from '../../components/FilterBar';
import { PageHeader } from '../../components/PageHeader';
import { Cover } from '../../components/Cover';
import { FormModal } from '../../components/FormModal';
import { usePlayStatusList } from '../../components/playStatuses';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { notifyError } from '../../components/notify';
import { IgdbHits, type IgdbHit, useIgdbSearch } from '../game/IgdbLink';
import { FiltersDrawer } from './FiltersDrawer';
import { activeFilterCount, type LibrarySearch, toApiQuery } from './search';
import classes from './LibraryPage.module.css';

const route = getRouteApi('/');

export function LibraryPage() {
  const { t } = useTranslation();
  const search = route.useSearch();
  const navigate = route.useNavigate();
  const canManage = useCanManage();
  const [filtersOpen, filters] = useDisclosure();
  const [adding, setAdding] = useState(false);
  const [view, setView] = useLocalStorage<'grid' | 'list'>({ key: 'memory-card-library-view', defaultValue: 'grid' });
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
  const clearFilters = () => {
    setQuery.cancel();
    setText('');
    void navigate({ to: '/', search: { sort: search.sort, ownership: search.ownership }, replace: true });
  };
  const collectionPlatforms = (platforms.data ?? []).filter((p) => p.game_count > 0 || search.platform?.includes(p.id));

  return (
    <Stack gap={24}>
      <PageHeader
        title={t('nav.library')}
        description={t('library.description')}
        count={games.data && t('library.count', { count: games.data.length })}
        countTestId="library-count"
      >
        {canManage && (
          <Button leftSection={<IconPlus size={18} />} onClick={() => setAdding(true)}>
            {t('library.addGame')}
          </Button>
        )}
      </PageHeader>

      <div className={classes.controls}>
        <div className={classes.toolbar}>
          <TextInput
            className={classes.search}
            size="md"
            leftSection={<IconSearch size={18} />}
            placeholder={t('library.search')}
            aria-label={t('library.search')}
            value={text}
            rightSection={text ? (
              <ActionIcon variant="subtle" color="gray" size="sm" aria-label={t('library.clearSearch')} onClick={() => {
                setQuery.cancel();
                setText('');
                setSearch({ q: undefined });
              }}><IconX size={16} /></ActionIcon>
            ) : null}
            onChange={(event) => {
              setText(event.currentTarget.value);
              setQuery(event.currentTarget.value);
            }}
          />
          <Button
            size="md"
            variant={filterCount ? 'light' : 'default'}
            leftSection={<IconAdjustments size={18} />}
            onClick={filters.open}
          >
            {filterCount ? t('library.filtersCount', { count: filterCount }) : t('library.filters')}
          </Button>
          <Select
            className={classes.sort}
            size="md"
            aria-label={t('library.filter.sort')}
            data={(['title', 'added', 'played'] as const).map((s) => ({ value: s, label: t(`library.sort.${s}`) }))}
            value={search.sort ?? 'title'}
            onChange={(value) => setSearch({ sort: value === 'added' || value === 'played' ? value : undefined })}
            allowDeselect={false}
          />
          <Group gap={4} wrap="nowrap" className={classes.viewToggle} role="group" aria-label={t('library.view')}>
            <ActionIcon size={34} variant={view === 'grid' ? 'light' : 'subtle'} color={view === 'grid' ? undefined : 'gray'} aria-label={t('library.gridView')} title={t('library.gridView')} aria-pressed={view === 'grid'} onClick={() => setView('grid')}>
              <IconLayoutGrid size={19} />
            </ActionIcon>
            <ActionIcon size={34} variant={view === 'list' ? 'light' : 'subtle'} color={view === 'list' ? undefined : 'gray'} aria-label={t('library.listView')} title={t('library.listView')} aria-pressed={view === 'list'} onClick={() => setView('list')}>
              <IconList size={20} />
            </ActionIcon>
          </Group>
        </div>
        {collectionPlatforms.length > 0 && (
          <FilterBar label={t('library.filter.platform')}>
            <FilterButton selected={!search.platform?.length} onClick={() => setSearch({ platform: undefined })}>
              {t('library.allPlatforms')}
            </FilterButton>
            {collectionPlatforms.map((platform) => (
              <FilterButton key={platform.id} selected={search.platform?.includes(platform.id) ?? false} onClick={() => setSearch({ platform: [platform.id] })}>
                {platform.abbreviation ?? platform.name}
              </FilterButton>
            ))}
          </FilterBar>
        )}
      </div>

      {games.isPending ? (
        <div className={classes.empty}><Loader size="sm" /></div>
      ) : games.isError ? (
        <Text c="red" role="alert">{errorMessage(t, games.error)}</Text>
      ) : games.data.length === 0 ? (
        <Stack align="center" className={classes.empty}>
          <IconDeviceGamepad2 size={48} stroke={1.2} />
          <Title order={3}>{filtered ? t('library.noMatchTitle') : t('library.emptyTitle')}</Title>
          <Text c="dimmed" ta="center">{filtered ? t('library.noMatch') : t('library.empty')}</Text>
          {filtered && <Button variant="default" onClick={clearFilters}>{t('library.clearFilters')}</Button>}
        </Stack>
      ) : (
        <div className={view === 'list' ? classes.list : classes.grid}>
          {games.data.map((game) => (
            <GameCard key={game.id} game={game} platformsById={platformsById} />
          ))}
        </div>
      )}

      <FiltersDrawer
        opened={filtersOpen}
        search={search}
        onChange={setSearch}
        onClear={clearFilters}
        onClose={filters.close}
      />
      {adding && <AddGameModal onClose={() => setAdding(false)} />}
    </Stack>
  );
}

function GameCard({ game, platformsById }: { game: GameSummary; platformsById: Map<number, Platform> }) {
  const statuses = usePlayStatusList();
  return (
    <Link
      className={classes.gameCard}
      to="/games/$gameId"
      params={{ gameId: String(game.id) }}
      aria-label={game.title}
      title={game.title}
      data-testid={`game-${game.title}`}
    >
      <div className={classes.artwork}>
        <Cover path={game.cover_path} title={game.title} />
      </div>
      <div className={classes.gameInfo}>
        <Text className={classes.gameTitle} lineClamp={2}>{game.title}</Text>
        <Group gap={6} className={classes.gamePlatforms}>
          {game.platforms.map(({ platform_id, copies }) => {
            const platform = platformsById.get(platform_id);
            const label = platform?.abbreviation ?? platform?.name ?? '';
            return (
              <span key={platform_id} className={classes.platformLabel}>
                {copies > 1 ? `${label} ×${copies}` : label}
              </span>
            );
          })}
          {game.play_status && (
            <Badge size="sm" color={statuses.color(game.play_status)} variant="light" tt="none" className={classes.status}>
              {statuses.name(game.play_status)}
            </Badge>
          )}
        </Group>
      </div>
    </Link>
  );
}

function AddGameModal({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const statuses = usePlayStatusList();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const form = useForm({
    initialValues: { title: '', play_status: '' },
    validate: { title: (value) => (value.trim() ? null : t('errors.required')) },
  });
  const onApiError = useApiErrorHandler(form);
  const igdb = useStatus().data?.igdb_configured ?? false;
  const { results, searched, search } = useIgdbSearch();
  const [picked, setPicked] = useState<IgdbHit | null>(null);

  const open = (id: number) => {
    onClose();
    void navigate({ to: '/games/$gameId', params: { gameId: String(id) } });
  };

  const create = useMutation({
    mutationFn: async (values: { title: string; play_status: string }) => {
      const game = await call(
        api.POST('/api/games', {
          body: {
            title: values.title,
            play_status: values.play_status || null,
          },
        }),
      );
      if (!picked) return { game, linkError: null };
      // The game exists either way; a failed link leaves it to be linked from its page.
      try {
        const linked = await call(
          api.POST('/api/games/{id}/igdb', {
            params: { path: { id: game.id } },
            body: { igdb_id: picked.igdb_id, overwrite: false },
          }),
        );
        return { game: linked, linkError: null };
      } catch (error) {
        return { game, linkError: error };
      }
    },
    onSuccess: ({ game, linkError }) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      void queryClient.invalidateQueries({ queryKey: queryKeys.status });
      queryClient.setQueryData(queryKeys.game(game.id), game);
      if (linkError) notifyError(errorMessage(t, linkError));
      open(game.id);
    },
    onError: onApiError,
  });

  const pick = (hit: IgdbHit) => {
    if (hit.game_id != null) {
      open(hit.game_id);
    } else if (picked?.igdb_id === hit.igdb_id) {
      setPicked(null);
    } else {
      setPicked(hit);
      form.setFieldValue('title', hit.name);
    }
  };

  return (
    <FormModal
      opened
      title={t('library.addGame')}
      saving={create.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => create.mutate(values))}
    >
      <TextInput
        label={t('game.title')}
        description={igdb ? t('library.addIgdbHint') : undefined}
        required
        data-autofocus
        {...form.getInputProps('title')}
      />
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
        <Stack gap="xs" data-testid="igdb-results">
          <IgdbHits
            results={results}
            disabled={create.isPending}
            onPick={pick}
            selectedId={picked?.igdb_id ?? null}
            badge={(hit) =>
              hit.game_id != null ? (
                <Badge variant="light" color="gray">{t('igdb.inCollection')}</Badge>
              ) : hit.igdb_id === picked?.igdb_id ? (
                <Badge variant="light">{t('igdb.selected')}</Badge>
              ) : null
            }
          />
        </Stack>
      )}
      <Select
        label={t('game.playStatus')}
        data={statuses.options}
        clearable
        {...form.getInputProps('play_status')}
      />
    </FormModal>
  );
}
