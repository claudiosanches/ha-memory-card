import {
  Button,
  Checkbox,
  Group,
  MultiSelect,
  NumberInput,
  SegmentedControl,
  Select,
  Switch,
  Text,
  Textarea,
  TextInput,
} from '@mantine/core';
import { type UseFormReturnType, useForm } from '@mantine/form';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call } from '../../api/errors';
import { queryKeys, useDevices, useGameModes, usePlatforms, useStatus, useStores } from '../../api/queries';
import {
  CONDITIONS,
  type CopyDetail,
  type CopyInput,
  type CopyStatus,
  type Format,
  type GameDetail,
  type GameInput,
  REGIONS,
  type Region,
  type ReleaseDetail,
} from '../../api/types';
import { FormModal } from '../../components/FormModal';
import { currencyOptions, localeCurrency } from '../../components/money';
import { notifySuccess } from '../../components/notify';
import { useApiErrorHandler } from '../../hooks/useApiForm';

/** Puts a mutation's GameDetail into the cache and refreshes lists. */
export function useGameSaved() {
  const queryClient = useQueryClient();
  const { t } = useTranslation();
  return (game: GameDetail, message = t('common.saved')) => {
    queryClient.setQueryData(queryKeys.game(game.id), game);
    void queryClient.invalidateQueries({ queryKey: queryKeys.games });
    void queryClient.invalidateQueries({ queryKey: queryKeys.status });
    void queryClient.invalidateQueries({ queryKey: queryKeys.devices });
    void queryClient.invalidateQueries({ queryKey: queryKeys.stores });
    void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
    void queryClient.invalidateQueries({ queryKey: queryKeys.missing });
    notifySuccess(message);
  };
}

/** Every editable field of a game, ready to change and send back. */
export function gameInput(game: GameDetail): GameInput {
  return {
    title: game.title,
    summary: game.summary,
    play_status: game.play_status,
    local_max_players: game.local_max_players,
    online_max_players: game.online_max_players,
    local_coop: game.local_coop,
    online_coop: game.online_coop,
    game_mode_ids: game.game_mode_ids,
  };
}

export const numberOrNull = (value: string | number) => (value === '' ? null : Number(value));
const textOrNull = (value: string) => (value.trim() ? value : null);
/** Checkbox state for an optional boolean: unknown and false look the same, so keep null unless checked. */
export const checkedOrNull = (value: boolean, before?: boolean | null) => (value ? true : before === false ? false : null);

type CopyFormProps = {
  /** The game a new copy goes to. */
  gameId?: number;
  /** Called on save when there is no `gameId`, to create the game only once the copy is saved. */
  resolveGame?: () => Promise<number>;
  copy?: Omit<CopyDetail, 'created_at' | 'created_by' | 'updated_by' | 'bundle' | 'contents'>;
  release?: Pick<ReleaseDetail, 'platform_id' | 'region' | 'product_code'>;
  /** Starts the form on this status, such as owned for a wanted copy just bought. */
  status?: CopyStatus;
  title?: string;
  onClose: () => void;
};

export function CopyForm({ gameId, resolveGame, copy, release, status, title, onClose }: CopyFormProps) {
  const { t, i18n } = useTranslation();
  const platforms = usePlatforms();
  const devices = useDevices();
  const stores = useStores();
  const appStatus = useStatus();
  const saved = useGameSaved();
  const [allDevices, setAllDevices] = useState(false);
  const currencies = useMemo(() => currencyOptions(i18n.language), [i18n.language]);
  // Remounts the searchable selects after "Save and add another"; they keep their typed text otherwise.
  const [round, setRound] = useState(0);

  const form = useForm({
    initialValues: {
      status: status ?? copy?.status ?? ('owned' as CopyStatus),
      platform_id: release ? String(release.platform_id) : '',
      region: (release?.region ?? '') as Region | '',
      product_code: release?.product_code ?? '',
      format: (copy?.format ?? 'physical') as Format,
      device_id: copy?.device_id ? String(copy.device_id) : '',
      store_id: copy?.store_id ? String(copy.store_id) : '',
      condition: copy?.condition ?? '',
      notes: copy?.notes ?? '',
      edition: copy?.edition ?? '',
      purchased_on: copy?.purchased_on ?? '',
      price: copy?.price ?? '',
      currency: copy?.currency ?? appStatus.data?.last_currency ?? localeCurrency(i18n.language),
    },
    validate: {
      platform_id: (value) => (value ? null : t('errors.required')),
      region: (value) => (value ? null : t('errors.required')),
    },
  });
  const onApiError = useApiErrorHandler(form);
  const values = form.getValues();
  const platformId = Number(values.platform_id);

  const platformOptions = (platforms.data ?? [])
    .filter((p) => p.active || p.id === release?.platform_id)
    .map((p) => ({ value: String(p.id), label: p.name }));
  const deviceOptions = (devices.data ?? [])
    .filter(
      (d) =>
        allDevices ||
        d.id === copy?.device_id ||
        (d.active && (!platformId || d.platform_ids.includes(platformId))),
    )
    .map((d) => ({ value: String(d.id), label: d.name }));

  // "Save and add another" keeps the form open for the same release.
  const save = useMutation({
    mutationFn: async ({ body }: { body: CopyInput; another: boolean }) => {
      if (copy) return call(api.PUT('/api/copies/{id}', { params: { path: { id: copy.id } }, body }));
      const id = gameId ?? (await resolveGame!());
      return call(api.POST('/api/games/{id}/copies', { params: { path: { id } }, body }));
    },
    onSuccess: (game, { another }) => {
      saved(game);
      if (!another) {
        onClose();
        return;
      }
      const { status, platform_id, region, product_code, format, purchased_on, currency } = form.getValues();
      form.setValues({
        status,
        platform_id,
        region,
        product_code,
        format,
        purchased_on,
        currency,
        device_id: '',
        store_id: '',
        condition: '',
        notes: '',
        edition: '',
        price: '',
      });
      form.resetDirty();
      setRound((value) => value + 1);
    },
    onError: onApiError,
  });

  const submit = (addAnother: boolean) =>
    form.onSubmit((v) => {
      const wanted = v.status === 'wanted';
      const body: CopyInput = {
        status: v.status,
        platform_id: Number(v.platform_id),
        region: v.region as Region,
        product_code: textOrNull(v.product_code),
        format: v.format,
        // A copy you do not have yet is on no device.
        device_id: v.device_id && !wanted ? Number(v.device_id) : null,
        store_id: v.store_id ? Number(v.store_id) : null,
        condition: v.format === 'physical' && v.condition ? (v.condition as CopyInput['condition']) : null,
        notes: textOrNull(v.notes),
        edition: textOrNull(v.edition),
        purchased_on: wanted ? null : textOrNull(v.purchased_on),
        price: textOrNull(v.price),
        currency: v.price.trim() ? v.currency : null,
      };
      save.mutate({ body, another: addAnother });
    });

  return (
    <FormModal
      opened
      title={title ?? (copy ? t('game.editCopy') : t('game.addCopy'))}
      saving={save.isPending && !save.variables.another}
      onClose={onClose}
      onSubmit={submit(false)}
      extraAction={
        !copy && (
          <Button variant="light" loading={save.isPending && save.variables.another} onClick={() => submit(true)()}>
            {t('game.saveAnother')}
          </Button>
        )
      }
    >
      <SegmentedControl
        aria-label={t('game.copyStatus')}
        data={(['owned', 'wanted'] as const).map((value) => ({ value, label: t(`copyStatus.${value}`) }))}
        {...form.getInputProps('status')}
      />
      <Select
        label={t('game.platform')}
        required
        searchable
        data={platformOptions}
        {...form.getInputProps('platform_id')}
      />
      <Select
        label={t('game.region')}
        required
        data={REGIONS.map((r) => ({ value: r, label: t(`region.${r}`) }))}
        {...form.getInputProps('region')}
      />
      <TextInput label={t('game.productCode')} {...form.getInputProps('product_code')} />
      <SegmentedControl
        aria-label={t('game.format')}
        data={(['physical', 'digital'] as const).map((f) => ({ value: f, label: t(`format.${f}`) }))}
        {...form.getInputProps('format')}
      />
      <TextInput label={t('game.edition')} placeholder={t('game.editionPlaceholder')} {...form.getInputProps('edition')} />
      {values.status === 'owned' && (
        <>
          <Select
            key={`device-${round}`}
            label={t('game.device')}
            placeholder={t('game.noDevice')}
            data={deviceOptions}
            clearable
            searchable
            {...form.getInputProps('device_id')}
            // Select treats an empty string as a value and would offer to clear it.
            value={form.values.device_id || null}
          />
          <Checkbox
            size="xs"
            label={t('game.showAllDevices')}
            checked={allDevices}
            onChange={(event) => setAllDevices(event.currentTarget.checked)}
          />
        </>
      )}
      <Select
        key={`store-${round}`}
        label={values.status === 'wanted' ? t('game.buyAt') : t('game.store')}
        placeholder={t('game.noStore')}
        data={(stores.data ?? []).map((s) => ({ value: String(s.id), label: s.name }))}
        clearable
        searchable
        {...form.getInputProps('store_id')}
        value={form.values.store_id || null}
      />
      {values.format === 'physical' && (
        <Select
          label={t('game.condition')}
          data={CONDITIONS.map((c) => ({ value: c, label: t(`condition.${c}`) }))}
          clearable
          {...form.getInputProps('condition')}
          value={form.values.condition || null}
        />
      )}
      {values.status === 'owned' && (
        <TextInput type="date" label={t('game.purchasedOn')} {...form.getInputProps('purchased_on')} />
      )}
      <Group grow align="flex-start">
        <TextInput
          label={values.status === 'wanted' ? t('game.expectedPrice') : t('game.price')}
          inputMode="decimal"
          {...form.getInputProps('price')}
        />
        <Select
          label={t('game.currency')}
          data={currencies}
          searchable
          allowDeselect={false}
          comboboxProps={{ width: 280, position: 'bottom-end' }}
          {...form.getInputProps('currency')}
        />
      </Group>
      <Textarea label={t('common.notes')} autosize minRows={2} {...form.getInputProps('notes')} />
    </FormModal>
  );
}

type InputProps = ReturnType<UseFormReturnType<Record<string, unknown>>['getInputProps']>;

export function PlayerFields(props: { localMax: InputProps; onlineMax: InputProps; localCoop: InputProps; onlineCoop: InputProps }) {
  const { t } = useTranslation();
  return (
    <>
      <Group grow>
        <NumberInput label={t('game.localMax')} min={1} max={99} {...props.localMax} />
        <NumberInput label={t('game.onlineMax')} min={1} max={99} {...props.onlineMax} />
      </Group>
      <Group>
        <Switch label={t('game.localCoop')} {...props.localCoop} />
        <Switch label={t('game.onlineCoop')} {...props.onlineCoop} />
      </Group>
    </>
  );
}

export function ReleaseForm({ release, onClose }: { release: ReleaseDetail; onClose: () => void }) {
  const { t } = useTranslation();
  const saved = useGameSaved();
  const form = useForm({
    initialValues: {
      title: release.title ?? '',
      product_code: release.product_code ?? '',
      release_date: release.release_date ?? '',
      local_max: (release.local_max_players ?? '') as string | number,
      online_max: (release.online_max_players ?? '') as string | number,
      local_coop: release.local_coop === true,
      online_coop: release.online_coop === true,
    },
  });
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (v: typeof form.values) =>
      call(
        api.PUT('/api/releases/{id}', {
          params: { path: { id: release.id } },
          body: {
            title: textOrNull(v.title),
            product_code: textOrNull(v.product_code),
            release_date: textOrNull(v.release_date),
            local_max_players: numberOrNull(v.local_max),
            online_max_players: numberOrNull(v.online_max),
            local_coop: checkedOrNull(v.local_coop, release.local_coop),
            online_coop: checkedOrNull(v.online_coop, release.online_coop),
          },
        }),
      ),
    onSuccess: (game) => {
      saved(game);
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={t('game.editRelease')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((v) => save.mutate(v))}
    >
      <TextInput label={t('game.releaseTitle')} {...form.getInputProps('title')} />
      <TextInput label={t('game.productCode')} {...form.getInputProps('product_code')} />
      <TextInput label={t('game.releaseDate')} placeholder={t('game.datePlaceholder')} {...form.getInputProps('release_date')} />
      <PlayerFields
        localMax={form.getInputProps('local_max')}
        onlineMax={form.getInputProps('online_max')}
        localCoop={form.getInputProps('local_coop', { type: 'checkbox' })}
        onlineCoop={form.getInputProps('online_coop', { type: 'checkbox' })}
      />
      <Text size="xs" c="dimmed">
        {t('game.releasePlayersHint')}
      </Text>
    </FormModal>
  );
}

export function GameForm({ game, onClose }: { game: GameDetail; onClose: () => void }) {
  const { t } = useTranslation();
  const gameModes = useGameModes();
  const saved = useGameSaved();
  const form = useForm({
    initialValues: {
      title: game.title,
      summary: game.summary ?? '',
      game_mode_ids: game.game_mode_ids.map(String),
      local_max: (game.local_max_players ?? '') as string | number,
      online_max: (game.online_max_players ?? '') as string | number,
      local_coop: game.local_coop === true,
      online_coop: game.online_coop === true,
    },
    validate: { title: (value) => (value.trim() ? null : t('errors.required')) },
  });
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (v: typeof form.values) =>
      call(
        api.PUT('/api/games/{id}', {
          params: { path: { id: game.id } },
          body: {
            ...gameInput(game),
            title: v.title,
            summary: textOrNull(v.summary),
            game_mode_ids: v.game_mode_ids.map(Number),
            local_max_players: numberOrNull(v.local_max),
            online_max_players: numberOrNull(v.online_max),
            local_coop: checkedOrNull(v.local_coop, game.local_coop),
            online_coop: checkedOrNull(v.online_coop, game.online_coop),
          },
        }),
      ),
    onSuccess: (updated) => {
      saved(updated);
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={t('game.edit')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((v) => save.mutate(v))}
    >
      <TextInput label={t('game.title')} required data-autofocus {...form.getInputProps('title')} />
      <MultiSelect
        label={t('game.gameModes')}
        data={(gameModes.data ?? []).map((m) => ({ value: String(m.id), label: m.name }))}
        clearable
        {...form.getInputProps('game_mode_ids')}
      />
      <PlayerFields
        localMax={form.getInputProps('local_max')}
        onlineMax={form.getInputProps('online_max')}
        localCoop={form.getInputProps('local_coop', { type: 'checkbox' })}
        onlineCoop={form.getInputProps('online_coop', { type: 'checkbox' })}
      />
      <Textarea label={t('game.summary')} autosize minRows={3} {...form.getInputProps('summary')} />
    </FormModal>
  );
}
