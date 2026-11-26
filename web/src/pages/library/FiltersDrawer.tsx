import { Button, Drawer, MultiSelect, NumberInput, SegmentedControl, Select, Stack } from '@mantine/core';
import { useTranslation } from 'react-i18next';
import { useDevices, useGameModes, useGenres, useLabels, usePlatforms, useStores } from '../../api/queries';
import { FORMATS, type Format, REGIONS, type Region } from '../../api/types';
import { usePlayStatusList } from '../../components/playStatuses';
import { type LibrarySearch, MAX_HOURS_TO_BEAT, NO_STATUS, type StatusFilter } from './search';

type Props = {
  opened: boolean;
  search: LibrarySearch;
  onChange: (patch: Partial<LibrarySearch>) => void;
  onClear: () => void;
  onClose: () => void;
};

const orUndefined = (values: string[]) => (values.length ? values : undefined);

export function FiltersDrawer({ opened, search, onChange, onClear, onClose }: Props) {
  const { t } = useTranslation();
  const statuses = usePlayStatusList();
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const gameModes = useGameModes();
  const genres = useGenres();
  const labels = useLabels();

  const platformOptions = (platforms.data ?? [])
    .filter((p) => p.active || p.game_count > 0 || search.platform?.includes(p.slug))
    .map((p) => ({ value: p.slug, label: p.name }));

  return (
    <Drawer opened={opened} onClose={onClose} title={t('library.filters')} position="right" size="sm">
      <Stack>
        <SegmentedControl
          fullWidth
          value={search.ownership ?? 'owned'}
          onChange={(value) => onChange({ ownership: value === 'any' || value === 'wanted' ? value : undefined })}
          data={[
            { value: 'owned', label: t('library.filter.owned') },
            { value: 'wanted', label: t('library.filter.wanted') },
            { value: 'any', label: t('library.filter.any') },
          ]}
        />
        {(labels.data?.length ?? 0) > 0 && (
          <MultiSelect
            label={t('library.filter.label')}
            data={(labels.data ?? []).map((l) => ({ value: l.slug, label: l.name }))}
            value={search.label ?? []}
            onChange={(values) => onChange({ label: orUndefined(values) })}
            searchable
            clearable
          />
        )}
        <MultiSelect
          label={t('library.filter.playStatus')}
          data={[...statuses.options, { value: NO_STATUS, label: t('common.notSet') }]}
          value={search.play_status ?? []}
          onChange={(values) => onChange({ play_status: values.length ? (values as StatusFilter[]) : undefined })}
          clearable
        />
        <MultiSelect
          label={t('library.filter.platform')}
          data={platformOptions}
          value={search.platform ?? []}
          onChange={(values) => onChange({ platform: orUndefined(values) })}
          searchable
          clearable
        />
        <MultiSelect
          label={t('library.filter.device')}
          data={(devices.data ?? []).map((d) => ({ value: d.slug, label: d.name }))}
          value={search.device ?? []}
          onChange={(values) => onChange({ device: orUndefined(values) })}
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
        <Select
          label={t('library.filter.igdb')}
          data={[
            { value: '', label: t('library.filter.igdbAny') },
            { value: 'linked', label: t('library.filter.igdbLinked') },
            { value: 'unlinked', label: t('library.filter.igdbUnlinked') },
          ]}
          value={search.igdb ?? ''}
          onChange={(value) =>
            onChange({ igdb: value === 'linked' || value === 'unlinked' ? value : undefined })
          }
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
          label={search.ownership === 'wanted' ? t('library.filter.buyAt') : t('library.filter.store')}
          data={(stores.data ?? []).map((s) => ({ value: s.slug, label: s.name }))}
          value={search.store ?? []}
          onChange={(values) => onChange({ store: orUndefined(values) })}
          searchable
          clearable
        />
        <MultiSelect
          label={t('library.filter.gameMode')}
          data={(gameModes.data ?? []).map((m) => ({ value: m.slug, label: m.name }))}
          value={search.game_mode ?? []}
          onChange={(values) => onChange({ game_mode: orUndefined(values) })}
          clearable
        />
        {(genres.data?.length ?? 0) > 0 && (
          <MultiSelect
            label={t('library.filter.genre')}
            data={(genres.data ?? []).map((g) => ({ value: g.slug, label: g.name }))}
            value={search.genre ?? []}
            onChange={(values) => onChange({ genre: orUndefined(values) })}
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
        <NumberInput
          label={t('library.filter.maxHoursToBeat')}
          min={1}
          max={MAX_HOURS_TO_BEAT}
          value={search.max_hours_to_beat ?? ''}
          onChange={(value) => {
            const hours = Number(value);
            onChange({
              max_hours_to_beat: Number.isInteger(hours) && hours >= 1 && hours <= MAX_HOURS_TO_BEAT ? hours : undefined,
            });
          }}
        />
        <Button variant="default" onClick={onClear}>
          {t('library.clearFilters')}
        </Button>
      </Stack>
    </Drawer>
  );
}
