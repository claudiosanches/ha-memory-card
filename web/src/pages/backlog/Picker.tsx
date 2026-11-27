import { Button, Group, Select, Text, Title } from '@mantine/core';
import { IconDice5, IconPlayerPlay } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { useAccounts, useDevices, useGameDevices, usePlatforms } from '../../api/queries';
import type { Suggestion } from '../../api/types';
import { Description } from '../../components/Description';
import { Cover } from '../../components/Cover';
import { playTime } from '../stats/StatsPage';
import classes from './BacklogPage.module.css';
import { useSetPlayStatus } from './usePlayStatus';

/** Hours to beat the main story, at most, offered in the picker. */
const HOUR_LIMITS = [2, 5, 10, 20, 40, 60, 100];

/** Rolls a random game from your own backlog, optionally one a device can play or a short one. */
export function Picker() {
  const { t } = useTranslation();
  const devices = useGameDevices();
  const [device, setDevice] = useState<string | null>(null);
  const [hours, setHours] = useState<string | null>(null);
  // Kept apart from the mutation, which drops its data while the next roll loads.
  const [picked, setPicked] = useState<{ suggestion: Suggestion; roll: number } | null>(null);
  const setStatus = useSetPlayStatus();

  const pick = useMutation({
    mutationFn: () =>
      call(
        api.POST('/api/suggestions', {
          body: {
            ...(device ? { device: [device] } : {}),
            ...(hours ? { max_hours_to_beat: Number(hours) } : {}),
          },
        }),
      ),
    onSuccess: (suggestion) => setPicked((last) => ({ suggestion, roll: (last?.roll ?? 0) + 1 })),
    onError: () => setPicked(null),
  });

  const start = (id: number) =>
    setStatus.mutate(
      { id, status: 'playing' },
      {
        onSuccess: () => {
          setPicked(null);
          pick.reset();
        },
      },
    );

  return (
    <section className={classes.picker}>
      <div className={classes.pickerIntro}>
        <Title order={2} className={classes.sectionTitle}>
          {t('backlog.pickTitle')}
        </Title>
        <Description>
          {t('backlog.pickText')}
        </Description>
        <Group className={classes.pickerControls} gap="sm" align="flex-end" mt="xs">
          <Select
            label={t('backlog.device')}
            placeholder={t('backlog.anyDevice')}
            data={(devices.data ?? [])
              .filter((d) => d.active)
              .map((d) => ({ value: d.slug, label: d.name }))}
            value={device}
            onChange={setDevice}
            clearable
            className={classes.deviceSelect}
          />
          <Select
            label={t('backlog.length')}
            placeholder={t('backlog.anyLength')}
            data={HOUR_LIMITS.map((count) => ({ value: String(count), label: t('backlog.upToHours', { count }) }))}
            value={hours}
            onChange={setHours}
            clearable
            className={classes.deviceSelect}
          />
          <Button
            leftSection={<IconDice5 size={18} className={pick.isPending ? classes.rolling : undefined} />}
            onClick={() => pick.mutate()}
            disabled={pick.isPending}
          >
            {picked ? t('backlog.pickAgain') : t('backlog.pick')}
          </Button>
        </Group>
      </div>

      <div
        className={classes.pickerResult}
        data-empty={(!picked && !pick.isError) || undefined}
        aria-live="polite"
        aria-busy={pick.isPending}
      >
        {picked ? (
          <Picked
            key={picked.roll}
            suggestion={picked.suggestion}
            rolling={pick.isPending}
            starting={setStatus.isPending}
            onStart={() => start(picked.suggestion.game.id)}
          />
        ) : pick.isError ? (
          <Text size="sm" c="dimmed">
            {errorMessage(t, pick.error)}
          </Text>
        ) : (
          <div className={classes.pickSlot} aria-hidden>
            <IconDice5 size={28} stroke={1.4} className={pick.isPending ? classes.rolling : undefined} />
          </div>
        )}
      </div>
    </section>
  );
}

type PickedProps = { suggestion: Suggestion; rolling: boolean; starting: boolean; onStart: () => void };

function Picked({ suggestion, rolling, starting, onStart }: PickedProps) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const devices = useDevices();
  const accounts = useAccounts();
  const { game, copy_ids } = suggestion;

  const where = new Set<string>();
  for (const release of game.releases) {
    for (const copy of release.copies) {
      if (!copy_ids.includes(copy.id)) continue;
      const platform = platforms.data?.find((p) => p.id === release.platform_id)?.name;
      const device = devices.data?.find((d) => d.id === copy.device_id)?.name;
      const account = accounts.data?.find((a) => a.id === copy.account_id)?.name;
      where.add([platform, t(`format.${copy.format}`), device, account].filter(Boolean).join(t('common.separator')));
    }
  }

  return (
    <div className={classes.picked} data-rolling={rolling || undefined} data-testid="picked">
      <Link to="/games/$gameId" params={{ gameId: String(game.id) }} className={classes.pickedCover}>
        <Cover path={game.cover_path} title={game.title} width={96} />
      </Link>
      <div className={classes.pickedText}>
        <Text fw={700} size="lg" lineClamp={2} className={classes.gameTitle}>
          {game.title}
        </Text>
        {[...where].map((line) => (
          <Text key={line} size="sm" c="dimmed">
            {line}
          </Text>
        ))}
        {game.time_to_beat.main != null && (
          <Text size="sm" c="dimmed">
            {t('game.beatMain', { time: playTime(game.time_to_beat.main, t) })}
          </Text>
        )}
        <Group className={classes.pickedActions}>
          <Button leftSection={<IconPlayerPlay size={16} />} onClick={onStart} loading={starting} disabled={rolling}>
            {t('backlog.start')}
          </Button>
        </Group>
      </div>
    </div>
  );
}
