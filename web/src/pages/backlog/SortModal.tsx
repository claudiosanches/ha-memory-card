import { Button, Kbd, Modal, Progress, Stack, Text } from '@mantine/core';
import { useHotkeys, useMediaQuery } from '@mantine/hooks';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { GameSummary } from '../../api/types';
import { Cover } from '../../components/Cover';
import { usePlayStatusList } from '../../components/playStatuses';
import classes from './BacklogPage.module.css';
import { useSetPlayStatus } from './usePlayStatus';

/** Walks through games without a play status, one at a time. */
export function SortModal({
  games,
  platformNames,
  onClose,
}: {
  games: GameSummary[];
  platformNames: Map<number, string>;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const phone = useMediaQuery('(max-width: 48em)');
  // A snapshot, so the list does not shift while statuses save.
  const [queue, setQueue] = useState(games);
  const [index, setIndex] = useState(0);
  const setStatus = useSetPlayStatus({ quiet: true });
  const statuses = usePlayStatusList();
  const game = queue[index];

  const choose = (status: string) => {
    if (!game) return;
    // A status that fails to save brings its game back at the end.
    void setStatus.mutateAsync({ id: game.id, status }).catch(() => setQueue((current) => [...current, game]));
    setIndex(index + 1);
  };
  const skip = () => game && setIndex(index + 1);

  // Number keys for the first nine statuses.
  const keyed = statuses.list.slice(0, 9);
  useHotkeys([...keyed.map((status, i): [string, () => void] => [String(i + 1), () => choose(status.key)]), ['s', skip]]);
  const key = (label: string) =>
    phone ? undefined : (
      <Kbd size="xs" aria-hidden>
        {label}
      </Kbd>
    );

  return (
    <Modal opened onClose={onClose} title={t('backlog.sortTitle')} fullScreen={phone} centered={!phone}>
      <Stack gap="md">
        <div>
          <Text size="sm" c="dimmed" mb={6}>
            {t('backlog.sortProgress', { done: Math.min(index, queue.length), total: queue.length })}
          </Text>
          <Progress value={(index / queue.length) * 100} aria-hidden />
        </div>

        {game ? (
          <div key={game.id} className={`${classes.sortGame} ${classes.picked}`} data-testid="sort-game">
            <Cover path={game.cover_path} title={game.title} width={phone ? 140 : 160} />
            <Text fw={700} size="lg" ta="center" className={classes.gameTitle}>
              {game.title}
            </Text>
            <Text size="sm" c="dimmed">
              {game.platforms
                .map((p) => platformNames.get(p.platform_id))
                .filter(Boolean)
                .join(t('common.separator'))}
            </Text>
          </div>
        ) : (
          <Text ta="center" py="xl" fw={600}>
            {t('backlog.sortDone')}
          </Text>
        )}

        {game ? (
          <div className={classes.choices}>
            {statuses.list.map((status, i) => (
              <Button
                key={status.key}
                color={status.color}
                variant="light"
                onClick={() => choose(status.key)}
                rightSection={i < 9 ? key(String(i + 1)) : undefined}
                aria-keyshortcuts={i < 9 ? String(i + 1) : undefined}
              >
                {status.name}
              </Button>
            ))}
            <Button variant="default" onClick={skip} rightSection={key('S')} aria-keyshortcuts="S">
              {t('backlog.skip')}
            </Button>
          </div>
        ) : (
          <Button onClick={onClose}>{t('backlog.done')}</Button>
        )}
      </Stack>
    </Modal>
  );
}
