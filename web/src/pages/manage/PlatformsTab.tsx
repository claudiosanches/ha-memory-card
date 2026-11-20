import {
  ActionIcon,
  Badge,
  Button,
  Group,
  Loader,
  Stack,
  Switch,
  Table,
  Text,
  TextInput,
  Tooltip,
} from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconArrowDown, IconArrowUp, IconPencil, IconPlus, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, usePlatforms } from '../../api/queries';
import type { Platform, PlatformInput } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { useApiErrorHandler } from '../../hooks/useApiForm';

type Editing = { platform?: Platform } | null;

function inUse(platform: Platform) {
  return platform.game_count > 0 || platform.device_count > 0;
}

export function PlatformsTab() {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const [editing, setEditing] = useState<Editing>(null);
  const [deleting, setDeleting] = useState<Platform | null>(null);
  const queryClient = useQueryClient();
  const refresh = () => queryClient.invalidateQueries({ queryKey: queryKeys.platforms });

  const reorder = useMutation({
    mutationFn: (ids: number[]) => call(api.PUT('/api/platforms/order', { body: { ids } })),
    onSuccess: (list) => queryClient.setQueryData(queryKeys.platforms, list),
    onError: (error) => {
      void refresh();
      notifyError(errorMessage(t, error));
    },
  });

  const toggle = useMutation({
    mutationFn: (platform: Platform) =>
      call(
        api.PUT('/api/platforms/{id}', {
          params: { path: { id: platform.id } },
          body: { name: platform.name, abbreviation: platform.abbreviation, active: !platform.active },
        }),
      ),
    onSuccess: () => void refresh(),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const remove = useMutation({
    mutationFn: (id: number) =>
      call(api.DELETE('/api/platforms/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void refresh();
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (platforms.isPending) return <Loader size="sm" />;
  if (platforms.isError) return <Text c="red">{errorMessage(t, platforms.error)}</Text>;

  const list = platforms.data;
  const move = (index: number, delta: -1 | 1) => {
    const ids = list.map((platform) => platform.id);
    const target = index + delta;
    const moved = ids[index];
    const other = ids[target];
    if (moved === undefined || other === undefined) return;
    ids[index] = other;
    ids[target] = moved;
    reorder.mutate(ids);
  };

  return (
    <Stack>
      <Group justify="flex-end">
        <Button leftSection={<IconPlus size={16} />} onClick={() => setEditing({})}>
          {t('platforms.add')}
        </Button>
      </Group>

      <Table.ScrollContainer minWidth={640}>
        <Table highlightOnHover>
          <Table.Thead>
            <Table.Tr>
              <Table.Th w={72} />
              <Table.Th>{t('common.name')}</Table.Th>
              <Table.Th>{t('platforms.abbreviation')}</Table.Th>
              <Table.Th>{t('platforms.games')}</Table.Th>
              <Table.Th>{t('platforms.devices')}</Table.Th>
              <Table.Th>{t('common.active')}</Table.Th>
              <Table.Th w={96}>{t('common.actions')}</Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {list.map((platform, index) => (
              <Table.Tr key={platform.id} data-testid={`platform-${platform.slug}`}>
                <Table.Td>
                  <Group gap={2} wrap="nowrap">
                    <ActionIcon
                      variant="subtle"
                      size="sm"
                      aria-label={t('common.moveUp')}
                      disabled={index === 0 || reorder.isPending}
                      onClick={() => move(index, -1)}
                    >
                      <IconArrowUp size={14} />
                    </ActionIcon>
                    <ActionIcon
                      variant="subtle"
                      size="sm"
                      aria-label={t('common.moveDown')}
                      disabled={index === list.length - 1 || reorder.isPending}
                      onClick={() => move(index, 1)}
                    >
                      <IconArrowDown size={14} />
                    </ActionIcon>
                  </Group>
                </Table.Td>
                <Table.Td>
                  <Group gap="xs" wrap="nowrap">
                    <Text size="sm" c={platform.active ? undefined : 'dimmed'}>
                      {platform.name}
                    </Text>
                    {platform.igdb_platform_id !== null && platform.igdb_platform_id !== undefined && (
                      <Tooltip label={t('platforms.fromIgdb')}>
                        <Badge size="xs" variant="light">
                          {t('settings.igdb')}
                        </Badge>
                      </Tooltip>
                    )}
                  </Group>
                </Table.Td>
                <Table.Td>{platform.abbreviation}</Table.Td>
                <Table.Td>{platform.game_count}</Table.Td>
                <Table.Td>{platform.device_count}</Table.Td>
                <Table.Td>
                  <Switch
                    checked={platform.active}
                    aria-label={t('platforms.activeToggle', { name: platform.name })}
                    disabled={toggle.isPending}
                    onChange={() => toggle.mutate(platform)}
                  />
                </Table.Td>
                <Table.Td>
                  <Group gap={4} wrap="nowrap">
                    <ActionIcon
                      variant="subtle"
                      aria-label={t('common.edit')}
                      onClick={() => setEditing({ platform })}
                    >
                      <IconPencil size={16} />
                    </ActionIcon>
                    <Tooltip label={t('platforms.inUse')} disabled={!inUse(platform)}>
                      <span>
                        <ActionIcon
                          variant="subtle"
                          color="red"
                          aria-label={t('common.delete')}
                          disabled={inUse(platform)}
                          onClick={() => setDeleting(platform)}
                        >
                          <IconTrash size={16} />
                        </ActionIcon>
                      </span>
                    </Tooltip>
                  </Group>
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      </Table.ScrollContainer>

      {editing && <PlatformForm platform={editing.platform} onClose={() => setEditing(null)} />}

      <ConfirmModal
        opened={deleting !== null}
        title={t('platforms.deleteTitle', { name: deleting?.name ?? '' })}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => deleting && remove.mutate(deleting.id)}
        onClose={() => setDeleting(null)}
      />
    </Stack>
  );
}

function PlatformForm({ platform, onClose }: { platform?: Platform; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const form = useForm({
    initialValues: {
      name: platform?.name ?? '',
      abbreviation: platform?.abbreviation ?? '',
      active: platform?.active ?? true,
    },
    validate: { name: (value) => (value.trim() ? null : t('errors.required')) },
  });
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (body: PlatformInput) =>
      platform
        ? call(api.PUT('/api/platforms/{id}', { params: { path: { id: platform.id } }, body }))
        : call(api.POST('/api/platforms', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={platform ? t('platforms.edit') : t('platforms.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => save.mutate(values))}
    >
      <TextInput label={t('common.name')} required data-autofocus {...form.getInputProps('name')} />
      <TextInput label={t('platforms.abbreviation')} {...form.getInputProps('abbreviation')} />
      <Switch
        label={t('common.active')}
        description={t('platforms.activeDescription')}
        {...form.getInputProps('active', { type: 'checkbox' })}
      />
    </FormModal>
  );
}
