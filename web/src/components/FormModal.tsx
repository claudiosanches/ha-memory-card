import { Button, Group, Modal, Stack } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import type { FormEvent, ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

type Props = {
  opened: boolean;
  title: string;
  saving: boolean;
  /** Keeps Save off while another action saves. */
  submitDisabled?: boolean;
  onClose: () => void;
  onSubmit: (event?: FormEvent<HTMLFormElement>) => void;
  /** Another button next to Save, such as "Save and add another". */
  extraAction?: ReactNode;
  submitLabel?: string;
  children: ReactNode;
};

/** Modal with a form and Save/Cancel buttons; full screen on phones. */
export function FormModal({
  opened,
  title,
  saving,
  submitDisabled,
  onClose,
  onSubmit,
  extraAction,
  submitLabel,
  children,
}: Props) {
  const { t } = useTranslation();
  const phone = useMediaQuery('(max-width: 48em)');

  return (
    // Centering a full-screen modal misplaces dropdowns inside it.
    <Modal opened={opened} onClose={onClose} title={title} fullScreen={phone} centered={!phone}>
      <form onSubmit={onSubmit} noValidate>
        <Stack>
          {children}
          <Group justify="flex-end">
            <Button variant="default" onClick={onClose}>
              {t('common.cancel')}
            </Button>
            {extraAction}
            <Button type="submit" loading={saving} disabled={submitDisabled}>
              {submitLabel ?? t('common.save')}
            </Button>
          </Group>
        </Stack>
      </form>
    </Modal>
  );
}
