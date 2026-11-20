import { Button, Drawer, MultiSelect, NumberInput, SegmentedControl, Select, Stack } from '@mantine/core';
import { useTranslation } from 'react-i18next';
import { useDevices, useGameModes, useGenres, usePlatforms, useStores } from '../../api/queries';
import { FORMATS, type Format, PLAY_STATUSES, type PlayStatus, REGIONS, type Region } from '../../api/types';
import type { LibrarySearch } from './search';

type Props = {
  opened: boolean;
  search: LibrarySearch;
  onChange: (patch: Partial<LibrarySearch>) => void;
  onClear: () => void;
  onClose: () => void;
};

const toIds = (values: string[]) => (values.length ? values.map(Number) : undefined);
const fromIds = (values?: number[]) => (values ?? []).map(String);

export function FiltersDrawer({ opened, search, onChange, onClear, onClose }: Props) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const gameModes = useGameModes();
  const genres = useGenres();

  const platformOptions = (platforms.data ?? [])
    .filter((p) => p.active || p.release_count > 0 || search.platform?.includes(p.id))
    .map((p) => ({ value: String(p.id), label: p.name }));

  return (
    <Drawer opened={opened} onClose={onClose} title={t('library.filters')} position="right" size="sm">
      <Stack>
        <SegmentedControl
          fullWidth
          value={search.ownership ?? 'owned'}
          onChange={(value) => onChange({ ownership: value === 'any' ? 'any' : undefined })}
          data={[
            { value: 'owned', label: t('library.filter.owned') },
            { value: 'any', label: t('library.filter.any') },
          ]}
        />
        <MultiSelect
          label={t('library.filter.playStatus')}
          data={PLAY_STATUSES.map((s) => ({ value: s, label: t(`playStatus.${s}`) }))}
          value={search.play_status ?? []}
          onChange={(values) => onChange({ play_status: values.length ? (values as PlayStatus[]) : undefined })}
          clearable
        />
        <MultiSelect
          label={t('library.filter.platform')}
          data={platformOptions}
          value={fromIds(search.platform)}
          onChange={(values) => onChange({ platform: toIds(values) })}
          searchable
          clearable
        />
        <MultiSelect
          label={t('library.filter.device')}
          data={(devices.data ?? []).map((d) => ({ value: String(d.id), label: d.name }))}
          value={fromIds(search.device)}
          onChange={(values) => onChange({ device: toIds(values) })}
          searchable
          clearable
        />
        <Select
          label={t('library.filter.format')}
          data={[
            { value: '', label: t('library.filter.anyFormat') },
            ...FORMATS.map((f) => ({ value: f, label: t(`format.${f}`) })),
          ]}
          value={search.format ?? ''}
          onChange={(value) => onChange({ format: value ? (value as Format) : undefined })}
          allowDeselect={false}
        />
        <MultiSelect
          label={t('library.filter.region')}
          data={REGIONS.map((r) => ({ value: r, label: t(`region.${r}`) }))}
          value={search.region ?? []}
          onChange={(values) => onChange({ region: values.length ? (values as Region[]) : undefined })}
          clearable
        />
        <MultiSelect
          label={t('library.filter.store')}
          data={(stores.data ?? []).map((s) => ({ value: String(s.id), label: s.name }))}
          value={fromIds(search.store)}
          onChange={(values) => onChange({ store: toIds(values) })}
          searchable
          clearable
        />
        <MultiSelect
          label={t('library.filter.gameMode')}
          data={(gameModes.data ?? []).map((m) => ({ value: String(m.id), label: m.name }))}
          value={fromIds(search.game_mode)}
          onChange={(values) => onChange({ game_mode: toIds(values) })}
          clearable
        />
        {(genres.data?.length ?? 0) > 0 && (
          <MultiSelect
            label={t('library.filter.genre')}
            data={(genres.data ?? []).map((g) => ({ value: String(g.id), label: g.name }))}
            value={fromIds(search.genre)}
            onChange={(values) => onChange({ genre: toIds(values) })}
            searchable
            clearable
          />
        )}
        <NumberInput
          label={t('library.filter.minLocalPlayers')}
          min={1}
          max={99}
          value={search.min_local_players ?? ''}
          onChange={(value) => {
            const players = Number(value);
            onChange({ min_local_players: Number.isInteger(players) && players > 1 ? players : undefined });
          }}
        />
        <Select
          label={t('library.filter.sort')}
          data={(['title', 'added', 'played'] as const).map((s) => ({ value: s, label: t(`library.sort.${s}`) }))}
          value={search.sort ?? 'title'}
          onChange={(value) =>
            onChange({ sort: value === 'added' || value === 'played' ? value : undefined })
          }
          allowDeselect={false}
        />
        <Button variant="default" onClick={onClear}>
          {t('library.clearFilters')}
        </Button>
      </Stack>
    </Drawer>
  );
}
