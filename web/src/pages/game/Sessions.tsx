import { ActionIcon, Button, Group, NumberInput, Select, Stack, Text, Textarea, TextInput } from '@mantine/core';
import { useForm } from '@mantine/form';
import { IconPencil, IconTrash } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import type { GameDetail, PlaySession } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { FormModal } from '../../components/FormModal';
import { notifyError } from '../../components/notify';
import { today } from '../../components/status';
import { useFormatDate } from '../../hooks/useFormatDate';
import { useApiErrorHandler } from '../../hooks/useApiForm';
import { numberOrNull, useGameSaved } from './forms';
import classes from './Sessions.module.css';

/** Sessions shown before "Show all". */
const SHOWN = 5;

type Props = {
  game: GameDetail;
  /** Owned copies a session can name, by ID. */
  copies: Map<number, string>;
  canTrack: boolean;
};

type Dialog = { kind: 'edit'; session?: PlaySession } | { kind: 'delete'; session: PlaySession } | null;

export function Sessions({ game, copies, canTrack }: Props) {
  const { t } = useTranslation();
  const formatDate = useFormatDate();
  const [dialog, setDialog] = useState<Dialog>(null);
  const [all, setAll] = useState(false);
  const close = () => setDialog(null);
  const saved = useGameSaved();

  const remove = useMutation({
    mutationFn: (id: number) => call(api.DELETE('/api/sessions/{id}', { params: { path: { id } } })),
    onSuccess: (updated) => {
      saved(updated, t('common.deleted'));
      close();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const shown = all ? game.sessions : game.sessions.slice(0, SHOWN);
  const describe = (session: PlaySession) =>
    [
      session.copy_id ? copies.get(session.copy_id) : null,
      session.minutes ? t('sessions.minutes', { count: session.minutes }) : null,
    ]
      .filter(Boolean)
      .join(t('common.separator'));

  return (
    <Stack gap="xs" data-testid="sessions">
      <Group justify="space-between" wrap="nowrap">
        <Text size="sm" fw={600}>
          {t('sessions.title')}
        </Text>
        {canTrack && (
          <Button size="compact-sm" variant="subtle" onClick={() => setDialog({ kind: 'edit' })}>
            {t('sessions.add')}
          </Button>
        )}
      </Group>
      {game.sessions.length === 0 ? (
        <Text size="sm" c="dimmed">
          {t('sessions.none')}
        </Text>
      ) : (
        <ul className={classes.list}>
          {shown.map((session) => (
            <li key={session.id} className={classes.session} data-testid={`session-${session.id}`}>
              <div className={classes.details}>
                <Text size="sm" fw={500}>
                  {formatDate(session.played_on)}
                </Text>
                {describe(session) && (
                  <Text size="xs" c="dimmed">
                    {describe(session)}
                  </Text>
                )}
                {session.notes && (
                  <Text size="xs" c="dimmed" lineClamp={2}>
                    {session.notes}
                  </Text>
                )}
              </div>
              {canTrack && (
                <Group gap={4} wrap="nowrap">
                  <ActionIcon
                    variant="subtle"
                    aria-label={t('sessions.edit')}
                    onClick={() => setDialog({ kind: 'edit', session })}
                  >
                    <IconPencil size={16} />
                  </ActionIcon>
                  <ActionIcon
                    variant="subtle"
                    color="red"
                    aria-label={t('sessions.delete')}
                    onClick={() => setDialog({ kind: 'delete', session })}
                  >
                    <IconTrash size={16} />
                  </ActionIcon>
                </Group>
              )}
            </li>
          ))}
        </ul>
      )}
      {game.sessions.length > SHOWN && (
        <Button size="compact-sm" variant="subtle" w="fit-content" onClick={() => setAll(!all)}>
          {all ? t('sessions.showFewer') : t('sessions.showAll', { count: game.sessions.length })}
        </Button>
      )}

      {dialog?.kind === 'edit' && (
        <SessionForm game={game} session={dialog.session} copies={copies} onClose={close} />
      )}
      <ConfirmModal
        opened={dialog?.kind === 'delete'}
        title={t('sessions.deleteTitle')}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => dialog?.kind === 'delete' && remove.mutate(dialog.session.id)}
        onClose={close}
      />
    </Stack>
  );
}

/** The copy a new session starts with: the last one played, else the only owned copy. */
export function defaultCopy(game: GameDetail, copies: Map<number, string>): number | null {
  const last = game.sessions.find((session) => session.copy_id)?.copy_id;
  if (last && copies.has(last)) return last;
  return copies.size === 1 ? [...copies.keys()][0]! : null;
}

function SessionForm({
  game,
  session,
  copies,
  onClose,
}: {
  game: GameDetail;
  session?: PlaySession;
  copies: Map<number, string>;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const saved = useGameSaved();
  const form = useForm({
    initialValues: {
      played_on: session?.played_on ?? today(),
      copy_id: session ? (session.copy_id ? String(session.copy_id) : null) : String(defaultCopy(game, copies) ?? '') || null,
      minutes: (session?.minutes ?? '') as string | number,
      notes: session?.notes ?? '',
    },
  });
  const onApiError = useApiErrorHandler(form);

  const save = useMutation({
    mutationFn: (v: typeof form.values) => {
      const body = {
        played_on: v.played_on,
        copy_id: v.copy_id ? Number(v.copy_id) : null,
        minutes: numberOrNull(v.minutes),
        notes: v.notes.trim() ? v.notes : null,
      };
      return session
        ? call(api.PUT('/api/sessions/{id}', { params: { path: { id: session.id } }, body }))
        : call(api.POST('/api/games/{id}/sessions', { params: { path: { id: game.id } }, body }));
    },
    onSuccess: (updated) => {
      saved(updated);
      onClose();
    },
    onError: onApiError,
  });

  return (
    <FormModal
      opened
      title={session ? t('sessions.edit') : t('sessions.add')}
      saving={save.isPending}
      onClose={onClose}
      onSubmit={form.onSubmit((v) => save.mutate(v))}
    >
      <TextInput type="date" label={t('sessions.playedOn')} required {...form.getInputProps('played_on')} />
      <Select
        label={t('sessions.copy')}
        placeholder={t('common.notSet')}
        data={[...copies].map(([id, label]) => ({ value: String(id), label }))}
        clearable
        disabled={copies.size === 0}
        {...form.getInputProps('copy_id')}
      />
      <NumberInput
        label={t('sessions.minutesLabel')}
        min={1}
        max={1440}
        allowDecimal={false}
        {...form.getInputProps('minutes')}
      />
      <Textarea label={t('common.notes')} autosize minRows={2} {...form.getInputProps('notes')} />
    </FormModal>
  );
}
