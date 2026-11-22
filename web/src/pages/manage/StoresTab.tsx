import { ActionIcon, Group, Loader, Stack, Text, Textarea, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconPencil, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useStores } from '../../api/queries';
import type { Store, StoreInput } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { SlugInput, slugRule, useSlugInputs } from '../../components/SlugInput';
import { trimSlug } from '../../slug';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { ManageToolbar } from './ManageToolbar';
import classes from './ManageList.module.css';

type Editing = { store?: Store } | null;

export function StoresTab() {
  const { t } = useTranslation();
  const stores = useStores();
  const [editing, setEditing] = useState<Editing>(null);
  const [deleting, setDeleting] = useState<Store | null>(null);
  const queryClient = useQueryClient();

  const remove = useMutation({
    mutationFn: (id: number) =>
      call(api.DELETE('/api/stores/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.stores });
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (stores.isPending) return <Loader size="sm" />;
  if (stores.isError) return <Text c="red">{errorMessage(t, stores.error)}</Text>;

  return (
    <Stack>
      <ManageToolbar addLabel={t('stores.add')} onAdd={() => setEditing({})} />

      {stores.data.length === 0 ? (
        <div className={classes.empty}>{t('stores.empty')}</div>
      ) : (
        <ul className={classes.grid}>
          {stores.data.map((store) => (
            <li key={store.id} className={classes.card} data-testid={`store-${store.name}`}>
              <div className={classes.top}>
                <div className={classes.main}>
                  <Text className={classes.name}>{store.name}</Text>
                  <Text size="sm" c="dimmed">
                    {t('common.copyCount', { count: store.copy_count })}
                  </Text>
                  {store.notes && (
                    <Text size="sm" c="dimmed" lineClamp={3} className={classes.notes}>
                      {store.notes}
                    </Text>
                  )}
                </div>
                <Group gap={4} wrap="nowrap" className={classes.actions}>
                  <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => setEditing({ store })}>
                    <IconPencil size={16} />
                  </ActionIcon>
                  <ActionIcon
                    variant="subtle"
                    color="red"
                    aria-label={t('common.delete')}
                    onClick={() => setDeleting(store)}
                  >
                    <IconTrash size={16} />
                  </ActionIcon>
                </Group>
              </div>
            </li>
          ))}
        </ul>
      )}

      {editing && <StoreForm store={editing.store} onClose={() => setEditing(null)} />}

      <ConfirmModal
        opened={deleting !== null}
        title={t('stores.deleteTitle', { name: deleting?.name ?? '' })}
        message={deleting?.copy_count ? t('stores.deleteCopies', { count: deleting.copy_count }) : undefined}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => deleting && remove.mutate(deleting.id)}
        onClose={() => setDeleting(null)}
      />
    </Stack>
  );
}

function StoreForm({ store, onClose }: { store?: Store; onClose: () => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const form = useForm({
    initialValues: { name: store?.name ?? '', slug: store?.slug ?? '', notes: store?.notes ?? '' },
    validate: {
      name: (value) => (value.trim() ? null : t('errors.required')),
      slug: slugRule(t, store !== undefined),
    },
  });
  const inputs = useSlugInputs(form, store?.slug);
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (body: StoreInput) =>
      store
        ? call(api.PUT('/api/stores/{id}', { params: { path: { id: store.id } }, body }))
        : call(api.POST('/api/stores', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.stores });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={store ? t('stores.edit') : t('stores.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) => save.mutate({ ...values, slug: trimSlug(values.slug) || null }))}
    >
      <TextInput label={t('common.name')} required data-autofocus {...inputs.name} />
      <SlugInput param="store" saved={store?.slug} {...inputs.slug} />
      <Textarea label={t('common.notes')} autosize minRows={2} {...form.getInputProps('notes')} />
    </FormModal>
  );
}
