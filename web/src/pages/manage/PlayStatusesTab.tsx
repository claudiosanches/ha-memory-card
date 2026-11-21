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
import { ActionIcon, Button, ColorSwatch, Group, Input, Loader, Modal, Select, Stack, Text, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconCheck, IconGripVertical, IconPencil, IconPlus, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { type ReactNode, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys } from '../../api/queries';
import {
  type BuiltinStatus,
  type PlayStatusFields,
  STATUS_COLORS,
  STATUS_GROUPS,
  type StatusColor,
  type StatusGroup,
} from '../../api/types';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { type NamedStatus, usePlayStatusList } from '../../components/playStatuses';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import classes from './PlatformsTab.module.css';

type Editing = { status?: NamedStatus } | null;

export function PlayStatusesTab() {
  const { t } = useTranslation();
  const statuses = usePlayStatusList();
  const [editing, setEditing] = useState<Editing>(null);
  const [deleting, setDeleting] = useState<NamedStatus | null>(null);
  // The order just dropped, shown until the server's list replaces `base`.
  const [dropped, setDropped] = useState<{ keys: string[]; base: NamedStatus[] } | null>(null);
  const queryClient = useQueryClient();
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const reorder = useMutation({
    mutationFn: (keys: string[]) => call(api.PUT('/api/play-statuses/order', { body: { keys } })),
    onSuccess: (list) => queryClient.setQueryData(queryKeys.playStatuses, list),
    onError: (error) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.playStatuses });
      notifyError(errorMessage(t, error));
    },
  });

  if (statuses.isPending) return <Loader size="sm" />;

  const list =
    dropped?.base === statuses.list
      ? dropped.keys.flatMap((key) => statuses.list.find((s) => s.key === key) ?? [])
      : statuses.list;
  const name = (key: UniqueIdentifier) => statuses.name(String(key));
  const position = (key: UniqueIdentifier) => list.findIndex((s) => s.key === key) + 1;

  const onDragEnd = ({ active, over }: DragEndEvent) => {
    if (!over || active.id === over.id) return;
    const keys = arrayMove(list, position(active.id) - 1, position(over.id) - 1).map((s) => s.key);
    setDropped({ keys, base: statuses.list });
    reorder.mutate(keys);
  };

  return (
    <section className={classes.section}>
      <Group justify="space-between" align="flex-end" gap="sm">
        <Text size="sm" c="dimmed" maw={640}>
          {t('playStatuses.intro')}
        </Text>
        <Button variant="default" leftSection={<IconPlus size={16} />} onClick={() => setEditing({})}>
          {t('playStatuses.add')}
        </Button>
      </Group>

      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={onDragEnd}
        accessibility={{
          screenReaderInstructions: { draggable: t('playStatuses.drag.instructions') },
          announcements: {
            onDragStart: ({ active }) => t('playStatuses.drag.start', { name: name(active.id) }),
            onDragOver: ({ active, over }) =>
              over ? t('playStatuses.drag.over', { name: name(active.id), position: position(over.id) }) : undefined,
            onDragEnd: ({ active, over }) =>
              over ? t('playStatuses.drag.end', { name: name(active.id), position: position(over.id) }) : undefined,
            onDragCancel: ({ active }) => t('playStatuses.drag.cancel', { name: name(active.id) }),
          },
        }}
      >
        <SortableContext items={list.map((s) => s.key)} strategy={verticalListSortingStrategy}>
          <ol className={classes.list}>
            {list.map((status) => (
              <SortableRow key={status.key} status={status}>
                <ColorSwatch color={`var(--mantine-color-${status.color}-filled)`} size={14} aria-hidden />
                <div className={classes.name}>
                  <Text fw={600}>{status.name}</Text>
                  <Text size="xs" c="dimmed">
                    {[
                      t('playStatuses.summary', { group: t(`playStatuses.group.${status.counts_as}`) }),
                      t('platforms.gameCount', { count: status.game_count }),
                      status.builtin ? t('playStatuses.builtin') : null,
                    ]
                      .filter(Boolean)
                      .join(t('common.separator'))}
                  </Text>
                </div>
                <Group gap={4} wrap="nowrap" className={classes.actions}>
                  <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => setEditing({ status })}>
                    <IconPencil size={16} />
                  </ActionIcon>
                  {!status.builtin && (
                    <ActionIcon
                      variant="subtle"
                      color="red"
                      aria-label={t('common.delete')}
                      onClick={() => setDeleting(status)}
                    >
                      <IconTrash size={16} />
                    </ActionIcon>
                  )}
                </Group>
              </SortableRow>
            ))}
          </ol>
        </SortableContext>
      </DndContext>

      {editing && <StatusForm status={editing.status} onClose={() => setEditing(null)} />}
      {deleting && (
        <DeleteModal
          status={deleting}
          others={statuses.list.filter((s) => s.key !== deleting.key)}
          onClose={() => setDeleting(null)}
        />
      )}
    </section>
  );
}

function SortableRow({ status, children }: { status: NamedStatus; children: ReactNode }) {
  const { t } = useTranslation();
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } = useSortable({
    id: status.key,
  });
  return (
    <li
      ref={setNodeRef}
      className={classes.row}
      data-dragging={isDragging || undefined}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      data-testid={`status-${status.key}`}
    >
      <ActionIcon
        ref={setActivatorNodeRef}
        variant="subtle"
        color="gray"
        className={classes.handle}
        {...attributes}
        {...listeners}
        aria-label={t('playStatuses.drag.handle', { name: status.name })}
      >
        <IconGripVertical size={16} />
      </ActionIcon>
      {children}
    </li>
  );
}

type FormValues = { label: string; color: StatusColor; counts_as: StatusGroup };

function StatusForm({ status, onClose }: { status?: NamedStatus; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const builtin = status?.builtin ?? false;
  const form = useForm<FormValues>({
    initialValues: {
      label: status?.label ?? '',
      color: status?.color ?? 'teal',
      counts_as: status?.counts_as ?? 'backlog',
    },
    validate: { label: (value) => (builtin || value.trim() ? null : t('errors.required')) },
  });
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (body: PlayStatusFields) =>
      status
        ? call(api.PUT('/api/play-statuses/{key}', { params: { path: { key: status.key } }, body }))
        : call(api.POST('/api/play-statuses', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.playStatuses });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={status ? t('playStatuses.edit') : t('playStatuses.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => save.mutate({ ...values, label: values.label.trim() || null }))}
    >
      <TextInput
        label={t('playStatuses.name')}
        required={!builtin}
        placeholder={builtin && status ? t(`playStatus.${status.key as BuiltinStatus}`) : undefined}
        description={
          builtin && status
            ? t('playStatuses.defaultName', { name: t(`playStatus.${status.key as BuiltinStatus}`) })
            : undefined
        }
        data-autofocus
        {...form.getInputProps('label')}
      />
      <Input.Wrapper label={t('playStatuses.color')}>
        <Group gap={8} mt={6} role="radiogroup" aria-label={t('playStatuses.color')}>
          {STATUS_COLORS.map((color) => {
            const selected = form.values.color === color;
            return (
              <ColorSwatch
                key={color}
                component="button"
                type="button"
                role="radio"
                aria-checked={selected}
                aria-label={t(`playStatuses.colors.${color}`)}
                color={`var(--mantine-color-${color}-filled)`}
                onClick={() => form.setFieldValue('color', color)}
                className={classes.swatch}
              >
                {selected && <IconCheck size={14} color="white" />}
              </ColorSwatch>
            );
          })}
        </Group>
      </Input.Wrapper>
      <Select
        label={t('playStatuses.countsAs')}
        description={builtin ? t('playStatuses.builtinGroup') : t('playStatuses.countsAsHint')}
        data={STATUS_GROUPS.map((group) => ({ value: group, label: t(`playStatuses.group.${group}`) }))}
        allowDeselect={false}
        disabled={builtin}
        {...form.getInputProps('counts_as')}
      />
    </FormModal>
  );
}

function DeleteModal({ status, others, onClose }: { status: NamedStatus; others: NamedStatus[]; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [moveTo, setMoveTo] = useState<string | null>(null);

  const remove = useMutation({
    mutationFn: () =>
      call(
        api.DELETE('/api/play-statuses/{key}', {
          params: { path: { key: status.key }, query: moveTo ? { move_to: moveTo } : {} },
        }),
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.playStatuses });
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      void queryClient.invalidateQueries({ queryKey: ['game'] });
      notifySuccess(t('common.deleted'));
      onClose();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  return (
    <Modal opened onClose={onClose} title={t('playStatuses.deleteTitle', { name: status.name })} centered>
      <Stack>
        {status.game_count > 0 && (
          <Select
            label={t('playStatuses.moveTo', { count: status.game_count })}
            placeholder={t('common.notSet')}
            data={others.map((s) => ({ value: s.key, label: s.name }))}
            value={moveTo}
            onChange={setMoveTo}
            clearable
          />
        )}
        <Group justify="flex-end">
          <Button variant="default" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button color="red" onClick={() => remove.mutate()} loading={remove.isPending}>
            {t('common.delete')}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
