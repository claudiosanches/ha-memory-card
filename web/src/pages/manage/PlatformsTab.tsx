import { ActionIcon, Badge, Button, Group, Loader, Select, Stack, Text, TextInput, Title, Tooltip } from '@mantine/core';
import { useForm } from '@mantine/form';
import {
  IconArrowDown,
  IconArrowUp,
  IconMinus,
  IconPencil,
  IconPlus,
  IconSearch,
  IconTrash,
} from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, usePlatforms } from '../../api/queries';
import { PLATFORM_CATEGORIES, type Platform, type PlatformCategory, type PlatformInput } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import classes from './PlatformsTab.module.css';

type Editing = { platform?: Platform } | null;

function inUse(platform: Platform) {
  return platform.game_count > 0 || platform.device_count > 0;
}

/** Only platforms added by hand can go; IGDB's catalog stays complete. */
function deletable(platform: Platform) {
  return platform.igdb_platform_id == null && !inUse(platform);
}

function input(platform: Platform, active: boolean): PlatformInput {
  return { name: platform.name, abbreviation: platform.abbreviation, category: platform.category, active };
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

  const setActive = useMutation({
    mutationFn: ({ platform, active }: { platform: Platform; active: boolean }) =>
      call(
        api.PUT('/api/platforms/{id}', { params: { path: { id: platform.id } }, body: input(platform, active) }),
      ),
    onSuccess: () => void refresh(),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const remove = useMutation({
    mutationFn: (id: number) => call(api.DELETE('/api/platforms/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void refresh();
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (platforms.isPending) return <Loader size="sm" />;
  if (platforms.isError) return <Text c="red">{errorMessage(t, platforms.error)}</Text>;

  const active = platforms.data.filter((p) => p.active);
  const inactive = platforms.data.filter((p) => !p.active);

  // The API orders every platform; inactive ones keep their place after the active list.
  const move = (index: number, delta: -1 | 1) => {
    const ids = active.map((p) => p.id);
    const target = index + delta;
    const moved = ids[index];
    const other = ids[target];
    if (moved === undefined || other === undefined) return;
    ids[index] = other;
    ids[target] = moved;
    reorder.mutate([...ids, ...inactive.map((p) => p.id)]);
  };

  const actions = (platform: Platform) => (
    <>
      <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => setEditing({ platform })}>
        <IconPencil size={16} />
      </ActionIcon>
      {deletable(platform) && (
        <ActionIcon variant="subtle" color="red" aria-label={t('common.delete')} onClick={() => setDeleting(platform)}>
          <IconTrash size={16} />
        </ActionIcon>
      )}
    </>
  );

  return (
    <Stack gap="xl">
      <section className={classes.section}>
        <Group justify="space-between" align="flex-end" gap="sm">
          <div>
            <Title order={3} className={classes.heading}>
              {t('platforms.yourTitle')}
            </Title>
            <Text size="sm" c="dimmed">
              {t('platforms.yourText')}
            </Text>
          </div>
          <Button variant="default" leftSection={<IconPlus size={16} />} onClick={() => setEditing({})}>
            {t('platforms.add')}
          </Button>
        </Group>
        {active.length === 0 ? (
          <div className={classes.empty}>{t('platforms.yourEmpty')}</div>
        ) : (
          <ol className={classes.list}>
            {active.map((platform, index) => (
              <li key={platform.id} className={classes.row} data-testid={`platform-${platform.slug}`}>
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
                    disabled={index === active.length - 1 || reorder.isPending}
                    onClick={() => move(index, 1)}
                  >
                    <IconArrowDown size={14} />
                  </ActionIcon>
                </Group>
                <PlatformName platform={platform} />
                <Group gap={4} wrap="nowrap" className={classes.actions}>
                  {actions(platform)}
                  <Tooltip label={t('platforms.removeHint')}>
                    <ActionIcon
                      variant="subtle"
                      color="gray"
                      aria-label={t('platforms.remove', { name: platform.name })}
                      disabled={setActive.isPending}
                      onClick={() => setActive.mutate({ platform, active: false })}
                    >
                      <IconMinus size={16} />
                    </ActionIcon>
                  </Tooltip>
                </Group>
              </li>
            ))}
          </ol>
        )}
      </section>

      <Catalog
        platforms={inactive}
        busy={setActive.isPending}
        onAdd={(platform) => setActive.mutate({ platform, active: true })}
        actions={actions}
      />

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

function PlatformName({ platform }: { platform: Platform }) {
  const { t } = useTranslation();
  const details = [
    platform.abbreviation && platform.abbreviation !== platform.name ? platform.abbreviation : null,
    platform.generation ? t('platforms.generation', { generation: platform.generation }) : null,
    platform.game_count ? t('platforms.gameCount', { count: platform.game_count }) : null,
    platform.device_count ? t('platforms.deviceCount', { count: platform.device_count }) : null,
  ].filter(Boolean);
  return (
    <div className={classes.name}>
      <Text size="sm" fw={550} lineClamp={1}>
        {platform.name}
      </Text>
      {details.length > 0 && (
        <Text size="xs" c="dimmed" lineClamp={1}>
          {details.join(t('common.separator'))}
        </Text>
      )}
    </div>
  );
}

type CatalogProps = {
  platforms: Platform[];
  busy: boolean;
  onAdd: (platform: Platform) => void;
  actions: (platform: Platform) => React.ReactNode;
};

function Catalog({ platforms, busy, onAdd, actions }: CatalogProps) {
  const { t } = useTranslation();
  const [text, setText] = useState('');
  const [category, setCategory] = useState<PlatformCategory | null>(null);

  const needle = text.trim().toLowerCase();
  const matches = platforms.filter(
    (p) =>
      (!needle || p.name.toLowerCase().includes(needle) || p.abbreviation?.toLowerCase().includes(needle)) &&
      (!category || (p.category ?? 'other') === category),
  );
  const groups = PLATFORM_CATEGORIES.map((c) => ({
    category: c,
    items: matches.filter((p) => (p.category ?? 'other') === c),
  })).filter((g) => g.items.length > 0);

  return (
    <section className={classes.section}>
      <div>
        <Title order={3} className={classes.heading}>
          {t('platforms.catalogTitle')}
        </Title>
        <Text size="sm" c="dimmed">
          {t('platforms.catalogText')}
        </Text>
      </div>
      <TextInput
        leftSection={<IconSearch size={16} />}
        placeholder={t('platforms.search')}
        aria-label={t('platforms.search')}
        value={text}
        onChange={(event) => setText(event.currentTarget.value)}
      />
      <div className={classes.filters} role="group" aria-label={t('platforms.category')}>
        <button type="button" className={classes.filter} aria-pressed={!category} onClick={() => setCategory(null)}>
          {t('platforms.categories.all')}
        </button>
        {PLATFORM_CATEGORIES.map((c) => (
          <button
            type="button"
            key={c}
            className={classes.filter}
            aria-pressed={category === c}
            onClick={() => setCategory(c)}
          >
            {t(`platforms.categories.${c}`)}
          </button>
        ))}
      </div>
      {groups.length === 0 ? (
        <div className={classes.empty}>{t('platforms.noMatch')}</div>
      ) : (
        groups.map((group) => (
          <div key={group.category} className={classes.group}>
            <Text size="xs" fw={700} tt="uppercase" c="dimmed" className={classes.groupTitle}>
              {t(`platforms.categories.${group.category}`)}
            </Text>
            <ul className={classes.grid}>
              {group.items.map((platform) => (
                <li key={platform.id} className={classes.row} data-testid={`platform-${platform.slug}`}>
                  <PlatformName platform={platform} />
                  <Group gap={4} wrap="nowrap" className={classes.actions}>
                    {platform.igdb_platform_id == null && actions(platform)}
                    {inUse(platform) && (
                      <Badge size="xs" variant="light" color="yellow" tt="none">
                        {t('platforms.hiddenInUse')}
                      </Badge>
                    )}
                    <Button
                      size="compact-sm"
                      variant="light"
                      leftSection={<IconPlus size={14} />}
                      aria-label={t('platforms.addNamed', { name: platform.name })}
                      disabled={busy}
                      onClick={() => onAdd(platform)}
                    >
                      {t('platforms.enable')}
                    </Button>
                  </Group>
                </li>
              ))}
            </ul>
          </div>
        ))
      )}
    </section>
  );
}

function PlatformForm({ platform, onClose }: { platform?: Platform; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const form = useForm({
    initialValues: {
      name: platform?.name ?? '',
      abbreviation: platform?.abbreviation ?? '',
      category: (platform?.category ?? '') as PlatformCategory | '',
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
      onSubmit={form.onSubmit((values) =>
        save.mutate({
          name: values.name,
          abbreviation: values.abbreviation || null,
          category: values.category || null,
          active: platform?.active ?? true,
        }),
      )}
    >
      <TextInput label={t('common.name')} required data-autofocus {...form.getInputProps('name')} />
      <TextInput label={t('platforms.abbreviation')} {...form.getInputProps('abbreviation')} />
      <Select
        label={t('platforms.category')}
        data={PLATFORM_CATEGORIES.map((c) => ({ value: c, label: t(`platforms.categories.${c}`) }))}
        clearable
        {...form.getInputProps('category')}
      />
    </FormModal>
  );
}
