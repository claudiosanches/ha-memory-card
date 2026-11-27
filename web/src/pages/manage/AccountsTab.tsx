import { ActionIcon, Badge, Group, Loader, MultiSelect, Select, Stack, Switch, Text, Textarea, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconPencil, IconTrash, IconUserCircle } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useAccounts, usePeople, usePlatforms, useStores } from '../../api/queries';
import type { Account, AccountInput, Platform } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { EmptyState } from '../../components/EmptyState';
import { FormModal } from '../../components/FormModal';
import { notifyError, notifySuccess } from '../../components/notify';
import { SlugInput, slugRule, useSlugInputs } from '../../components/SlugInput';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { trimSlug } from '../../slug';
import { ManageToolbar } from './ManageToolbar';
import classes from './ManageList.module.css';

/** The store accounts digital copies live on, such as a Nintendo Account or Steam. */
export function AccountsTab() {
  const { t } = useTranslation();
  const accounts = useAccounts();
  const platforms = usePlatforms();
  const stores = useStores();
  const [editing, setEditing] = useState<{ account?: Account } | null>(null);
  const [deleting, setDeleting] = useState<Account | null>(null);
  const queryClient = useQueryClient();

  const platformsById = useMemo(
    () => new Map((platforms.data ?? []).map((platform) => [platform.id, platform])),
    [platforms.data],
  );

  const remove = useMutation({
    mutationFn: (id: number) => call(api.DELETE('/api/accounts/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.accounts });
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      // Its copies lose their account.
      void queryClient.invalidateQueries({ queryKey: queryKeys.missing });
      void queryClient.invalidateQueries({ queryKey: queryKeys.duplicates });
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (accounts.isPending || platforms.isPending) return <Loader size="sm" />;
  if (accounts.isError) return <Text c="red">{errorMessage(t, accounts.error)}</Text>;
  if (platforms.isError) return <Text c="red">{errorMessage(t, platforms.error)}</Text>;

  return (
    <Stack>
      <ManageToolbar addLabel={t('accounts.add')} onAdd={() => setEditing({})} />

      {accounts.data.length === 0 ? (
        <EmptyState icon={IconUserCircle} title={t('accounts.emptyTitle')} description={t('accounts.empty')} />
      ) : (
        <ul className={classes.grid}>
          {accounts.data.map((account) => {
            const store = stores.data?.find((s) => s.id === account.store_id)?.name;
            const about = [store, account.person && (account.person.name ?? account.person.id)].filter(Boolean);
            return (
            <li key={account.id} className={classes.card} data-testid={`account-${account.name}`}>
              <div className={classes.top}>
                <div className={classes.main}>
                  <Group gap="xs">
                    <Text className={classes.name}>{account.name}</Text>
                    {!account.active && (
                      <Badge size="xs" color="gray" variant="light">
                        {t('common.inactive')}
                      </Badge>
                    )}
                  </Group>
                  {account.username && (
                    <Text size="sm" c="dimmed" className={classes.username}>
                      {account.username}
                    </Text>
                  )}
                  {about.length > 0 && (
                    <Text size="sm" c="dimmed">
                      {about.join(t('common.separator'))}
                    </Text>
                  )}
                  {account.notes && (
                    <Text size="xs" c="dimmed" className={classes.notes}>
                      {account.notes}
                    </Text>
                  )}
                </div>
                <Group gap={4} wrap="nowrap" className={classes.actions}>
                  <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => setEditing({ account })}>
                    <IconPencil size={16} />
                  </ActionIcon>
                  <ActionIcon
                    variant="subtle"
                    color="red"
                    aria-label={t('common.delete')}
                    onClick={() => setDeleting(account)}
                  >
                    <IconTrash size={16} />
                  </ActionIcon>
                </Group>
              </div>
              <div className={classes.footer}>
                <Group gap={4} aria-label={t('accounts.platforms')}>
                  {account.platform_ids.map((id) => {
                    const platform = platformsById.get(id);
                    return platform ? (
                      <Badge key={id} variant="outline" size="sm" tt="none">
                        {platform.abbreviation ?? platform.name}
                      </Badge>
                    ) : null;
                  })}
                </Group>
                <Text size="xs" c="dimmed" className={classes.count}>
                  {t('common.copyCount', { count: account.copy_count })}
                </Text>
              </div>
            </li>
            );
          })}
        </ul>
      )}

      {editing && <AccountForm account={editing.account} platforms={platforms.data} onClose={() => setEditing(null)} />}

      <ConfirmModal
        opened={deleting !== null}
        title={t('accounts.deleteTitle', { name: deleting?.name ?? '' })}
        message={deleting?.copy_count ? t('accounts.deleteCopies', { count: deleting.copy_count }) : undefined}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => deleting && remove.mutate(deleting.id)}
        onClose={() => setDeleting(null)}
      />
    </Stack>
  );
}

function AccountForm({
  account,
  platforms,
  onClose,
}: {
  account?: Account;
  platforms: Platform[];
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const stores = useStores();
  const people = usePeople();
  const form = useForm({
    initialValues: {
      name: account?.name ?? '',
      slug: account?.slug ?? '',
      username: account?.username ?? '',
      store_id: account?.store_id ? String(account.store_id) : '',
      user_id: account?.person?.id ?? '',
      notes: account?.notes ?? '',
      active: account?.active ?? true,
      platform_ids: (account?.platform_ids ?? []).map(String),
    },
    validate: {
      name: (value) => (value.trim() ? null : t('errors.required')),
      slug: slugRule(t, account !== undefined),
    },
  });
  const inputs = useSlugInputs(form, account?.slug);
  const onApiError = useApiErrorHandler(form);

  // Inactive platforms stay listed when the account already uses them.
  const platformOptions = platforms
    .filter((platform) => platform.active || account?.platform_ids.includes(platform.id))
    .map((platform) => ({ value: String(platform.id), label: platform.name }));

  const save = useMutation({
    mutationFn: (body: AccountInput) =>
      account
        ? call(api.PUT('/api/accounts/{id}', { params: { path: { id: account.id } }, body }))
        : call(api.POST('/api/accounts', { body })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.accounts });
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      notifySuccess(t('common.saved'));
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={account ? t('accounts.edit') : t('accounts.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((values) =>
        save.mutate({
          name: values.name,
          slug: trimSlug(values.slug) || null,
          username: values.username.trim() || null,
          store_id: values.store_id ? Number(values.store_id) : null,
          user_id: values.user_id || null,
          notes: values.notes.trim() || null,
          active: values.active,
          platform_ids: values.platform_ids.map(Number),
        }),
      )}
    >
      <TextInput
        label={t('common.name')}
        description={t('accounts.nameDescription')}
        placeholder={t('accounts.namePlaceholder')}
        required
        data-autofocus
        {...inputs.name}
      />
      <SlugInput param="account" saved={account?.slug} {...inputs.slug} />
      <TextInput
        label={t('accounts.username')}
        description={t('accounts.usernameDescription')}
        placeholder={t('accounts.usernamePlaceholder')}
        autoComplete="off"
        {...form.getInputProps('username')}
      />
      <Select
        label={t('accounts.store')}
        description={t('accounts.storeDescription')}
        data={(stores.data ?? []).map((s) => ({ value: String(s.id), label: s.name }))}
        clearable
        searchable
        {...form.getInputProps('store_id')}
        value={form.values.store_id || null}
      />
      <Select
        label={t('accounts.person')}
        description={t('accounts.personDescription')}
        data={(people.data ?? []).map((p) => ({ value: p.id, label: p.name ?? p.id }))}
        clearable
        searchable
        {...form.getInputProps('user_id')}
        value={form.values.user_id || null}
      />
      <MultiSelect
        label={t('accounts.platforms')}
        description={t('accounts.platformsDescription')}
        data={platformOptions}
        searchable
        clearable
        {...form.getInputProps('platform_ids')}
      />
      <Textarea label={t('common.notes')} autosize minRows={2} {...form.getInputProps('notes')} />
      <Switch
        label={t('common.active')}
        description={t('accounts.activeDescription')}
        {...form.getInputProps('active', { type: 'checkbox' })}
      />
    </FormModal>
  );
}
