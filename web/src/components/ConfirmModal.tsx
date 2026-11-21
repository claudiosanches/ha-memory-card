import { Button, Group, Modal, Stack, Text } from '@mantine/core';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

type Props = {
  opened: boolean;
  title: string;
  message?: string;
  confirmLabel: string;
  busy: boolean;
  onConfirm: () => void;
  onClose: () => void;
  /** Another button before the confirm button. */
  extraAction?: ReactNode;
};

export function ConfirmModal({ opened, title, message, confirmLabel, busy, onConfirm, onClose, extraAction }: Props) {
  const { t } = useTranslation();
  return (
    <Modal opened={opened} onClose={onClose} title={title} centered>
      <Stack>
        {message && <Text size="sm">{message}</Text>}
        <Group justify="flex-end">
          <Button variant="default" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          {extraAction}
          <Button color="red" onClick={onConfirm} loading={busy}>
            {confirmLabel}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
