import { ActionIcon, Button, Group, Modal, Text } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { IconChevronLeft, IconChevronRight, IconTrash } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import type { CopyPhoto } from '../../api/types';
import { notifyError } from '../../components/notify';
import { useGameSaved } from './forms';
import classes from './Copies.module.css';
import { useFormatDate } from '../../hooks/useFormatDate';

type Props = { photos: CopyPhoto[]; start: number; canDelete: boolean; onClose: () => void };

/** One copy's photos, full size, one at a time. */
export function PhotoViewer({ photos, start, canDelete, onClose }: Props) {
  const { t } = useTranslation();
  const formatDate = useFormatDate();
  const phone = useMediaQuery('(max-width: 48em)');
  const saved = useGameSaved();
  const [index, setIndex] = useState(start);
  const photo = photos[Math.min(index, photos.length - 1)];

  const remove = useMutation({
    mutationFn: (id: number) => call(api.DELETE('/api/photos/{id}', { params: { path: { id } } })),
    onSuccess: (game) => {
      saved(game, t('common.deleted'));
      if (photos.length <= 1) onClose();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  if (!photo) return null;
  const count = photos.length;
  return (
    <Modal opened onClose={onClose} title={t('photos.title', { number: index + 1, count })} size="xl" fullScreen={phone} centered={!phone}>
      <img src={`covers/${photo.path}`} alt={t('photos.alt', { number: index + 1 })} className={classes.photo} />
      <Group justify="space-between" mt="sm">
        <Group gap={4}>
          <ActionIcon variant="default" aria-label={t('photos.previous')} disabled={index === 0} onClick={() => setIndex(index - 1)}>
            <IconChevronLeft size={16} />
          </ActionIcon>
          <ActionIcon variant="default" aria-label={t('photos.next')} disabled={index >= count - 1} onClick={() => setIndex(index + 1)}>
            <IconChevronRight size={16} />
          </ActionIcon>
          <Text size="xs" c="dimmed" ml="xs">
            {formatDate(photo.created_at)}
          </Text>
        </Group>
        {canDelete && (
          <Button
            variant="subtle"
            color="red"
            leftSection={<IconTrash size={16} />}
            loading={remove.isPending}
            onClick={() => remove.mutate(photo.id)}
          >
            {t('photos.delete')}
          </Button>
        )}
      </Group>
    </Modal>
  );
}
