import {
  ActionIcon,
  Badge,
  Button,
  Group,
  Loader,
  MultiSelect,
  Select,
  Stack,
  Switch,
  Table,
  Text,
  Textarea,
  TextInput,
} from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconPencil, IconPlus, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useDevices, usePlatforms } from '../../api/queries';
import { DEVICE_KINDS, type Device, type DeviceInput, type DeviceKind, type Platform } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { useApiErrorHandler } from '../../hooks/useApiForm';

type Editing = { device?: Device } | null;

function platformLabel(platform: Platform) {
  return platform.abbreviation ?? platform.name;
}

export function DevicesTab() {
  const { t } = useTranslation();
  const devices = useDevices();
  const platforms = usePlatforms();
  const [editing, setEditing] = useState<Editing>(null);
  const [deleting, setDeleting] = useState<Device | null>(null);
  const queryClient = useQueryClient();

  const platformsById = useMemo(
    () => new Map((platforms.data ?? []).map((platform) => [platform.id, platform])),
    [platforms.data],
  );

  const remove = useMutation({
    mutationFn: (id: number) =>
      call(api.DELETE('/api/devices/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.devices });
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (devices.isPending || platforms.isPending) return <Loader size="sm" />;
  if (devices.isError) return <Text c="red">{errorMessage(t, devices.error)}</Text>;
  if (platforms.isError) return <Text c="red">{errorMessage(t, platforms.error)}</Text>;

  return (
    <Stack>
      <Group justify="flex-end">
        <Button leftSection={<IconPlus size={16} />} onClick={() => setEditing({})}>
          {t('devices.add')}
        </Button>
      </Group>

      {devices.data.length === 0 ? (
        <Text c="dimmed">{t('devices.empty')}</Text>
      ) : (
        <Table.ScrollContainer minWidth={640}>
          <Table highlightOnHover>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>{t('common.name')}</Table.Th>
                <Table.Th>{t('devices.kind')}</Table.Th>
                <Table.Th>{t('devices.platforms')}</Table.Th>
                <Table.Th>{t('common.copies')}</Table.Th>
                <Table.Th w={96}>{t('common.actions')}</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {devices.data.map((device) => (
                <Table.Tr key={device.id} data-testid={`device-${device.name}`}>
                  <Table.Td>
                    <Group gap="xs">
                      <Text size="sm">{device.name}</Text>
                      {!device.active && (
                        <Badge size="xs" color="gray" variant="light">
                          {t('common.inactive')}
                        </Badge>
                      )}
                    </Group>
                    {device.model && (
                      <Text size="xs" c="dimmed">
                        {device.model}
                      </Text>
                    )}
                  </Table.Td>
                  <Table.Td>{t(`deviceKind.${device.kind}`)}</Table.Td>
                  <Table.Td>
                    <Group gap={4}>
                      {device.platform_ids.map((id) => {
                        const platform = platformsById.get(id);
                        return platform ? (
                          <Badge key={id} variant="outline" size="sm" tt="none">
                            {platformLabel(platform)}
                          </Badge>
                        ) : null;
                      })}
                    </Group>
                  </Table.Td>
                  <Table.Td>{device.copy_count}</Table.Td>
                  <Table.Td>
                    <Group gap={4} wrap="nowrap">
                      <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => setEditing({ device })}>
                        <IconPencil size={16} />
                      </ActionIcon>
                      <ActionIcon
                        variant="subtle"
                        color="red"
                        aria-label={t('common.delete')}
                        onClick={() => setDeleting(device)}
                      >
                        <IconTrash size={16} />
                      </ActionIcon>
                    </Group>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </Table.ScrollContainer>
      )}

      {editing && (
        <DeviceForm device={editing.device} platforms={platforms.data} onClose={() => setEditing(null)} />
      )}

      <ConfirmModal
        opened={deleting !== null}
        title={t('devices.deleteTitle', { name: deleting?.name ?? '' })}
        message={deleting?.copy_count ? t('devices.deleteCopies', { count: deleting.copy_count }) : undefined}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => deleting && remove.mutate(deleting.id)}
        onClose={() => setDeleting(null)}
      />
    </Stack>
  );
}

type FormValues = {
  name: string;
  kind: DeviceKind;
  model: string;
  notes: string;
  active: boolean;
  platform_ids: string[];
};

function DeviceForm({
  device,
  platforms,
  onClose,
}: {
  device?: Device;
  platforms: Platform[];
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const form = useForm<FormValues>({
    initialValues: {
      name: device?.name ?? '',
      kind: device?.kind ?? 'console',
      model: device?.model ?? '',
      notes: device?.notes ?? '',
      active: device?.active ?? true,
      platform_ids: (device?.platform_ids ?? []).map(String),
    },
    validate: { name: (value) => (value.trim() ? null : t('errors.required')) },
  });
  const onApiError = useApiErrorHandler(form);

  // Inactive platforms stay listed when the device already uses them.
  const platformOptions = platforms
    .filter((platform) => platform.active || device?.platform_ids.includes(platform.id))
    .map((platform) => ({ value: String(platform.id), label: platform.name }));

  const save = useMutation({
    mutationFn: (body: DeviceInput) =>
      device
        ? call(api.PUT('/api/devices/{id}', { params: { path: { id: device.id } }, body }))
        : call(api.POST('/api/devices', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.devices });
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={device ? t('devices.edit') : t('devices.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) =>
        save.mutate({ ...values, platform_ids: values.platform_ids.map(Number) }),
      )}
    >
      <TextInput label={t('common.name')} required data-autofocus {...form.getInputProps('name')} />
      <Select
        label={t('devices.kind')}
        allowDeselect={false}
        data={DEVICE_KINDS.map((kind) => ({ value: kind, label: t(`deviceKind.${kind}`) }))}
        {...form.getInputProps('kind')}
      />
      <TextInput label={t('devices.model')} placeholder={t('devices.modelPlaceholder')} {...form.getInputProps('model')} />
      <MultiSelect
        label={t('devices.platforms')}
        description={t('devices.platformsDescription')}
        data={platformOptions}
        searchable
        clearable
        {...form.getInputProps('platform_ids')}
      />
      <Textarea label={t('common.notes')} autosize minRows={2} {...form.getInputProps('notes')} />
      <Switch
        label={t('common.active')}
        description={t('devices.activeDescription')}
        {...form.getInputProps('active', { type: 'checkbox' })}
      />
    </FormModal>
  );
}
