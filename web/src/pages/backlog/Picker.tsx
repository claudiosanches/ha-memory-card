import { Button, Group, Select, Text, Title } from '@mantine/core';
import { IconDice5, IconPlayerPlay } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { useDevices, usePlatforms } from '../../api/queries';
import type { Suggestion } from '../../api/types';
import { Cover } from '../../components/Cover';
import classes from './BacklogPage.module.css';
import { useSetPlayStatus } from './usePlayStatus';

/** Rolls a random backlog game, optionally one a device can play. */
export function Picker() {
  const { t } = useTranslation();
  const devices = useDevices();
  const [device, setDevice] = useState<string | null>(null);
  const setStatus = useSetPlayStatus();

  const pick = useMutation({
    mutationFn: () => call(api.POST('/api/suggestions', { body: device ? { device: [Number(device)] } : {} })),
  });

  return (
    <section className={classes.picker}>
      <div className={classes.pickerIntro}>
        <Title order={2} className={classes.sectionTitle}>
          {t('backlog.pickTitle')}
        </Title>
        <Text size="sm" c="dimmed">
          {t('backlog.pickText')}
        </Text>
        <Group gap="sm" align="flex-end" mt="xs">
          <Select
            label={t('backlog.device')}
            placeholder={t('backlog.anyDevice')}
            data={(devices.data ?? [])
              .filter((d) => d.active)
              .map((d) => ({ value: String(d.id), label: d.name }))}
            value={device}
            onChange={setDevice}
            clearable
            w={220}
          />
          <Button
            leftSection={<IconDice5 size={18} className={pick.isPending ? classes.rolling : undefined} />}
            onClick={() => pick.mutate()}
            disabled={pick.isPending}
          >
            {pick.data ? t('backlog.pickAgain') : t('backlog.pick')}
          </Button>
        </Group>
      </div>

      {pick.isError && (
        <Text size="sm" c="dimmed" className={classes.pickerResult}>
          {errorMessage(t, pick.error)}
        </Text>
      )}
      {pick.data && (
        <Picked
          key={pick.submittedAt}
          suggestion={pick.data}
          starting={setStatus.isPending}
          onStart={() =>
            pick.data && setStatus.mutate({ id: pick.data.game.id, status: 'playing' }, { onSuccess: () => pick.reset() })
          }
        />
      )}
    </section>
  );
}

function Picked({ suggestion, starting, onStart }: { suggestion: Suggestion; starting: boolean; onStart: () => void }) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const devices = useDevices();
  const { game, copy_ids } = suggestion;

  const where = new Set<string>();
  for (const release of game.releases) {
    for (const copy of release.copies) {
      if (!copy_ids.includes(copy.id)) continue;
      const platform = platforms.data?.find((p) => p.id === release.platform_id)?.name;
      const device = devices.data?.find((d) => d.id === copy.device_id)?.name;
      where.add([platform, t(`format.${copy.format}`), device].filter(Boolean).join(t('common.separator')));
    }
  }

  return (
    <div className={`${classes.pickerResult} ${classes.picked}`} data-testid="picked">
      <Link to="/games/$gameId" params={{ gameId: String(game.id) }} className={classes.pickedCover}>
        <Cover path={game.cover_path} title={game.title} width={96} />
      </Link>
      <div className={classes.rowText}>
        <Text fw={700} size="lg" className={classes.gameTitle}>
          {game.title}
        </Text>
        {[...where].map((line) => (
          <Text key={line} size="sm" c="dimmed">
            {line}
          </Text>
        ))}
        <Group mt="sm">
          <Button leftSection={<IconPlayerPlay size={16} />} onClick={onStart} loading={starting}>
            {t('backlog.start')}
          </Button>
        </Group>
      </div>
    </div>
  );
}
