import { ActionIcon, Badge, Group, Loader, Stack, Text, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconPencil, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useLabels } from '../../api/queries';
import type { Label, LabelInput, StatusColor } from '../../api/types';
import { ColorPicker } from '../../components/ColorPicker';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { SlugInput, slugRule, useSlugInputs } from '../../components/SlugInput';
import { trimSlug } from '../../slug';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { ManageToolbar } from './ManageToolbar';
import classes from './ManageList.module.css';

type Editing = { label?: Label } | null;

export function LabelsTab() {
  const { t } = useTranslation();
  const labels = useLabels();
  const [editing, setEditing] = useState<Editing>(null);
  const [deleting, setDeleting] = useState<Label | null>(null);
  const queryClient = useQueryClient();

  const remove = useMutation({
    mutationFn: (id: number) => call(api.DELETE('/api/labels/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.labels });
      void queryClient.invalidateQueries({ queryKey: ['game'] });
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (labels.isPending) return <Loader size="sm" />;
  if (labels.isError) return <Text c="red">{errorMessage(t, labels.error)}</Text>;

  return (
    <Stack>
      <ManageToolbar addLabel={t('labels.add')} onAdd={() => setEditing({})} />

      {labels.data.length === 0 ? (
        <div className={classes.empty}>{t('labels.empty')}</div>
      ) : (
        <ul className={classes.grid}>
          {labels.data.map((label) => (
            <li key={label.id} className={classes.card} data-testid={`label-${label.name}`}>
              <div className={classes.top}>
                <div className={classes.main}>
                  <div>
                    <Badge variant="light" color={label.color} tt="none" size="lg" className={classes.name}>
                      {label.name}
                    </Badge>
                  </div>
                  <Text size="sm" c="dimmed">
                    {t('labels.gameCount', { count: label.game_count })}
                  </Text>
                </div>
                <Group gap={4} wrap="nowrap" className={classes.actions}>
                  <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => setEditing({ label })}>
                    <IconPencil size={16} />
                  </ActionIcon>
                  <ActionIcon variant="subtle" color="red" aria-label={t('common.delete')} onClick={() => setDeleting(label)}>
                    <IconTrash size={16} />
                  </ActionIcon>
                </Group>
              </div>
            </li>
          ))}
        </ul>
      )}

      {editing && <LabelForm label={editing.label} onClose={() => setEditing(null)} />}

      <ConfirmModal
        opened={deleting !== null}
        title={t('labels.deleteTitle', { name: deleting?.name ?? '' })}
        message={deleting?.game_count ? t('labels.deleteGames', { count: deleting.game_count }) : undefined}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => deleting && remove.mutate(deleting.id)}
        onClose={() => setDeleting(null)}
      />
    </Stack>
  );
}

function LabelForm({ label, onClose }: { label?: Label; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const form = useForm({
    initialValues: { name: label?.name ?? '', slug: label?.slug ?? '', color: label?.color ?? ('gray' as StatusColor) },
    validate: {
      name: (value) =>
        !value.trim() ? t('errors.required') : /[,;]/.test(value) ? t('errors.label_separator') : null,
      slug: slugRule(t, label !== undefined),
    },
  });
  const inputs = useSlugInputs(form, label?.slug);
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (body: LabelInput) =>
      label
        ? call(api.PUT('/api/labels/{id}', { params: { path: { id: label.id } }, body }))
        : call(api.POST('/api/labels', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.labels });
      // Game pages show the label's name and color.
      void queryClient.invalidateQueries({ queryKey: ['game'] });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={label ? t('labels.edit') : t('labels.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => save.mutate({ ...values, slug: trimSlug(values.slug) || null }))}
    >
      <TextInput label={t('common.name')} required data-autofocus {...inputs.name} />
      <SlugInput param="label" saved={label?.slug} {...inputs.slug} />
      <ColorPicker
        label={t('labels.color')}
        value={form.values.color}
        onChange={(color) => form.setFieldValue('color', color)}
      />
    </FormModal>
  );
}
