import {
  closestCenter,
  DndContext,
  type DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  type UniqueIdentifier,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { ActionIcon, Badge, Button, Group, Loader, Select, Stack, Text, TextInput, Tooltip } from '@mantine/core';
import { useForm } from '@mantine/form';
import {
  IconGripVertical,
  IconMinus,
  IconPencil,
  IconPlus,
  IconSearch,
  IconTrash,
} from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { type ReactNode, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, usePlatforms } from '../../api/queries';
import { PLATFORM_CATEGORIES, type Platform, type PlatformCategory, type PlatformInput } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { SlugInput, slugRule, useSlugInputs } from '../../components/SlugInput';
import { trimSlug } from '../../slug';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { SectionHeading } from '../../components/SectionHeading';
import { FilterBar, FilterButton } from '../../components/FilterBar';
import { ManageToolbar } from './ManageToolbar';
import classes from './PlatformsTab.module.css';

type Editing = { platform?: Platform } | null;

function inUse(platform: Platform) {
  return platform.game_count > 0 || platform.device_count > 0 || platform.account_count > 0;
}

/** Only platforms added by hand or by an import can go; the catalog stays complete. */
function deletable(platform: Platform) {
  return platform.custom && !inUse(platform);
}

/** Every platform id in the new order after moving one of the active ones. */
export function reorderedIds(active: Platform[], inactive: Platform[], from: number, to: number): number[] {
  return [...arrayMove(active, from, to), ...inactive].map((p) => p.id);
}

function input(platform: Platform, active: boolean): PlatformInput {
  return { name: platform.name, abbreviation: platform.abbreviation, category: platform.category, active };
}

export function PlatformsTab() {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const [editing, setEditing] = useState<Editing>(null);
  const [deleting, setDeleting] = useState<Platform | null>(null);
  // The order just dropped, shown until the server's list replaces `base`.
  // Query cache updates render a tick late, which would make the row jump back first.
  const [dropped, setDropped] = useState<{ ids: number[]; base: Platform[] } | null>(null);
  const queryClient = useQueryClient();
  const refresh = () => queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

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

  const serverActive = platforms.data.filter((p) => p.active);
  const active =
    dropped?.base === platforms.data
      ? dropped.ids.flatMap((id) => serverActive.find((p) => p.id === id) ?? [])
      : serverActive;
  const inactive = platforms.data.filter((p) => !p.active);

  const all = platforms.data;
  const name = (id: UniqueIdentifier) => all.find((p) => p.id === id)?.name ?? '';
  const position = (id: UniqueIdentifier) => active.findIndex((p) => p.id === id) + 1;

  const onDragEnd = ({ active: dragged, over }: DragEndEvent) => {
    if (!over || dragged.id === over.id) return;
    const ids = reorderedIds(active, inactive, position(dragged.id) - 1, position(over.id) - 1);
    setDropped({ ids, base: platforms.data });
    reorder.mutate(ids);
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
    <Stack gap="section">
      <section className={classes.section}>
        <ManageToolbar addLabel={t('platforms.add')} onAdd={() => setEditing({})}>
          <SectionHeading
            title={t('platforms.yourTitle')}
            description={t('platforms.yourText')}
            titleClassName={classes.heading}
          />
        </ManageToolbar>
        {active.length === 0 ? (
          <div className={classes.empty}>{t('platforms.yourEmpty')}</div>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={onDragEnd}
            accessibility={{
              screenReaderInstructions: { draggable: t('platforms.drag.instructions') },
              announcements: {
                onDragStart: ({ active: a }) => t('platforms.drag.start', { name: name(a.id) }),
                onDragOver: ({ active: a, over }) =>
                  over ? t('platforms.drag.over', { name: name(a.id), position: position(over.id) }) : undefined,
                onDragEnd: ({ active: a, over }) =>
                  over ? t('platforms.drag.end', { name: name(a.id), position: position(over.id) }) : undefined,
                onDragCancel: ({ active: a }) => t('platforms.drag.cancel', { name: name(a.id) }),
              },
            }}
          >
            <SortableContext items={active.map((p) => p.id)} strategy={verticalListSortingStrategy}>
              <ol className={classes.list}>
                {active.map((platform) => (
                  <SortableRow key={platform.id} platform={platform}>
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
                  </SortableRow>
                ))}
              </ol>
            </SortableContext>
          </DndContext>
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

function SortableRow({ platform, children }: { platform: Platform; children: ReactNode }) {
  const { t } = useTranslation();
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } = useSortable({
    id: platform.id,
  });
  return (
    <li
      ref={setNodeRef}
      className={classes.row}
      data-dragging={isDragging || undefined}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      data-testid={`platform-${platform.slug}`}
    >
      <ActionIcon
        ref={setActivatorNodeRef}
        variant="subtle"
        color="gray"
        className={classes.handle}
        {...attributes}
        {...listeners}
        aria-label={t('platforms.drag.handle', { name: platform.name })}
      >
        <IconGripVertical size={16} />
      </ActionIcon>
      {children}
    </li>
  );
}

function PlatformName({ platform }: { platform: Platform }) {
  const { t } = useTranslation();
  const details = [
    platform.abbreviation && platform.abbreviation !== platform.name ? platform.abbreviation : null,
    platform.generation ? t('platforms.generation', { generation: platform.generation }) : null,
    platform.game_count ? t('platforms.gameCount', { count: platform.game_count }) : null,
    platform.device_count ? t('platforms.deviceCount', { count: platform.device_count }) : null,
    platform.account_count ? t('platforms.accountCount', { count: platform.account_count }) : null,
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
      <SectionHeading
        title={t('platforms.catalogTitle')}
        description={t('platforms.catalogText')}
        titleClassName={classes.heading}
      />
      <TextInput
        leftSection={<IconSearch size={16} />}
        placeholder={t('platforms.search')}
        aria-label={t('platforms.search')}
        value={text}
        onChange={(event) => setText(event.currentTarget.value)}
      />
      <FilterBar label={t('platforms.category')}>
        <FilterButton selected={!category} onClick={() => setCategory(null)}>
          {t('platforms.categories.all')}
        </FilterButton>
        {PLATFORM_CATEGORIES.map((c) => (
          <FilterButton
            key={c}
            selected={category === c}
            onClick={() => setCategory(c)}
          >
            {t(`platforms.categories.${c}`)}
          </FilterButton>
        ))}
      </FilterBar>
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
                    {platform.custom && actions(platform)}
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
      slug: platform?.slug ?? '',
      abbreviation: platform?.abbreviation ?? '',
      category: (platform?.category ?? '') as PlatformCategory | '',
    },
    validate: {
      name: (value) => (value.trim() ? null : t('errors.required')),
      slug: slugRule(t, platform !== undefined),
    },
  });
  const inputs = useSlugInputs(form, platform?.slug);
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
          slug: trimSlug(values.slug) || null,
          abbreviation: values.abbreviation || null,
          category: values.category || null,
          active: platform?.active ?? true,
        }),
      )}
    >
      <TextInput label={t('common.name')} required data-autofocus {...inputs.name} />
      <SlugInput param="platform" saved={platform?.slug} {...inputs.slug} />
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
