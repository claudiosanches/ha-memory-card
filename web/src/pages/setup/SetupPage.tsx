import { Badge, Button, Card, Chip, Group, Loader, MultiSelect, Progress, Select, Stack, Text, Title } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useLocalStorage } from '@mantine/hooks';
import { IconLink, IconPlus } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import {
  queryKeys,
  useDevices,
  useGame,
  useGameModes,
  useMissingDetails,
  usePlatforms,
  useStores,
} from '../../api/queries';
import {
  CONDITIONS,
  type CopyDetail,
  type CopyInput,
  type GameDetail,
  type GameInput,
  type MissingField,
  type ReleaseDetail,
} from '../../api/types';
import { notifyError } from '../../components/notify';
import { CoverPicker } from '../game/CoverPicker';
import { checkedOrNull, CopyForm, gameInput, numberOrNull, PlayerFields } from '../game/forms';
import { IgdbLinkModal } from '../game/IgdbLink';
import classes from './SetupPage.module.css';

const FIELDS: MissingField[] = [
  'cover',
  'game_modes',
  'players',
  'copies',
  'device',
  'store',
  'condition',
  'igdb',
];

/** Walks through games that lack the chosen details, one at a time. */
export function SetupTab() {
  const { t } = useTranslation();
  const missing = useMissingDetails();
  const [wanted, setWanted] = useLocalStorage<MissingField[]>({
    key: 'memory-card-setup-fields',
    defaultValue: FIELDS,
  });
  // The list is fixed when a run starts, so finishing a game does not shift the rest.
  const [run, setRun] = useState<{ ids: number[]; index: number } | null>(null);

  if (missing.isPending) return <Loader size="sm" />;
  if (missing.isError) return <Text c="red">{errorMessage(t, missing.error)}</Text>;

  const lacking = (fields: MissingField[]) => fields.filter((field) => wanted.includes(field));
  const matching = missing.data.filter((game) => lacking(game.fields).length > 0);
  const counts = new Map<MissingField, number>();
  for (const game of missing.data) {
    for (const field of game.fields) counts.set(field, (counts.get(field) ?? 0) + 1);
  }

  if (run) {
    const id = run.ids[run.index];
    if (id === undefined) {
      return (
        <Stack gap="md">
          <Text>{t('setup.done')}</Text>
          <Button w="fit-content" onClick={() => setRun(null)}>
            {t('setup.again')}
          </Button>
        </Stack>
      );
    }
    return (
      <Step
        key={id}
        gameId={id}
        index={run.index}
        total={run.ids.length}
        fields={lacking(missing.data.find((game) => game.game_id === id)?.fields ?? [])}
        onNext={() => setRun({ ...run, index: run.index + 1 })}
        onStop={() => setRun(null)}
      />
    );
  }

  return (
    <Card withBorder radius="md">
      <Stack gap="md">
        <div>
          <Text fw={600}>{t('setup.lookFor')}</Text>
          <Text size="sm" c="dimmed">
            {t('setup.intro')}
          </Text>
        </div>
        <Chip.Group multiple value={wanted} onChange={(value) => setWanted(value as MissingField[])}>
          <Group gap="xs">
            {FIELDS.filter((field) => counts.has(field)).map((field) => (
              <Chip key={field} value={field}>
                {t(`setup.fields.${field}`)}
                {t('common.separator')}
                {counts.get(field)}
              </Chip>
            ))}
          </Group>
        </Chip.Group>
        {matching.length > 0 ? (
          <Button
            w="fit-content"
            onClick={() => setRun({ ids: matching.map((game) => game.game_id), index: 0 })}
          >
            {t('setup.start', { count: matching.length })}
          </Button>
        ) : (
          <Text c="dimmed">{t('setup.nothing')}</Text>
        )}
      </Stack>
    </Card>
  );
}

type StepProps = {
  gameId: number;
  index: number;
  total: number;
  /** What the game still lacks, among the chosen details. */
  fields: MissingField[];
  onNext: () => void;
  onStop: () => void;
};

function Step(props: StepProps) {
  const { t } = useTranslation();
  const game = useGame(props.gameId);
  if (game.isPending) return <Loader size="sm" />;
  if (game.isError) {
    return (
      <Stack>
        <Text c="red">{errorMessage(t, game.error)}</Text>
        <Button w="fit-content" onClick={props.onNext}>
          {t('setup.skip')}
        </Button>
      </Stack>
    );
  }
  return <StepForm {...props} game={game.data} />;
}

type CopyEdit = { device_id?: string | null; store_id?: string | null; condition?: string | null };
type OwnedCopy = { copy: CopyDetail; release: ReleaseDetail };

function StepForm({ game, index, total, fields, onNext, onStop }: StepProps & { game: GameDetail }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const gameModes = useGameModes();
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const [dialog, setDialog] = useState<'copy' | 'igdb' | null>(null);
  const [copyEdits, setCopyEdits] = useState<Record<number, CopyEdit>>({});
  const form = useForm({
    initialValues: {
      game_mode_ids: [] as string[],
      local_max: '' as string | number,
      online_max: '' as string | number,
      local_coop: false,
      online_coop: false,
    },
  });

  const has = (field: MissingField) => fields.includes(field);
  const show = {
    cover: has('cover'),
    gameModes: has('game_modes'),
    players: has('players'),
    copies: has('copies'),
    igdb: has('igdb'),
  };
  const gaps = (copy: CopyDetail) => ({
    device: has('device') && copy.device_id == null,
    store: has('store') && copy.store_id == null,
    condition: has('condition') && copy.format === 'physical' && copy.condition == null,
  });
  const incomplete: OwnedCopy[] = game.releases
    // A bundled copy follows its bundle, as the missing-details list does.
    .flatMap((release) =>
      release.copies.filter((copy) => copy.status === 'owned' && !copy.bundle).map((copy) => ({ copy, release })),
    )
    .filter(({ copy }) => Object.values(gaps(copy)).some(Boolean));
  const editCopy = (id: number, edit: CopyEdit) =>
    setCopyEdits((edits) => ({ ...edits, [id]: { ...edits[id], ...edit } }));
  const changed = form.isDirty() || Object.keys(copyEdits).length > 0;

  const save = useMutation({
    mutationFn: async (v: typeof form.values) => {
      let latest = game;
      const patch: Partial<GameInput> = {};
      if (has('game_modes') && v.game_mode_ids.length) patch.game_mode_ids = v.game_mode_ids.map(Number);
      if (has('players') && (v.local_max !== '' || v.online_max !== '' || v.local_coop || v.online_coop)) {
        patch.local_max_players = numberOrNull(v.local_max);
        patch.online_max_players = numberOrNull(v.online_max);
        patch.local_coop = checkedOrNull(v.local_coop, game.local_coop);
        patch.online_coop = checkedOrNull(v.online_coop, game.online_coop);
      }
      if (Object.keys(patch).length) {
        latest = await call(
          api.PUT('/api/games/{id}', { params: { path: { id: game.id } }, body: { ...gameInput(game), ...patch } }),
        );
      }
      for (const { copy, release } of incomplete) {
        const edit = copyEdits[copy.id];
        if (!edit) continue;
        const body: CopyInput = {
          platform_id: release.platform_id,
          region: release.region,
          product_code: release.product_code,
          format: copy.format,
          device_id: edit.device_id ? Number(edit.device_id) : copy.device_id,
          store_id: edit.store_id ? Number(edit.store_id) : copy.store_id,
          condition: (edit.condition as CopyInput['condition']) || copy.condition,
          notes: copy.notes,
          edition: copy.edition,
          purchased_on: copy.purchased_on,
          price: copy.price,
          currency: copy.currency,
          estimated_value: copy.estimated_value,
          estimated_currency: copy.estimated_currency,
          barcode: copy.barcode,
        };
        latest = await call(api.PUT('/api/copies/{id}', { params: { path: { id: copy.id } }, body }));
      }
      return latest;
    },
    onSuccess: (latest) => {
      queryClient.setQueryData(queryKeys.game(latest.id), latest);
      for (const key of [queryKeys.games, queryKeys.missing, queryKeys.status, queryKeys.devices, queryKeys.stores]) {
        void queryClient.invalidateQueries({ queryKey: key });
      }
      onNext();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const platformName = (id: number) => platforms.data?.find((p) => p.id === id)?.name;
  const deviceOptions = (release: ReleaseDetail) => {
    const active = (devices.data ?? []).filter((d) => d.active);
    const fitting = active.filter((d) => d.platform_ids.includes(release.platform_id));
    return (fitting.length ? fitting : active).map((d) => ({ value: String(d.id), label: d.name }));
  };

  return (
    <Stack gap="card">
      <Stack gap="xs">
        <Text size="sm" c="dimmed">
          {t('setup.position', { index: index + 1, total })}
        </Text>
        <Progress value={(index / total) * 100} aria-hidden />
      </Stack>

      <Card withBorder radius="md">
        <div className={classes.step}>
          <CoverPicker
            target={{ kind: 'game', id: game.id }}
            path={game.cover_path}
            uploaded={game.cover_uploaded}
            title={game.title}
            width={120}
          />
          <Stack gap="md" className={classes.fields}>
            <Title order={2} size="h3" className={classes.title}>
              {game.title}
            </Title>
            {fields.length === 0 ? (
              <Text c="dimmed">{t('setup.complete')}</Text>
            ) : (
              <Group gap={6} data-testid="missing-fields">
                <Text size="sm" fw={600}>
                  {t('setup.missing')}:
                </Text>
                {fields.map((field) => (
                  <Badge key={field} variant="light" tt="none">
                    {t(`setup.fields.${field}`)}
                  </Badge>
                ))}
              </Group>
            )}
            {show.cover && (
              <Text size="sm" c="dimmed">
                {t('setup.coverHint')}
              </Text>
            )}
            {show.gameModes && (
              <MultiSelect
                label={t('game.gameModes')}
                data={(gameModes.data ?? []).map((m) => ({ value: String(m.id), label: m.name }))}
                clearable
                {...form.getInputProps('game_mode_ids')}
              />
            )}
            {show.players && (
              <PlayerFields
                localMax={form.getInputProps('local_max')}
                onlineMax={form.getInputProps('online_max')}
                localCoop={form.getInputProps('local_coop', { type: 'checkbox' })}
                onlineCoop={form.getInputProps('online_coop', { type: 'checkbox' })}
              />
            )}
            {show.copies && (
              <Group gap="sm">
                <Text size="sm" c="dimmed">
                  {t('setup.copiesHint')}
                </Text>
                <Button variant="default" size="xs" leftSection={<IconPlus size={14} />} onClick={() => setDialog('copy')}>
                  {t('game.addCopy')}
                </Button>
              </Group>
            )}
            {incomplete.map(({ copy, release }) => {
              const gap = gaps(copy);
              const edit = copyEdits[copy.id] ?? {};
              return (
                <div key={copy.id} className={classes.copy} data-testid={`setup-copy-${copy.id}`}>
                  <Text size="sm" fw={600}>
                    {[platformName(release.platform_id), t(`region.${release.region}`), t(`format.${copy.format}`)]
                      .filter(Boolean)
                      .join(t('common.separator'))}
                  </Text>
                  <div className={classes.copyFields}>
                    {gap.device && (
                      <Select
                        label={t('game.device')}
                        placeholder={t('game.noDevice')}
                        data={deviceOptions(release)}
                        searchable
                        clearable
                        value={edit.device_id ?? null}
                        onChange={(value) => editCopy(copy.id, { device_id: value })}
                      />
                    )}
                    {gap.store && (
                      <Select
                        label={t('game.store')}
                        placeholder={t('game.noStore')}
                        data={(stores.data ?? []).map((s) => ({ value: String(s.id), label: s.name }))}
                        searchable
                        clearable
                        value={edit.store_id ?? null}
                        onChange={(value) => editCopy(copy.id, { store_id: value })}
                      />
                    )}
                    {gap.condition && (
                      <Select
                        label={t('game.condition')}
                        placeholder={t('common.notSet')}
                        data={CONDITIONS.map((c) => ({ value: c, label: t(`condition.${c}`) }))}
                        clearable
                        value={edit.condition ?? null}
                        onChange={(value) => editCopy(copy.id, { condition: value })}
                      />
                    )}
                  </div>
                </div>
              );
            })}
            {show.igdb && (
              <Button
                variant="default"
                w="fit-content"
                leftSection={<IconLink size={16} />}
                onClick={() => setDialog('igdb')}
              >
                {t('igdb.link')}
              </Button>
            )}
          </Stack>
        </div>
      </Card>

      <Group justify="space-between" className={classes.actions}>
        <Button variant="subtle" color="gray" onClick={onStop}>
          {t('setup.stop')}
        </Button>
        <Group gap="sm">
          <Button variant="default" onClick={onNext} disabled={save.isPending}>
            {t('setup.skip')}
          </Button>
          <Button onClick={() => save.mutate(form.getValues())} loading={save.isPending} disabled={!changed}>
            {t('setup.saveNext')}
          </Button>
        </Group>
      </Group>

      {dialog === 'copy' && <CopyForm gameId={game.id} onClose={() => setDialog(null)} />}
      {dialog === 'igdb' && <IgdbLinkModal game={game} onClose={() => setDialog(null)} />}
    </Stack>
  );
}
