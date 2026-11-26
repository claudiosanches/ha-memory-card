import { Group, MultiSelect, SegmentedControl, Select, Switch, Textarea, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call } from '../../api/errors';
import { queryKeys, usePlatforms, useStatus, useStores } from '../../api/queries';
import {
  CONDITIONS,
  type CopyStatus,
  DEVICE_KINDS,
  type Device,
  type DeviceInput,
  type DeviceKind,
  GAME_DEVICE_KINDS,
  holdsGames,
  REGIONS,
} from '../../api/types';
import { FormModal } from '../../components/FormModal';
import { currencyOptions, localeCurrency } from '../../components/money';
import { notifySuccess } from '../../components/notify';
import { SlugInput, slugRule, useSlugInputs } from '../../components/SlugInput';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { trimSlug } from '../../slug';

const text = (value: string) => value.trim() || null;

/** Adds or edits a device: a console or account copies live on, or a controller, accessory or toy. */
export function HardwareForm({
  device,
  status,
  onClose,
}: {
  device?: Device;
  /** Overrides the device's status, such as `owned` for Got it. */
  status?: CopyStatus;
  onClose: () => void;
}) {
  const { t, i18n } = useTranslation();
  const queryClient = useQueryClient();
  const platforms = usePlatforms();
  const stores = useStores();
  const appStatus = useStatus();
  const currencies = useMemo(() => currencyOptions(i18n.language), [i18n.language]);
  const fallbackCurrency = appStatus.data?.default_currency ?? localeCurrency(i18n.language);
  const form = useForm({
    initialValues: {
      name: device?.name ?? '',
      slug: device?.slug ?? '',
      kind: device?.kind ?? ('console' as DeviceKind),
      status: status ?? device?.status ?? ('owned' as CopyStatus),
      model: device?.model ?? '',
      notes: device?.notes ?? '',
      active: device?.active ?? true,
      platform_ids: (device?.platform_ids ?? []).map(String),
      platform_id: device?.platform_id ? String(device.platform_id) : '',
      region: device?.region ?? '',
      condition: device?.condition ?? '',
      store_id: device?.store_id ? String(device.store_id) : '',
      purchased_on: device?.purchased_on ?? '',
      price: device?.price ?? '',
      currency: device?.currency ?? fallbackCurrency,
      estimated_value: device?.estimated_value ?? '',
      estimated_currency: device?.estimated_currency ?? device?.currency ?? fallbackCurrency,
      serial_number: device?.serial_number ?? '',
      barcode: device?.barcode ?? '',
    },
    validate: {
      name: (value) => (value.trim() ? null : t('errors.required')),
      slug: slugRule(t, device !== undefined),
    },
  });
  const inputs = useSlugInputs(form, device?.slug);
  const onApiError = useApiErrorHandler(form);
  const values = form.getValues();
  const games = holdsGames(values.kind);
  const account = values.kind === 'account';
  const owned = values.status === 'owned';

  // Inactive platforms stay listed when the device already uses them.
  const platformOptions = (platforms.data ?? [])
    .filter((p) => p.active || device?.platform_ids.includes(p.id) || device?.platform_id === p.id)
    .map((p) => ({ value: String(p.id), label: p.name }));

  const save = useMutation({
    mutationFn: (body: DeviceInput) =>
      device
        ? call(api.PUT('/api/devices/{id}', { params: { path: { id: device.id } }, body }))
        : call(api.POST('/api/devices', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.devices });
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      void queryClient.invalidateQueries({ queryKey: queryKeys.stats });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  const submit = form.onSubmit((v) =>
    save.mutate({
      name: v.name,
      slug: trimSlug(v.slug) || null,
      kind: v.kind,
      status: v.status,
      model: text(v.model),
      notes: text(v.notes),
      active: v.active,
      platform_ids: games ? v.platform_ids.map(Number) : [],
      platform_id: !games && v.platform_id ? Number(v.platform_id) : null,
      region: (v.region || null) as DeviceInput['region'],
      condition: !account && v.condition ? (v.condition as DeviceInput['condition']) : null,
      store_id: v.store_id ? Number(v.store_id) : null,
      purchased_on: owned ? text(v.purchased_on) : null,
      price: text(v.price),
      currency: v.currency,
      estimated_value: owned ? text(v.estimated_value) : null,
      estimated_currency: v.estimated_currency,
      serial_number: account ? null : text(v.serial_number),
      barcode: account ? null : text(v.barcode),
    }),
  );

  const kindGroups = [
    { group: t('hardware.holdsGames'), items: GAME_DEVICE_KINDS },
    { group: t('hardware.other'), items: DEVICE_KINDS.filter((k) => !holdsGames(k)) },
  ].map(({ group, items }) => ({ group, items: items.map((kind) => ({ value: kind, label: t(`deviceKind.${kind}`) })) }));

  return (
    <FormModal
      opened
      title={device ? t('hardware.edit') : t('hardware.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={submit}
    >
      <SegmentedControl
        fullWidth
        data={[
          { value: 'owned', label: t('copyStatus.owned') },
          { value: 'wanted', label: t('copyStatus.wanted') },
        ]}
        {...form.getInputProps('status')}
      />
      <TextInput
        label={t('common.name')}
        description={t('hardware.nameDescription')}
        placeholder={t('devices.namePlaceholder')}
        required
        data-autofocus
        {...inputs.name}
      />
      <SlugInput param="device" saved={device?.slug} {...inputs.slug} />
      <Select label={t('devices.kind')} allowDeselect={false} data={kindGroups} {...form.getInputProps('kind')} />
      <TextInput
        label={t('devices.model')}
        description={t('hardware.modelDescription')}
        placeholder={t('devices.modelPlaceholder')}
        {...form.getInputProps('model')}
      />
      {games ? (
        <MultiSelect
          label={t('devices.platforms')}
          description={t('devices.platformsDescription')}
          data={platformOptions}
          searchable
          clearable
          {...form.getInputProps('platform_ids')}
        />
      ) : (
        <Select
          label={t('hardware.for')}
          description={t('hardware.forDescription')}
          data={platformOptions}
          searchable
          clearable
          {...form.getInputProps('platform_id')}
          value={values.platform_id || null}
        />
      )}
      <Select
        label={t('library.filter.region')}
        data={REGIONS.map((r) => ({ value: r, label: t(`region.${r}`) }))}
        clearable
        {...form.getInputProps('region')}
        value={values.region || null}
      />
      {!account && (
        <Select
          label={t('game.condition')}
          data={CONDITIONS.map((c) => ({ value: c, label: t(`condition.${c}`) }))}
          clearable
          {...form.getInputProps('condition')}
          value={values.condition || null}
        />
      )}
      <Select
        label={owned ? t('game.store') : t('game.buyAt')}
        placeholder={t('game.noStore')}
        data={(stores.data ?? []).map((s) => ({ value: String(s.id), label: s.name }))}
        clearable
        searchable
        {...form.getInputProps('store_id')}
        value={values.store_id || null}
      />
      {owned && <TextInput type="date" label={t('game.purchasedOn')} {...form.getInputProps('purchased_on')} />}
      <Group grow align="flex-start">
        <TextInput
          label={owned ? t('game.price') : t('game.expectedPrice')}
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
      {owned && (
        <Group grow align="flex-start">
          <TextInput
            label={t('game.estimatedValue')}
            description={t('game.estimatedValueHint')}
            inputMode="decimal"
            {...form.getInputProps('estimated_value')}
          />
          <Select
            label={t('game.currency')}
            aria-label={t('game.estimatedCurrency')}
            data={currencies}
            searchable
            allowDeselect={false}
            comboboxProps={{ width: 280, position: 'bottom-end' }}
            {...form.getInputProps('estimated_currency')}
          />
        </Group>
      )}
      {!account && (
        <Group grow align="flex-start">
          <TextInput label={t('hardware.serialNumber')} {...form.getInputProps('serial_number')} />
          <TextInput label={t('game.barcode')} inputMode="numeric" {...form.getInputProps('barcode')} />
        </Group>
      )}
      <Textarea label={t('common.notes')} autosize minRows={2} {...form.getInputProps('notes')} />
      {games && (
        <Switch
          label={t('common.active')}
          description={t('devices.activeDescription')}
          {...form.getInputProps('active', { type: 'checkbox' })}
        />
      )}
    </FormModal>
  );
}
