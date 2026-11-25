import { Button, Group, Loader, Modal, Stack, Text } from '@mantine/core';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys } from '../../api/queries';
import type { CopyDetail } from '../../api/types';
import { notifySuccess } from '../../components/notify';

type Props = { gameId: number; copy: CopyDetail; onClose: () => void };

// A cancel sent before its start finishes could clear nothing and leave the copy waiting.
let last: Promise<unknown> = Promise.resolve();
function inOrder<T>(request: () => Promise<T>): Promise<T> {
  const next = last.then(request, request);
  last = next.catch(() => undefined);
  return next;
}

/** Waits for the next tag scanned in Home Assistant and links it to the copy. */
export function TagLinkModal({ gameId, copy, onClose }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  // Tags linked before this dialog opened. Scanning one of them again links it anew.
  const tagKey = (tag: CopyDetail['tags'][number]) => `${tag.tag_id} ${tag.created_at}`;
  const [known] = useState(() => new Set(copy.tags.map(tagKey)));
  const [expiresAt, setExpiresAt] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    let cancelled = false;
    inOrder(() => call(api.POST('/api/copies/{id}/tag-link', { params: { path: { id: copy.id } } })))
      .then((link) => !cancelled && setExpiresAt(Date.parse(link.expires_at)))
      .catch((err: unknown) => !cancelled && setError(errorMessage(t, err)));
    return () => {
      cancelled = true;
      void inOrder(() => api.DELETE('/api/copies/{id}/tag-link', { params: { path: { id: copy.id } } }));
    };
  }, [copy.id, t]);

  const waiting = expiresAt != null && now < expiresAt;
  const game = useQuery({
    queryKey: queryKeys.game(gameId),
    queryFn: () => call(api.GET('/api/games/{id}', { params: { path: { id: gameId } } })),
    refetchInterval: waiting ? 1500 : false,
  });
  useEffect(() => {
    if (!waiting) return;
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, [waiting]);

  const current = game.data?.releases.flatMap((r) => r.copies).find((c) => c.id === copy.id);
  const linked = current?.tags.some((tag) => !known.has(tagKey(tag)));
  useEffect(() => {
    if (!linked) return;
    notifySuccess(t('tags.linked'));
    void queryClient.invalidateQueries({ queryKey: queryKeys.games });
    onClose();
  }, [linked, onClose, queryClient, t]);

  const seconds = expiresAt ? Math.max(0, Math.ceil((expiresAt - now) / 1000)) : null;
  return (
    <Modal opened onClose={onClose} title={t('tags.linkTitle')} centered>
      <Stack>
        {error ? (
          <Text size="sm" c="red">
            {error}
          </Text>
        ) : expiresAt == null ? (
          // Scans count only once the app has taken the link.
          <Loader size="sm" data-testid="tag-starting" />
        ) : waiting ? (
          <Group gap="sm" wrap="nowrap" data-testid="tag-waiting">
            <Loader size="sm" />
            <Text size="sm">{t('tags.scanNow', { seconds })}</Text>
          </Group>
        ) : (
          <Text size="sm">{t('tags.expired')}</Text>
        )}
        <Group justify="flex-end">
          <Button variant="default" onClick={onClose}>
            {t('common.cancel')}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
